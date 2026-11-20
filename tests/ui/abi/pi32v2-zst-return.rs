//@ add-minicore
//@ build-fail
//@ compile-flags: --target pi32v2-unknown-none --crate-type=rlib
//@ needs-llvm-components: pi32v2
//@ ignore-backends: gcc

// clang (target pi32v2) returns a zero-sized struct through a hidden pointer, as any aggregate,
// and rustc can't: a C-ABI function returning one is refused, at its definition and at calls,
// rather than called with every argument one register off.

#![feature(no_core, asm_experimental_arch)]
#![no_core]

extern crate minicore;
use minicore::*;

#[repr(C)]
pub struct Z {
    z: [u8; 0],
}

#[no_mangle]
pub extern "C" fn define(a: u32) -> Z {
    //~^ ERROR this function definition returns the zero-sized type `Z`
    let _ = a;
    Z { z: [] }
}

unsafe extern "C" {
    fn import(a: u32) -> Z;
}

#[no_mangle]
pub unsafe fn call_import() {
    import(1);
    //~^ ERROR this function call returns the zero-sized type `Z`
}

#[no_mangle]
pub unsafe fn call_pointer(f: extern "C" fn(u32) -> Z) {
    f(1);
    //~^ ERROR this function call returns the zero-sized type `Z`
}

// `()` is C's void, and stays allowed.
#[no_mangle]
pub extern "C" fn unit(a: u32) {
    let _ = a;
}

// A naked function's assembly can follow the C convention itself (the hidden pointer in r0).
#[unsafe(naked)]
#[no_mangle]
pub extern "C" fn naked(a: u32) -> Z {
    naked_asm!("rts")
}

// An uninhabited return type is C's noreturn void, not a struct.
pub enum Never {}

unsafe extern "C" {
    fn stop(code: u32) -> Never;
}

#[no_mangle]
pub unsafe fn call_stop() -> Never {
    stop(1)
}
