//@ add-minicore
//@ needs-llvm-components: pi32v2
//@ compile-flags: --target=pi32v2-unknown-none --crate-type=rlib
//@ ignore-backends: gcc
#![no_core]
#![feature(no_core, lang_items)]

extern crate minicore;
use minicore::*;

extern "pi32v2-interrupt" fn f() {}
//~^ ERROR "pi32v2-interrupt" ABI is experimental

type TA = extern "pi32v2-interrupt" fn();
//~^ ERROR "pi32v2-interrupt" ABI is experimental

extern "pi32v2-interrupt" {}
//~^ ERROR "pi32v2-interrupt" ABI is experimental
