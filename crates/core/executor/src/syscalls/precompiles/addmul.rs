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

        //// Get the current value of a word, but doesn't use a memory record.
        // we will write back this memory with corresponding memory record
        // which will prove the value in memory before and after the write
        let p = rt.word_unsafe(arg1);

        let (q_memory_records, q) = rt.mr(arg2);
        let (r_memory_records, r) = rt.mr(X12 as u32);
        let (s_memory_records, s) = rt.mr(X13 as u32);
    
        rt.clk += 1;

        // ignore the overflow for now
        let ret = p * q + r * s;
        let p_memory_records = rt.mw(arg1, ret);

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
            p_ptr: arg1,           
            q_ptr: arg2,
            r_ptr: X12 as u32,    
            s_ptr: X13 as u32,
            p_memory_records,
            q_memory_records,
            r_memory_records,
            s_memory_records,
            local_mem_access: rt.postprocess(), // ### captures any local memory accesses that occurred during the syscall execution.
        });

        let sycall_event =
            rt.rt.syscall_event(clk, syscall_code.syscall_id(), arg1, arg2, lookup_id);
        rt.add_precompile_event(syscall_code, sycall_event, event);

        None
    }
}