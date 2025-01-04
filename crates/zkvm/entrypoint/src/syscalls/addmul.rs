#[cfg(target_os = "zkvm")]
use core::arch::asm;

/// a*b + c*d operation.
///
/// The result is written over the first input.
#[allow(unused_variables)]
#[no_mangle]
// Raw pointers don't follow Rust's normal borrowing rules
// They're always unsafe to dereference
// explicit use *const vs *mut distinction in raw pointers
// &mut i32 - References
// *mut i32 - raw pointer
pub extern "C" fn syscall_add_mul(x: *mut u32, y: *mut u32, p: *mut u32, q: *mut u32) {
    #[cfg(target_os = "zkvm")] // only compile this syscall when the target OS is zkvm
    unsafe {
        asm!(
            "ecall",
            in("t0") crate::syscalls::ADDMUL,
            in("a0") x,
            in("a1") y,
            in("a2") p,
            in("a3") q
        );
    }

    #[cfg(not(target_os = "zkvm"))] 
    unreachable!()
}
