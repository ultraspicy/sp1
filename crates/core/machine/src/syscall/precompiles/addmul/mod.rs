mod addmul;

pub use addmul::*;

#[cfg(test)]
mod tests {
    use p3_baby_bear::BabyBear;
    use p3_matrix::dense::RowMajorMatrix;
    use rand::Rng;
    use sp1_core_executor::events::AddMulEvent;
    use crate::{
        io::SP1Stdin,
        utils::{
            self,
            run_test_io,
            uni_stark_prove as prove,
            uni_stark_verify as verify
        },
    };
    use sp1_stark::{
        air::MachineAir, baby_bear_poseidon2::BabyBearPoseidon2, CpuProver, StarkGenericConfig,
    };
    use test_artifacts::ADD_MUL_ELF;
    use test_artifacts::ADD_MUL_NATIVE_ELF;
    use sp1_core_executor::{
        events::{
            LookupId, MemoryReadRecord, MemoryWriteRecord, PrecompileEvent, SyscallEvent
        },
        syscalls::SyscallCode,
        ExecutionRecord, Program,
    };
    use crate::syscall::precompiles::addmul::AddmulChip;

    #[test]
    fn test_add_mul_elf() {
        utils::setup_logger();
        println!("This test is running!");
        let program = Program::from(ADD_MUL_ELF).unwrap();
        println!("Program loaded successfully"); 
        run_test_io::<CpuProver<_, _>>(program, SP1Stdin::new()).unwrap();
    }
}