//@ add-minicore
//@ assembly-output: emit-asm
//@ compile-flags: --target pi32v2-unknown-none
//@ needs-llvm-components: pi32v2

#![crate_type = "rlib"]
#![feature(no_core, asm_experimental_arch)]
#![no_core]

extern crate minicore;
use minicore::*;

// CHECK-LABEL: @psr_clobber
// CHECK: call void asm sideeffect "", "~{psr}"()
#[no_mangle]
pub unsafe fn psr_clobber() {
    asm!("", options(nostack, nomem));
}

// CHECK-LABEL: @no_clobber
// CHECK: call void asm sideeffect "", ""()
#[no_mangle]
pub unsafe fn no_clobber() {
    asm!("", options(nostack, nomem, preserves_flags));
}

// A call in asm writes rets, so clobber_abi("C") clobbers it too, and the function saves its own.
// CHECK-LABEL: @clobber_abi
// CHECK: asm sideeffect "", "={r0},={r1},={r2},={r3},~{rets}"()
#[no_mangle]
pub unsafe fn clobber_abi() {
    asm!("", clobber_abi("C"), options(nostack, nomem, preserves_flags));
}

// CHECK-LABEL: @rets_clobber
// CHECK: asm sideeffect "", "~{rets}"()
#[no_mangle]
pub unsafe fn rets_clobber() {
    asm!("", out("rets") _, options(nostack, nomem, preserves_flags));
}
