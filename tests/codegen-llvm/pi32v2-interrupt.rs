//@ add-minicore
//@ compile-flags: --target pi32v2-unknown-none -Copt-level=1
//@ needs-llvm-components: pi32v2

// `extern "pi32v2-interrupt"` is the C convention with the "interrupt" function attribute, which
// the backend takes as clang's __attribute__((interrupt)): psr and reti saved, every register used
// saved, and rti to return.

#![crate_type = "rlib"]
#![feature(no_core, abi_pi32v2_interrupt)]
#![no_core]

extern crate minicore;
use minicore::*;

// CHECK: define {{.*}}void @timer_isr() unnamed_addr #[[ATTRS:[0-9]+]]
#[no_mangle]
pub extern "pi32v2-interrupt" fn timer_isr() {}

// CHECK: attributes #[[ATTRS]] = {{.*}}"interrupt"
