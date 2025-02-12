#![no_main]
sp1_zkvm::entrypoint!(main);

use sp1_zkvm::syscalls::syscall_add_mul;
pub fn main() {
    let mut a: u32 = 1;
    let b: u32 = 2;
    let c: u32 = 4;
    let d: u32 = 5;
    println!("a: {}", a);
    for i in 0..100000 {
        // need to reset a, otherwise a will be exponentially growing and cuz overflow and 
        // fail the verification
        a = 1;
        a = a*b + c*d
        //let result = 22;
    }
    // syscall_add_mul(
    //     &mut a,
    //     &b,
    //     &c,
    //     &d,
    // );
    sp1_zkvm::io::commit(&a);
}