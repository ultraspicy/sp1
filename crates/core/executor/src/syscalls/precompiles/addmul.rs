use num::{BigUint, One, Zero};

use sp1_curves::edwards::WORDS_FIELD_ELEMENT;
use sp1_primitives::consts::{bytes_to_words_le, words_to_bytes_le_vec, WORD_SIZE};

use crate::{
    events::{AddMulEvent, PrecompileEvent},
    syscalls::{Syscall, SyscallCode, SyscallContext},
};

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

        // arg1 points to a memory location that contains our 4 pointers
        let ptr_base = arg1;

        if ptr_base % 4 != 0 {
            panic!();
        }

        // Read the 4 pointers from memory
        let (_, ptrs) = rt.mr_slice(ptr_base, 4); // Read 4 u32 words containing our pointers
        let a = ptrs[0];
        let b = ptrs[1];
        let c = ptrs[2];
        let d = ptrs[3];

        // Perform computations, a b c d is within range of [0, 255] so a*b + c*d won't exceed the range
        let modulus: u32 = 1 << 32; // Use 2^32 as modulus

        // First multiplication: a * b
        let mul1_result = a * b;

        // Second multiplication: c * d
        let mul2_result = c * d;

        // Final addition: (a*b) + (c*d)
        let result = (mul1_result + mul2_result) % modulus;

        // Convert result to bytes and pad
        let mut result_bytes: [u8; 4] = result.to_le_bytes();

        result_bytes.resize(32, 0u8); // Pad to 32 bytes
        let result_words = bytes_to_words_le::<8>(&result_bytes);

        // Increment clock and write result back to first pointer (a_ptr)
        rt.clk += 1;
        let result_memory_records = rt.mw_slice(a_ptr, &result_words);

        // Record event
        let lookup_id = rt.syscall_lookup_id;
        let shard = rt.current_shard();
        let event = PrecompileEvent::ADDMul(AddMulEvent {
            lookup_id,
            shard,
            clk,
            a,
            b,
            c,
            d,
            a_memory_records,
            b_memory_records,
            c_memory_records,
            d_memory_records,
            result_memory_records,
            local_mem_access: rt.postprocess(),
        });

        let syscall_event =
            rt.rt.syscall_event(clk, syscall_code.syscall_id(), arg1, arg2, lookup_id);
        rt.add_precompile_event(syscall_code, syscall_event, event);

        None
    }
}
