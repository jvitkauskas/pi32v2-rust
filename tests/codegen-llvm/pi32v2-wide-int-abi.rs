//@ add-minicore
//@ compile-flags: --target pi32v2-unknown-none -Copt-level=1
//@ needs-llvm-components: pi32v2

// i128 is passed in words, as LLVM lowers it and passes it to its own runtime calls: a u128
// division is a call to __udivti3, which compiler-builtins defines as an extern "C" function, so
// that definition must take its operands by value, not by reference. (clang's _BitInt(128) goes
// by reference instead; its __int128, which it has with -fforce-enable-int128, goes as i128 does.)

#![crate_type = "rlib"]
#![feature(no_core)]
#![no_core]
#![allow(improper_ctypes_definitions)]

extern crate minicore;
use minicore::*;

// CHECK: define {{.*}}i128 @__udivti3(i128 {{.*}}%x, i128 {{.*}}%_y)
#[no_mangle]
pub extern "C" fn __udivti3(x: u128, _y: u128) -> u128 {
    x
}

unsafe extern "C" {
    fn echo_wide(x: i128, tag: i32) -> i128;
}

// CHECK-LABEL: @call_wide
// CHECK: call {{.*}}i128 @echo_wide(i128 {{.*}}1, i32 {{.*}}2)
#[no_mangle]
pub unsafe fn call_wide() -> i128 {
    echo_wide(1, 2)
}
