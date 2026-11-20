//@ add-minicore
//@ needs-asm-support
//@ compile-flags: --target pi32v2-unknown-none
//@ needs-llvm-components: pi32v2
//@ ignore-backends: gcc

#![crate_type = "rlib"]
#![feature(no_core, asm_experimental_arch)]
#![no_core]

extern crate minicore;
use minicore::*;

fn f() {
    let mut x = 0i32;
    let y = 0i64;
    unsafe {
        asm!("", out("r7") _);
        //~^ ERROR invalid register `r7`: r7 is used internally by LLVM as the frame base
        asm!("", out("sp") _);
        //~^ ERROR invalid register `sp`: the stack pointer cannot be used as an operand for inline asm
        asm!("", out("pc") _);
        //~^ ERROR invalid register `pc`: the program counter cannot be used as an operand for inline asm
        asm!("", out("psr") _);
        //~^ ERROR invalid register `psr`: the status register cannot be used as an operand for inline asm
        asm!("", out("reti") _);
        //~^ ERROR invalid register `reti`: special registers cannot be used as operands for inline asm
        asm!("", in("rets") x);
        //~^ ERROR can only be used as a clobber
        //~| ERROR type `i32` cannot be used with this register class
        asm!("/* {} */", in(sreg) x);
        //~^ ERROR can only be used as a clobber
        //~| ERROR type `i32` cannot be used with this register class
        asm!("/* {} */", in(reg) y);
        //~^ ERROR type `i64` cannot be used with this register class
        asm!("", in("r0") x, out("r0") _);
        //~^ ERROR register `r0` conflicts with register `r0`
        asm!("", out("r1") x, clobber_abi("C"));
    }
}
