//@ add-minicore
//@ assembly-output: emit-asm
//@ compile-flags: --target pi32v2-unknown-none -Copt-level=2
//@ needs-llvm-components: pi32v2

// A u128 division is a call LLVM makes to __udivti3, which compiler-builtins defines as an
// extern "C" function: both sides must agree in the machine code. The result comes back through a
// pointer in r0, and the dividend's words follow it, by value, in r1-r3 and then on the stack.

#![crate_type = "rlib"]
#![feature(no_core, intrinsics, rustc_attrs)]
#![no_core]
#![allow(improper_ctypes_definitions)]

extern crate minicore;
use minicore::*;

#[rustc_intrinsic]
pub unsafe fn unchecked_div<T: Copy>(x: T, y: T) -> T;

// CHECK-LABEL: divide:
// CHECK: r0 = sp + {{[0-9]+}}
// CHECK-NEXT: call __udivti3
#[no_mangle]
pub unsafe fn divide(x: u128, y: u128) -> u128 {
    unchecked_div(x, y)
}

// The definition takes x's words as values (not through r1) and writes the result through r0.
// CHECK-LABEL: __udivti3:
// CHECK-NOT: = [r1+
// CHECK-DAG: [r0+0] = r1
// CHECK-DAG: [r0+4] = r2
// CHECK-DAG: [r0+8] = r3
// CHECK: rts
#[no_mangle]
pub extern "C" fn __udivti3(x: u128, _y: u128) -> u128 {
    x
}
