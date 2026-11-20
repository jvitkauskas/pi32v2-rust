//@ add-minicore
//@ compile-flags: --target pi32v2-unknown-none -Copt-level=1
//@ needs-llvm-components: pi32v2

// An integer wider than 64 bits goes as clang's _BitInt(128) does: by reference to a copy (clang's
// byval, one pointer word in the backend), and returned through a hidden pointer.

#![crate_type = "rlib"]
#![feature(no_core)]
#![no_core]

extern crate minicore;
use minicore::*;

unsafe extern "C" {
    fn echo_wide(x: i128, tag: i32) -> i128;
    fn narrow(x: i64, tag: i32) -> i64;
}

// CHECK-LABEL: @call_wide
// CHECK: call void @echo_wide(ptr {{.*}}sret{{.*}}, ptr {{.*}}, i32 {{.*}}2)
#[no_mangle]
pub unsafe fn call_wide() -> i128 {
    echo_wide(1, 2)
}

// CHECK-LABEL: @call_narrow
// CHECK: call {{.*}}i64 @narrow(i64 {{.*}}1, i32 {{.*}}2)
#[no_mangle]
pub unsafe fn call_narrow() -> i64 {
    narrow(1, 2)
}
