//@ add-minicore
//@ assembly-output: emit-asm
//@ compile-flags: --target pi32v2-unknown-none
//@ needs-llvm-components: pi32v2

#![feature(no_core, asm_experimental_arch)]
#![crate_type = "rlib"]
#![no_core]

extern crate minicore;
use minicore::*;

macro_rules! check {
    ($func:ident $ty:ident $class:ident) => {
        #[no_mangle]
        pub unsafe fn $func(x: $ty) -> $ty {
            let y;
            asm!("{} = {}", lateout($class) y, in($class) x);
            y
        }
    };
}

macro_rules! check_reg {
    ($func:ident $ty:ident $reg:tt) => {
        #[no_mangle]
        pub unsafe fn $func(x: $ty) -> $ty {
            let y;
            asm!(concat!($reg, " = ", $reg), lateout($reg) y, in($reg) x);
            y
        }
    };
}

// CHECK-LABEL: reg_i8:
// CHECK: ;APP
// CHECK: r{{[0-9]+}} = r{{[0-9]+}}
// CHECK: ;NO_APP
check!(reg_i8 i8 reg);

// CHECK-LABEL: reg_i16:
// CHECK: ;APP
// CHECK: r{{[0-9]+}} = r{{[0-9]+}}
// CHECK: ;NO_APP
check!(reg_i16 i16 reg);

// CHECK-LABEL: reg_i32:
// CHECK: ;APP
// CHECK: r{{[0-9]+}} = r{{[0-9]+}}
// CHECK: ;NO_APP
check!(reg_i32 i32 reg);

// CHECK-LABEL: reg_f32:
// CHECK: ;APP
// CHECK: r{{[0-9]+}} = r{{[0-9]+}}
// CHECK: ;NO_APP
check!(reg_f32 f32 reg);

// CHECK-LABEL: r0_i32:
// CHECK: ;APP
// CHECK: r0 = r0
// CHECK: ;NO_APP
check_reg!(r0_i32 i32 "r0");

// CHECK-LABEL: r5_i32:
// CHECK: ;APP
// CHECK: r5 = r5
// CHECK: ;NO_APP
check_reg!(r5_i32 i32 "r5");

// CHECK-LABEL: r15_i8:
// CHECK: ;APP
// CHECK: r15 = r15
// CHECK: ;NO_APP
check_reg!(r15_i8 i8 "r15");
