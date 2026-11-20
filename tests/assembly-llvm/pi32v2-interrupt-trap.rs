//@ add-minicore
//@ assembly-output: emit-asm
//@ compile-flags: --target pi32v2-unknown-none -Copt-level=2
//@ needs-llvm-components: pi32v2

// An interrupt handler returns with rti and saves psr and reti; a trap is btbclr (an illegal
// instruction on the AC791N, so the CPU exception) and a goto to itself, not a call to abort().

#![crate_type = "rlib"]
#![feature(no_core, abi_pi32v2_interrupt, asm_experimental_arch, intrinsics, rustc_attrs)]
#![no_core]

extern crate minicore;
use minicore::*;

#[rustc_intrinsic]
pub fn abort() -> !;

unsafe extern "C" {
    fn tick();
}

// CHECK-LABEL: timer_isr:
// CHECK: [--sp] = {psr, reti}
// CHECK-NEXT: [--sp] = {rets, r3-r0}
// CHECK-NEXT: call tick
// CHECK-NEXT: {rets, r3-r0} = [sp++]
// CHECK-NEXT: {psr, reti} = [sp++]
// CHECK-NEXT: rti
#[no_mangle]
pub extern "pi32v2-interrupt" fn timer_isr() {
    unsafe { tick() }
}

// CHECK-LABEL: stop:
// CHECK-NOT: call abort
// CHECK: btbclr
// CHECK-NEXT: [[L:.Ltmp[0-9]+]]:
// CHECK-NEXT: goto [[L]]
#[no_mangle]
pub fn stop() -> ! {
    abort()
}

// A call in asm writes rets: with clobber_abi("C"), a function with no other call saves its own.
// CHECK-LABEL: asm_call:
// CHECK: [--sp] = rets
// CHECK: call ext
// CHECK: pc = [sp++]
#[no_mangle]
pub unsafe fn asm_call() {
    asm!("call ext", clobber_abi("C"));
}
