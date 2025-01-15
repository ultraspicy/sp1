use p3_field::{AbstractField, PrimeField32};
use p3_matrix::{dense::RowMajorMatrix, Matrix};
use p3_air::{Air, AirBuilder, BaseAir};

use sp1_core_executor::{
    events::{ByteRecord, FieldOperation, PrecompileEvent},
    syscalls::SyscallCode,
    ExecutionRecord, Program,
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

/// The number of columns in the AddmulCols.
const NUM_COLS: usize = size_of::<AddmulCols<u8>>();

#[derive(Default)]
pub struct AddmulChip;

impl AddmulChip {
    pub const fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, AlignedBorrow)]
#[repr(C)]
pub struct AddmulCols<T> {
    /// The shard number of the syscall.
    pub shard: T,

    /// The clock cycle of the syscall.
    pub clk: T,

    /// The nonce of the operation.
    pub nonce: T,

    pub p: T,
    pub q: T,
    pub r: T,
    pub s: T,

    // The result (what gets written to p)
    pub result: T,
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
            .get_precompile_events(SyscallCode::ADD_MUL)
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

        // Convert the trace to a row major matrix.
        let mut trace =
            RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_COLS);

        // Write the nonces to the trace.
        for i in 0..trace.height() {
            let cols: &mut AddmulCols<F> =
                trace.values[i * NUM_COLS..(i + 1) * NUM_COLS].borrow_mut();
            cols.nonce = F::from_canonical_usize(i);
        }

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
        // no-op
    }
}
