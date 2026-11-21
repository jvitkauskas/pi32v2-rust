//@ add-minicore
//@ compile-flags: --target pi32v2-unknown-none
//@ needs-llvm-components: pi32v2

// The target has compare-and-swap up to a word: read-modify-write atomics are calls to the
// __sync_* functions of the pi32v2 LLVM toolchain's runtime library. There are no 64-bit atomics.

#![crate_type = "lib"]
#![feature(no_core)]
#![no_core]

extern crate minicore;
use minicore::*;

// CHECK: define{{.*}}i32 @cas_8()
#[cfg(target_has_atomic = "8")]
#[no_mangle]
pub fn cas_8() -> u32 {
    8
}

// CHECK: define{{.*}}i32 @cas_16()
#[cfg(target_has_atomic = "16")]
#[no_mangle]
pub fn cas_16() -> u32 {
    16
}

// CHECK: define{{.*}}i32 @cas_32()
#[cfg(target_has_atomic = "32")]
#[no_mangle]
pub fn cas_32() -> u32 {
    32
}

// CHECK: define{{.*}}i32 @cas_ptr()
#[cfg(target_has_atomic = "ptr")]
#[no_mangle]
pub fn cas_ptr() -> u32 {
    1
}

// CHECK-NOT: @cas_64
#[cfg(target_has_atomic = "64")]
#[no_mangle]
pub fn cas_64() -> u32 {
    64
}
