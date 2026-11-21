//! Inline assembly for JieLi's pi32v2 (the AC79's CPU), as the pi32v2-llvm fork's backend takes it: sixteen 32-bit
//! general registers r0-r15, with sp, pc, rets, psr and the other special registers apart from them.
//! The register roles are pi32v2-isa's docs/ABI.md and the backend's PI32V2RegisterInfo.

use std::fmt;

use rustc_span::Symbol;

use super::{InlineAsmArch, InlineAsmType, ModifierInfo};

def_reg_class! {
    Pi32v2 Pi32v2InlineAsmRegClass {
        reg,
        // rets, the return address a call writes: clobber-only, so that asm that calls (and clobber_abi("C")) makes
        // the function save its own.
        sreg,
    }
}

impl Pi32v2InlineAsmRegClass {
    pub fn valid_modifiers(self, _arch: super::InlineAsmArch) -> &'static [char] {
        &[]
    }

    pub fn suggest_class(self, _arch: InlineAsmArch, _ty: InlineAsmType) -> Option<Self> {
        None
    }

    pub fn suggest_modifier(
        self,
        _arch: InlineAsmArch,
        _ty: InlineAsmType,
    ) -> Option<ModifierInfo> {
        None
    }

    pub fn default_modifier(self, _arch: InlineAsmArch) -> Option<ModifierInfo> {
        None
    }

    pub fn supported_types(
        self,
        _arch: InlineAsmArch,
    ) -> &'static [(InlineAsmType, Option<Symbol>)] {
        match self {
            // 64-bit values take a register pair in LLVM ("r1_r0"), which named registers can't say; not offered.
            Self::reg => types! { _: I8, I16, I32, F32; },
            Self::sreg => &[],
        }
    }
}

def_regs! {
    Pi32v2 Pi32v2InlineAsmReg Pi32v2InlineAsmRegClass {
        r0: reg = ["r0"],
        r1: reg = ["r1"],
        r2: reg = ["r2"],
        r3: reg = ["r3"],
        r4: reg = ["r4"],
        r5: reg = ["r5"],
        r6: reg = ["r6"],
        r8: reg = ["r8"],
        r9: reg = ["r9"],
        r10: reg = ["r10"],
        r11: reg = ["r11"],
        r12: reg = ["r12"],
        r13: reg = ["r13"],
        r14: reg = ["r14"],
        r15: reg = ["r15"],
        rets: sreg = ["rets"],

        #error = ["r7"] =>
            "r7 is used internally by LLVM as the frame base and cannot be used as an operand for inline asm",
        #error = ["sp"] =>
            "the stack pointer cannot be used as an operand for inline asm",
        #error = ["pc"] =>
            "the program counter cannot be used as an operand for inline asm",
        #error = ["psr"] =>
            "the status register cannot be used as an operand for inline asm; use `preserves_flags` to keep its flags",
        #error = ["reti", "rete", "retx", "ssp", "usp", "icfg", "cnum"] =>
            "special registers cannot be used as operands for inline asm; read or write them in the asm itself",
    }
}

impl Pi32v2InlineAsmReg {
    pub fn emit(
        self,
        out: &mut dyn fmt::Write,
        _arch: InlineAsmArch,
        _modifier: Option<char>,
    ) -> fmt::Result {
        out.write_str(self.name())
    }
}
