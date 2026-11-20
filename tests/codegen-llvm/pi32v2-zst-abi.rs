//@ add-minicore
//@ compile-flags: --target pi32v2-unknown-none -Copt-level=1
//@ needs-llvm-components: pi32v2

// clang passes an empty struct as any aggregate, by reference to a copy: it takes an argument
// slot, among variadic arguments too. (It also returns one through a hidden pointer, which Rust
// doesn't do yet.) `()` stays C's void.

#![crate_type = "rlib"]
#![feature(no_core)]
#![no_core]

extern crate minicore;
use minicore::*;

#[repr(C)]
pub struct Empty {}

unsafe extern "C" {
    fn take(a: i32, e: Empty, b: i32) -> i32;
    fn cons(n: i32, ...) -> i32;
}

// CHECK-LABEL: @call_take
// CHECK: call {{.*}}i32 @take(i32 {{.*}}1, ptr {{.*}}, i32 {{.*}}42)
#[no_mangle]
pub unsafe fn call_take() -> i32 {
    take(1, Empty {}, 42)
}

// CHECK-LABEL: @call_cons
// CHECK: call {{.*}}i32 (i32, ...) @cons(i32 {{.*}}1, ptr {{.*}}, i32 {{.*}}42)
#[no_mangle]
pub unsafe fn call_cons() -> i32 {
    cons(1, Empty {}, 42)
}

// CHECK: define {{.*}}void @unit_ret()
#[no_mangle]
pub extern "C" fn unit_ret() {}
