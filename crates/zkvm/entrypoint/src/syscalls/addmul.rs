#[cfg(target_os = "zkvm")]
use core::arch::asm;
/// The result is written over the first input.
#[allow(unused_variables)]
#[no_mangle]


pub extern "C" fn syscall_add_mul(p: *mut u32, q: *const u32, r: *const u32, s: *const u32) {
    #[cfg(target_os = "zkvm")]
    unsafe {
        asm!(
            "ecall",
            in("t0") crate::syscalls::0x00_00_01_30,
            in("a0") p,
            in("a1") q,
            in("a2") r,
            in("a3") s,
        );
    }
    #[cfg(not(target_os = "zkvm"))]
    unreachable!()
}