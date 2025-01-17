#[cfg(target_os = "zkvm")]
use core::arch::asm;
/// The result is written over the first input.
#[allow(unused_variables)]
#[no_mangle]

pub extern "C" fn syscall_add_mul(p: *mut u32, q: *const u32, r: *const u32, s: *const u32) {
    #[cfg(target_os = "zkvm")]
    unsafe {
        let r_val = *r;  // Get value 4
        let s_val = *s;  // Get value 5
        
        asm!(
            "mv x12, {0}",  // Move value 4 to x12 
            "mv x13, {1}",  // Move value 5 to x13
            "ecall",
            in(reg) r_val,  // Value 4
            in(reg) s_val,  // Value 5
            in("t0") crate::syscalls::ADD_MUL,
            in("a0") p,
            in("a1") q,
        );
    }
    #[cfg(not(target_os = "zkvm"))]
    unreachable!()
}