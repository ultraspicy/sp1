use num::{BigUint, One, Zero};
use sp1_primitives::consts::{bytes_to_words_le, words_to_bytes_le_vec, WORD_SIZE};
use crate::{
    events::{AddMulEvent, PrecompileEvent},
    syscalls::{Syscall, SyscallCode, SyscallContext},
};

use crate::Register::{X12, X13};

pub(crate) struct AddMulSyscall;

impl Syscall for AddMulSyscall {
    fn execute(
        &self,
        rt: &mut SyscallContext,
        syscall_code: SyscallCode,
        arg1: u32,
        arg2: u32,  
    ) -> Option<u32> {
        let clk = rt.clk;

        let (_, p) = rt.mr(arg1);
        let (_, q) = rt.mr(arg2);
        let (_, r) = rt.mr(X12 as u32);
        let (_, s) = rt.mr(X13 as u32);

        rt.clk += 1;

        // we know it won't overflow
        let ret = p * q + r * s;
        let memory_records = rt.mw(arg1, ret);

        let lookup_id = rt.syscall_lookup_id;
        let shard = rt.current_shard();
        let event = PrecompileEvent::ADDMul(AddMulEvent {
            lookup_id,
            shard,
            clk,
            p,
            q,
            r,
            s,
            local_mem_access: rt.postprocess(),
        });

        let sycall_event = rt.rt.syscall_event(clk, syscall_code.syscall_id(), arg1, arg2, lookup_id);
        rt.add_precompile_event(syscall_code, sycall_event, event);

        None
    }
}