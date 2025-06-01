use crate::{
    air::MemoryAirBuilder,
    memory::{value_as_limbs, MemoryCols, MemoryReadCols, MemoryWriteCols},
    operations::field::field_op::FieldOpCols,
    utils::{limbs_from_access, pad_rows_fixed, words_to_bytes_le},
};
use log::debug;
use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{AbstractField, PrimeField32};
use p3_matrix::{dense::RowMajorMatrix, Matrix};
use sp1_core_executor::{
    events::{ByteRecord, PrecompileEvent},
    syscalls::SyscallCode,
    ExecutionRecord, Program,
};
use sp1_derive::AlignedBorrow;
use sp1_stark::{
    air::{InteractionScope, MachineAir, SP1AirBuilder},
    MachineRecord,
};
use std::{
    borrow::{Borrow, BorrowMut},
    mem::size_of,
};

const MAX_VECTOR_LEN: usize = 350;
const NUM_COLS: usize = size_of::<InnerProductCols<u8>>();

#[derive(Default)]
pub struct InnerProductChip;

impl InnerProductChip {
    // `const` that allows functions to be evaluated at compile time rather than runtime.
    pub const fn new() -> Self {
        Self
    }
}

/// A set of columns for the innper product operation.
/// NOTE: the size of the trace must be fixed at compile time!!!
#[derive(Debug, Clone, AlignedBorrow)]
#[repr(C)]
pub struct InnerProductCols<T> {
    /// The shard number of the syscall.
    pub shard: T,

    /// The clock cycle of the syscall.
    pub clk: T,

    /// The pointer to the first input.
    pub a_ptr: T,

    /// The pointer to the second input.
    pub b_ptr: T,

    /// Memory read records for vector lengths
    pub a_len_memory: MemoryReadCols<T>,
    pub b_len_memory: MemoryReadCols<T>,

    /// The actual length value (this will be constrained to match the memory read)
    pub len: T,

    /// Memory columns for vector elements - using dynamic size
    /// We'll need to determine max supported vector length at compile time
    pub a_memory: [MemoryReadCols<T>; MAX_VECTOR_LEN + 1],
    pub b_memory: [MemoryReadCols<T>; MAX_VECTOR_LEN + 1],

    /// Memory column for writing result
    pub result_memory: MemoryWriteCols<T>,

    /// Running sum for accumulating the inner product
    pub running_sum: T,

    /// Is this row a real computation vs padding
    pub is_real: T,
}

impl<F: PrimeField32> MachineAir<F> for InnerProductChip {
    type Record = ExecutionRecord;
    type Program = Program;

    fn name(&self) -> String {
        "InnerProduct".to_string()
    }

    fn generate_trace(
        &self,
        input: &ExecutionRecord, // syscalls, memory operations, precompile events, program execution state
        output: &mut ExecutionRecord,
    ) -> RowMajorMatrix<F> {
        debug!("invoking generate_trace");
        let rows_and_records = input
            .get_precompile_events(SyscallCode::INNER_PRODUCT)
            .chunks(1)
            .map(|events| {
                // creates a new empty ExecutionRecord instance by default()
                let mut records = ExecutionRecord::default();
                let mut new_byte_lookup_events = Vec::new();

                let rows = events
                    .iter()
                    .map(|(_, event)| {
                        let event = if let PrecompileEvent::InnerProduct(event) = event {
                            event
                        } else {
                            unreachable!()
                        };

                        println!("=== TRACE GENERATION ===");
                        println!(
                            "Event at clk={}, a_ptr={}, b_ptr={}",
                            event.clk, event.a_ptr, event.b_ptr
                        );
                        println!("Vector a has {} elements", event.a.len());
                        println!("Vector b has {} elements", event.b.len());
                        println!("a_memory_records has {} records", event.a_memory_records.len());
                        println!("b_memory_records has {} records", event.b_memory_records.len());

                        // init an empty row
                        let mut row: [F; NUM_COLS] = [F::zero(); NUM_COLS];
                        let cols: &mut InnerProductCols<F> = row.as_mut_slice().borrow_mut();

                        // metadata fields
                        cols.is_real = F::one();
                        cols.shard = F::from_canonical_u32(event.shard);
                        cols.clk = F::from_canonical_u32(event.clk);
                        cols.a_ptr = F::from_canonical_u32(event.a_ptr);
                        cols.b_ptr = F::from_canonical_u32(event.b_ptr);

                        // Length reads
                        cols.a_len_memory.populate(event.a_len_memory, &mut new_byte_lookup_events); //33
                        cols.b_len_memory.populate(event.b_len_memory, &mut new_byte_lookup_events); // 33
                        cols.len = F::from_canonical_u32(event.a.len() as u32);
                        debug!("inner product rs len: {}", event.a.len());
                        // Vector reads
                        for i in 0..=MAX_VECTOR_LEN {
                            cols.a_memory[i]
                                .populate(event.a_memory_records[i], &mut new_byte_lookup_events);
                            cols.b_memory[i]
                                .populate(event.b_memory_records[i], &mut new_byte_lookup_events);
                        }

                        // Compute inner product
                        let mut sum = 0u32;
                        for i in 1..event.a.len() {
                            // println!("inner product rs a[i]: {}", event.a[i]);
                            // println!("inner product rs b[i]: {}", event.b[i]);
                            sum += event.a[i] * event.b[i];
                        }
                        debug!("inner product rs sum: {}", sum);
                        cols.running_sum = F::from_canonical_u32(sum);

                        // Result write
                        cols.result_memory
                            .populate(event.result_memory_records, &mut new_byte_lookup_events);

                        //debug!("row.len(): {:?}", row.len());
                        row
                    })
                    .collect::<Vec<_>>();

                records.add_byte_lookup_events(new_byte_lookup_events);
                (rows, records)
            })
            .collect::<Vec<_>>();

        // Collect rows and pad
        let mut rows = Vec::new();
        for (row, mut record) in rows_and_records {
            rows.extend(row);
            output.append(&mut record);
        }

        // Pad rows to power of 2
        debug!("Number of rows before padding: {}", rows.len());
        pad_rows_fixed(
            &mut rows,
            || {
                let mut row: [F; NUM_COLS] = [F::zero(); NUM_COLS];
                let cols: &mut InnerProductCols<F> = row.as_mut_slice().borrow_mut();
                cols.is_real = F::zero();
                // debug!("row: {:?}", row);
                // debug!(
                //     "input.fixed_log2_rows::<F, _>(self),: {:?}",
                //     input.fixed_log2_rows::<F, _>(self)
                // );
                row
            },
            //input.fixed_log2_rows::<F, _>(self),
            Some(10),
        );
        debug!("Number of rows after padding: {}", rows.len());
        // debug!("rows.len(): {:?}", rows.len());

        // debug!(
        //     "RowMajorMatrix: {:?}",
        //     RowMajorMatrix::new(rows.clone().into_iter().flatten().collect::<Vec<_>>(), NUM_COLS)
        // );
        let matrix = RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_COLS);

        debug!("Generated matrix dimensions: {} rows x {} cols", matrix.height(), matrix.width());
        debug!("Matrix height is power of 2: {}", matrix.height().is_power_of_two());
        debug!("Matrix height log2: {}", matrix.height().trailing_zeros());
        matrix
    }

    fn included(&self, shard: &Self::Record) -> bool {
        if let Some(shape) = shard.shape.as_ref() {
            shape.included::<F, _>(self)
        } else {
            !shard.get_precompile_events(SyscallCode::INNER_PRODUCT).is_empty()
        }
    }
}

// defines the basic algebraic constraints that must be satisfied for a valid execution.
impl<F> BaseAir<F> for InnerProductChip {
    fn width(&self) -> usize {
        NUM_COLS
    }
}

impl<AB> Air<AB> for InnerProductChip
where
    AB: SP1AirBuilder,
{
    fn eval(&self, builder: &mut AB) {
        debug!("=== CONSTRAINT MEMORY OPERATIONS ===");
        let main = builder.main();
        debug!("main dimensions: {:?}", main.dimensions());

        // Check if we have any rows at all
        if main.height() == 0 {
            debug!("WARNING: main height is 0!");
            return;
        }

        let local = main.row_slice(0);

        debug!("Successfully got row slice");
        let local: &InnerProductCols<AB::Var> = (*local).borrow();

        // Assert is_real is boolean
        builder.assert_bool(local.is_real);

        //  Verify the inner product syscall was properly registered
        builder.receive_syscall(
            local.shard,
            local.clk,
            AB::F::from_canonical_u32(SyscallCode::INNER_PRODUCT.syscall_id()),
            local.a_ptr,
            local.b_ptr,
            local.is_real,
            InteractionScope::Local,
        );

        // Verify length reads
        builder.when(local.is_real).eval_memory_access(
            local.shard,
            local.clk.into(),
            local.a_ptr,
            &local.a_len_memory,
            local.is_real,
        );

        builder.when(local.is_real).eval_memory_access(
            local.shard,
            local.clk.into(),
            local.b_ptr,
            &local.b_len_memory,
            local.is_real,
        );

        // Verify both lengths are equal
        builder.when(local.is_real).assert_eq(
            local.a_len_memory.value().reduce::<AB>(),
            local.b_len_memory.value().reduce::<AB>(),
        );

        // Verify length matches what we recorded in the trace
        builder.when(local.is_real).assert_eq(local.a_len_memory.value().reduce::<AB>(), local.len);

        // Verify vector reads
        for i in 0..=MAX_VECTOR_LEN {
            // Calculate memory addresses for each element
            let a_addr = local.a_ptr + AB::Expr::from_canonical_u32(4 + i as u32 * 4);
            let b_addr = local.b_ptr + AB::Expr::from_canonical_u32(4 + i as u32 * 4);

            // Verify memory accesses
            builder.when(local.is_real).eval_memory_access(
                local.shard,
                local.clk.into(),
                a_addr,
                &local.a_memory[i],
                local.is_real,
            );

            builder.when(local.is_real).eval_memory_access(
                local.shard,
                local.clk.into(),
                b_addr,
                &local.b_memory[i],
                local.is_real,
            );
        }

        // Calculate the inner product
        let mut running_sum = AB::Expr::zero();
        for i in 1..=MAX_VECTOR_LEN {
            // Start from 1 to match trace generation
            //println!("Processing index {}", i);
            let a_value = local.a_memory[i].value().reduce::<AB>();
            //println!("a_value processed {:?}", a_value);
            let b_value = local.b_memory[i].value().reduce::<AB>();
            //println!("b_value processed {:?}", b_value);
            running_sum = running_sum + (a_value * b_value);
        }

        // Verify the calculated inner product matches what's in the trace
        builder.when(local.is_real).assert_eq(running_sum, local.running_sum);
        debug!("verified local.running_sum equals running_sum");

        // Verify result write - at clk+1, the result should be written to a_ptr
        builder.when(local.is_real).eval_memory_access(
            local.shard,
            local.clk.into() + AB::Expr::one(),
            local.a_ptr,
            &local.result_memory,
            local.is_real,
        );
        debug!("verified result write - at clk+1");

        // Verify the memory value written is the computed inner product
        builder
            .when(local.is_real)
            .assert_eq(local.result_memory.value().reduce::<AB>(), local.running_sum);
        debug!("verified the memory value written is the computed inner product");

        debug!("Total memory accesses: {}", 2 + 2 * (MAX_VECTOR_LEN + 1) + 1);
    }
}
