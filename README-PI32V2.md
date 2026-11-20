# pi32v2-rust

Rust 1.100.0 with a proof-of-concept port to **pi32v2**, the CPU core of JieLi's AC79 chips (the AC791N in the
M-VAVE FM-1 and SMK-37 Pro). Unofficial; not affiliated with the Rust project, JieLi or M-VAVE.

This branch is the release tag `1.100.0` with the pi32v2 port on top:

- `compiler/rustc_llvm`: pi32v2 among the LLVM targets rustc initializes;
- `compiler/rustc_target/src/callconv/pi32v2.rs`: the C calling convention, as clang lowers it for the target
  (aggregates by reference to a copy, results through a hidden pointer, scalars directly with integers widened to
  32 bits);
- a built-in target, **`pi32v2-unknown-none`** (tier 3, `no_std`; `src/doc/rustc/src/platform-support/pi32v2-unknown-none.md`),
  and `pi32v2` as a `target_arch` of its own: no JSON target specification or `-Zjson-target-spec` is needed;
- **`asm!`** (`#![feature(asm_experimental_arch)]`): the register class `reg` (`r0`-`r15`; `r7`, the frame base, can't
  be named),
  the clobber-only `sreg` (`rets`), `clobber_abi("C")` (`r0`-`r3` and `rets`), and the flags in `psr` clobbered
  unless `preserves_flags`;
- an interrupt ABI, **`extern "pi32v2-interrupt" fn()`** (`#![feature(abi_pi32v2_interrupt)]`): the backend's
  `"interrupt"` attribute, as clang's `__attribute__((interrupt))`;
- C-variadic `va_arg` as clang lowers it (word slots, no alignment above a word; `tests/assembly-llvm/c-variadic/pi32v2.rs`
  compares it with clang's code), for defining one with `#![feature(c_variadic_experimental_arch)]`;
- empty struct arguments passed as clang passes them (by reference to a copy), not left out, and `i128` as clang's
  `_BitInt(128)`; a C-ABI function returning a zero-sized struct, which clang returns through a hidden pointer and
  rustc can't yet, is a compile error.

The code generator is LLVM's PI32V2 target from [jvitkauskas/pi32v2-llvm](https://github.com/jvitkauskas/pi32v2-llvm)
(LLVM 23.1.5, as Rust 1.100 uses): rustc is built against that LLVM (`bootstrap.pi32v2.toml`), not against its own
`src/llvm-project`, which this build does not need. With that LLVM's commit "Traps: btbclr and a goto to itself", a
panic with `-Cpanic=immediate-abort` is a trap instruction rather than a call to `abort()`. With cross-language LTO
(`-Clinker-plugin-lto`), the linker's LLVM generates the code, so it needs that commit too.

```sh
cp bootstrap.pi32v2.toml bootstrap.toml     # set llvm-config in it
./x.py build --stage 1 library              # build/host/stage1/bin/rustc
```

`no_std` code is built with any recent cargo:

```sh
RUSTC_BOOTSTRAP=1 RUSTC=build/host/stage1/bin/rustc cargo build --release --target pi32v2-unknown-none -Zbuild-std=core
```

Tested on an FM-1 (from RAM, through [m-apps](https://github.com/jvitkauskas/m-apps)): a Rust TIMER5 handler,
`extern "pi32v2-interrupt"`, at 10 kHz over a computation that came out the same with and without it; `icfg` and
`csync` through `asm!`; and a bounds-check failure ending in the CPU exception. Tests:
`tests/assembly-llvm/asm/pi32v2-types.rs`, `tests/codegen-llvm/asm/pi32v2-clobbers.rs`,
`tests/codegen-llvm/pi32v2-interrupt.rs`, `tests/assembly-llvm/pi32v2-interrupt-trap.rs`, `tests/ui/asm/pi32v2/`,
`tests/ui/feature-gates/feature-gate-abi-pi32v2-interrupt.rs`.

Not there yet: atomic read-modify-write (the backend makes them `__sync_*` calls, so the target has no CAS), `i64`
operands in `asm!`, returning zero-sized structs through the C ABI, crate metadata in an object file (an `rlib`'s
raw metadata members make `ld.lld` warn), and `std`.

Licence: Rust's, MIT OR Apache-2.0 ([COPYRIGHT](COPYRIGHT), [LICENSE-MIT](LICENSE-MIT),
[LICENSE-APACHE](LICENSE-APACHE)).
