//@ add-minicore
//@ assembly-output: emit-asm
//
//@ revisions: PI32V2
//@ [PI32V2] compile-flags: -Copt-level=3 --target=pi32v2-unknown-none
//@ [PI32V2] needs-llvm-components: pi32v2
#![feature(c_variadic_experimental_arch, no_core, lang_items, intrinsics, rustc_attrs)]
#![no_core]
#![crate_type = "lib"]

// Check that rustc and clang output match: va_list points at the next word, and nothing is aligned
// higher than a word (clang's PI32V2ABIInfo::EmitVAArg), so an i64 or f64 split between the saved
// r3 and the stack is read from the word it starts at.

extern crate minicore;
use minicore::ffi::VaList;
use minicore::*;

#[unsafe(no_mangle)]
unsafe extern "C" fn read_i32(ap: &mut VaList<'_>) -> i32 {
    // CHECK-LABEL: read_i32:
    //
    // PI32V2: r1 = [r0+0]
    // PI32V2-NEXT: r2 = r1 + 4
    // PI32V2-NEXT: [r0+0] = r2
    // PI32V2-NEXT: r0 = [r1+0]
    // PI32V2-NEXT: rts
    ap.next_arg()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn read_i64(ap: &mut VaList<'_>) -> i64 {
    // CHECK-LABEL: read_i64:
    //
    // PI32V2: r1 = [r0+0]
    // PI32V2-NEXT: r2 = r1 + 8
    // PI32V2-NEXT: [r0+0] = r2
    // PI32V2-NEXT: r0 = [r1+0]
    // PI32V2-NEXT: r1 = [r1+4]
    // PI32V2-NEXT: rts
    ap.next_arg()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn read_f64(ap: &mut VaList<'_>) -> f64 {
    // CHECK-LABEL: read_f64:
    //
    // PI32V2: r1 = [r0+0]
    // PI32V2-NEXT: r2 = r1 + 8
    // PI32V2-NEXT: [r0+0] = r2
    // PI32V2-NEXT: r0 = [r1+0]
    // PI32V2-NEXT: r1 = [r1+4]
    // PI32V2-NEXT: rts
    ap.next_arg()
}
