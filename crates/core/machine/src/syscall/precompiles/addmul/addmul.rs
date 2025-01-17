use p3_field::{AbstractField, PrimeField32};
use p3_matrix::{dense::RowMajorMatrix, Matrix};
use p3_air::{Air, AirBuilder, BaseAir};
use crate::memory::{value_as_limbs, MemoryCols, MemoryReadCols, MemoryWriteCols};

use sp1_core_executor::{
    events::{ByteRecord, FieldOperation, PrecompileEvent},
    syscalls::SyscallCode,
    ExecutionRecord, Program, Register,
};
use sp1_derive::AlignedBorrow;
use sp1_stark::{
    air::{BaseAirBuilder, InteractionScope, MachineAir, Polynomial, SP1AirBuilder},
    MachineRecord,
};
use std::{
    borrow::{Borrow, BorrowMut},
    mem::size_of,
};

use crate::{
    air::MemoryAirBuilder,
    operations::{field::range::FieldLtCols, IsZeroOperation},
    utils::{
        limbs_from_access, limbs_from_prev_access, pad_rows_fixed, words_to_bytes_le,
        words_to_bytes_le_vec,
    },
};

/// The number of columns in the AddmulCols.
const NUM_COLS: usize = size_of::<AddmulCols<u8>>();
// Two extra register for passing in r and s
const LO_REGISTER: u32 = Register::X12 as u32;
const HI_REGISTER: u32 = Register::X13 as u32;

#[derive(Default)]
pub struct AddmulChip;

impl AddmulChip {
    pub const fn new() -> Self {
        Self
    }
}

// In STARK proofs, all values in the trace matrix need to be field elements of generic T
#[derive(Debug, Clone, AlignedBorrow)]
#[repr(C)]
pub struct AddmulCols<T> {
    /// The shard number of the syscall.
    pub shard: T,

    /// The clock cycle of the syscall.
    pub clk: T,

    /// The nonce of the operation.
    pub nonce: T,

    // Input values and pointers
    pub p: T,
    pub q: T,
    pub r: T,
    pub s: T,
    pub p_ptr: T,
    pub q_ptr: T,
    pub r_ptr: T,
    pub s_ptr: T,

    // Memory operations tracking
    pub p_memory: MemoryWriteCols<T>,
    pub q_memory: MemoryReadCols<T>,
    pub r_memory: MemoryReadCols<T>,
    pub s_memory: MemoryReadCols<T>,

    // The result (what gets written to p)
    pub result: T,

    // distinguish between actual computation steps and padding rows in the execution trace
    pub is_real: T,
}

impl<F: PrimeField32> MachineAir<F> for AddmulChip {
    type Record = ExecutionRecord;
    type Program = Program;

    fn name(&self) -> String {
        "AddmulMod".to_string()
    }

    fn generate_trace(
        &self,
        input: &ExecutionRecord,
        output: &mut ExecutionRecord,
    ) -> RowMajorMatrix<F> {
        let rows_and_records = input
            .get_precompile_events(SyscallCode::ADD_MUL) // get all the precompile events for a syscall code.
            .chunks(1)
            .map(|events| {
                let mut records = ExecutionRecord::default();
                let mut new_byte_lookup_events = Vec::new();

                let rows = events
                    .iter()
                    .map(|(_, event)| {
                        let event = if let PrecompileEvent::ADDMul(event) = event {
                            event
                        } else {
                            unreachable!()
                        };

                        let mut row: [F; NUM_COLS] = [F::zero(); NUM_COLS];
                        let cols: &mut AddmulCols<F> = row.as_mut_slice().borrow_mut();
                        
                        cols.shard = F::from_canonical_u32(event.shard);
                        cols.clk = F::from_canonical_u32(event.clk);
                        cols.is_real = F::one();
                    
                        cols.p = F::from_canonical_u32(event.p);
                        cols.q = F::from_canonical_u32(event.q);
                        cols.r = F::from_canonical_u32(event.r);
                        cols.s = F::from_canonical_u32(event.s);
                        
                        cols.p_ptr = F::from_canonical_u32(event.p_ptr);
                        cols.q_ptr = F::from_canonical_u32(event.q_ptr);
                        cols.r_ptr = F::from_canonical_u32(event.r_ptr);
                        cols.s_ptr = F::from_canonical_u32(event.s_ptr);

                        cols.result = F::from_canonical_u32(event.p * event.q + event.r * event.s);

                        cols.p_memory.populate(event.p_memory_records, &mut new_byte_lookup_events);
                        cols.q_memory.populate(event.q_memory_records, &mut new_byte_lookup_events);
                        cols.r_memory.populate(event.r_memory_records, &mut new_byte_lookup_events);
                        cols.s_memory.populate(event.s_memory_records, &mut new_byte_lookup_events);

                        println!("============================================");
                        println!("row: {:?}", row);
                        row
                        
                    })
                    .collect::<Vec<_>>();
                records.add_byte_lookup_events(new_byte_lookup_events);
                (rows, records)
            })
            .collect::<Vec<_>>();

        //  Generate the trace rows for each event.
        let mut rows = Vec::new();
        for (row, mut record) in rows_and_records {
            rows.extend(row);
            output.append(&mut record); 
        }

        // STARK needs row number to be power of 2 so we need to pad dummmy rows
        pad_rows_fixed(
            &mut rows,
            || {
                let mut row: [F; NUM_COLS] = [F::zero(); NUM_COLS];
                let cols: &mut AddmulCols<F> = row.as_mut_slice().borrow_mut();
                cols.is_real = F::zero(); 
                cols.result = F::zero();
                row
            },
            input.fixed_log2_rows::<F, _>(self),
        );

        // Convert the trace to a row major matrix.
        let mut trace =
            RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_COLS);

        // Write the nonces to the trace.
        for i in 0..trace.height() {
            let cols: &mut AddmulCols<F> =
                trace.values[i * NUM_COLS..(i + 1) * NUM_COLS].borrow_mut();
            cols.nonce = F::from_canonical_usize(i);
        }

        println!("============================================");
        println!("trace: {:?}", trace);
        trace
    }

    fn included(&self, shard: &Self::Record) -> bool {
        if let Some(shape) = shard.shape.as_ref() {
            shape.included::<F, _>(self)
        } else {
            !shard.get_precompile_events(SyscallCode::ADD_MUL).is_empty()
        }
    }
}

impl<F> BaseAir<F> for AddmulChip {
    fn width(&self) -> usize {
        NUM_COLS
    }
}

impl<AB> Air<AB> for AddmulChip
where
    AB: SP1AirBuilder,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        let local: &AddmulCols<AB::Var> = (*local).borrow();
        let next = main.row_slice(1);
        let next: &AddmulCols<AB::Var> = (*next).borrow();

        // Assert that is_real is a boolean.
        builder.assert_bool(local.is_real);

        // Evaluate the memory accesses
        builder.eval_memory_access(
            local.shard,
            local.clk.into() + AB::Expr::one(), // Add 1 to clock for write
            local.p_ptr,
            &local.p_memory,
            local.is_real,
        );

        builder.eval_memory_access(
            local.shard,
            local.clk.into(),
            local.q_ptr,
            &local.q_memory,
            local.is_real,
        );

        builder.eval_memory_access(
            local.shard,
            local.clk.into(),
            local.r_ptr,
            &local.r_memory,
            local.is_real,
        );

        builder.eval_memory_access(
            local.shard,
            local.clk.into(),
            local.s_ptr,
            &local.s_memory,
            local.is_real,
        );

        builder.receive_syscall(
            local.shard,           // The shard ID where this computation happens
            local.clk,             // Current clock cycle in the VM
            local.nonce,           // Unique identifier for this operation
            AB::F::from_canonical_u32(SyscallCode::ADD_MUL.syscall_id()), // Syscall type ID
            local.p_ptr,           // Memory pointer to first input
            local.q_ptr,           // Memory pointer to second input
            local.is_real,         // Whether this is a real computation or padding
            InteractionScope::Local, // Scope of the interaction
        );

        // Constrain the incrementing nonce.
        builder.when_first_row().assert_zero(local.nonce);
        builder.when_transition().assert_eq(local.nonce + AB::Expr::one(), next.nonce); 

        // Verify result computation
        let computed_result = local.p * local.q + local.r * local.s;
        builder.when(local.is_real).assert_eq(local.result, computed_result);
   
    }
}
