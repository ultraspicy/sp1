use crate::{
    memory::{value_as_limbs, MemoryReadCols, MemoryWriteCols},
    operations::field::field_op::FieldOpCols,
};

use sp1_core_executor::{
    events::{ByteRecord, FieldOperation, PrecompileEvent},
    syscalls::SyscallCode,
    ExecutionRecord, Program,
};

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{AbstractField, PrimeField32};

use crate::{
    air::MemoryAirBuilder,
    operations::{field::range::FieldLtCols, IsZeroOperation},
    utils::{
        limbs_from_access, limbs_from_prev_access, pad_rows_fixed, words_to_bytes_le,
        words_to_bytes_le_vec,
    },
};

use p3_matrix::{dense::RowMajorMatrix, Matrix};
use sp1_derive::AlignedBorrow;
use sp1_stark::{
    air::{BaseAirBuilder, InteractionScope, MachineAir, Polynomial, SP1AirBuilder},
    MachineRecord,
};

const NUM_COLS: usize = size_of::<AddMulChipCols<u8>>();

#[derive(Default)]
pub struct AddMulChip;

impl AddMulChip {
    pub const fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, AlignedBorrow)]
#[repr(C)]
pub struct AddMulChipCols<T> {
    /// The shard number of the operation
    pub shard: T,

    /// The clock cycle
    pub clk: T,

    /// Unique identifier for this operation
    pub nonce: T,

    // Memory pointers for inputs
    pub a: T,
    pub b: T,
    pub c: T,
    pub d: T,
}

impl<F: PrimeField32> MachineAir<F> for AddMulChip {
    type Record = ExecutionRecord;
    type Program = Program;

    fn name(&self) -> String {
        "AddMul".to_string()
    }

    fn generate_trace(
        &self,
        input: &ExecutionRecord, //  Contains the input state before the precompile execution
        output: &mut ExecutionRecord, // Will contain the resulting state after execution
    ) -> RowMajorMatrix<F> {
        // A matrix storing the execution trace in row-major format,
        // Generate trace rows for each event
        let rows_and_records = input
            .get_precompile_events(SyscallCode::ADDMUL)
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

                        row
                    })
                    .collect::<Vec<_>>();
                records.add_byte_lookup_events(new_byte_lookup_events);
                (rows, records)
            })
            .collect::<Vec<_>>();

        // Collect all rows
        let mut rows = Vec::new();
        for (row, mut record) in rows_and_records {
            rows.extend(row);
            output.append(&mut record);
        }
        // Create matrix and add nonces
        let mut trace =
            RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_COLS);

        // // Write nonces to trace
        // for i in 0..trace.height() {
        //     let cols: &mut AddMulChipCols<F> =
        //         trace.values[i * NUM_COLS..(i + 1) * NUM_COLS].borrow_mut();
        //     cols.nonce = F::from_canonical_usize(i);
        // }

        trace
    }

    fn included(&self, shard: &Self::Record) -> bool {
        if let Some(shape) = shard.shape.as_ref() {
            shape.included::<F, _>(self)
        } else {
            !shard.get_precompile_events(SyscallCode::ADDMUL).is_empty()
        }
    }
}

impl<F> BaseAir<F> for AddMulChip {
    fn width(&self) -> usize {
        NUM_COLS
    }
}

impl<AB> Air<AB> for AddMulChip
where
    AB: SP1AirBuilder,
{
    fn eval(&self, builder: &mut AB) {}
}
