# `pi32v2-unknown-none`

**Tier: 3**

JieLi's pi32v2, the CPU of the AC79 chips (such as the AC791N in M-VAVE's FM-1 and SMK-37), bare metal.

## Target maintainers

[@jvitkauskas](https://github.com/jvitkauskas)

## Requirements

The code generator is the PI32V2 target of the [pi32v2-llvm](https://github.com/jvitkauskas/pi32v2-llvm) fork of
LLVM, which upstream LLVM doesn't have: build rustc against that LLVM (`llvm-config` in `bootstrap.toml`), not against
its own `src/llvm-project`. Its `pi32v2` branch is LLVM 23.1.6; what this page says about traps, `__int128`, the
`ac791n` warnings and reassembling `--emit=asm` output needs release `pi32v2-23.1.6-1` or later.

- `no_std` only: `core` (and `alloc` with an allocator), built with `-Zbuild-std=core`.
- The CPU is `r3` (the instruction set with floating point, revision 1), as C code for these chips is usually built
  (`clang -target pi32v2 -mcpu=r3`). `-C target-cpu=ac791n` generates the same code, and makes the assembler warn
  about the instructions that chip's hardware runs differently, in `asm!` and `global_asm!` too ("popcnt.rr does
  nothing on the AC791N").
- The C ABI is clang's for the target: arguments in `r0`-`r3`, then the stack, aggregates (empty structs too) by
  reference to a copy, aggregate results through a hidden pointer, and `r4`-`r15` and `rets` preserved. `i128` and
  `f128` go in words, as LLVM passes them to its own runtime calls (which compiler-builtins defines), not as clang's
  `_BitInt(128)`, which goes by reference. The C type that matches `i128` and `u128` is `__int128`, which clang has
  on this target only with `-fforce-enable-int128`: the same size, alignment (a word) and passing. (Their results
  come back through a pointer in `r0`, as the backend returns anything wider than two words, with the arguments
  after it.) SIMD vectors
  can't be passed to or returned from C-ABI functions: rustc refuses them, as the target has no vector ABI yet. clang
  returns a zero-sized struct through a hidden pointer, which rustc can't do yet, so a C-ABI function returning one
  (defined, imported or through a function pointer) is a compile error rather than a mismatch, unless it is a naked
  function; a type with no values that is an enum (`enum Never {}`) is C's noreturn `void`, but a struct with such a
  field is still a struct. One corner remains: a `#[repr(C)]` enum whose variants all hold zero-sized data is a bare
  tag to rustc, and its C equivalent (a struct of the tag and a union of empty members) goes by reference.
- Atomics: up to 32 bits. Loads, stores and fences (`csync`) are inline. Read-modify-write operations and
  compare-and-swap are calls to `__sync_*` functions, which the pi32v2 LLVM toolchain's runtime library defines
  (`lib/clang/23/lib/pi32v2/libclang_rt.builtins.a`; they mask interrupts for the operation). A program that uses
  them adds that library to its link; one that only loads and stores needs nothing. There are no 64-bit atomics.
- Panics: a trap is `btbclr`, which the AC791N takes as an illegal instruction (its CPU exception, with `EMU_MSG` 2),
  followed by a branch to itself. `-C panic=immediate-abort` needs no `abort` function.
- Linking: `ld.lld` from the pi32v2 LLVM toolchain, with a linker script of your own. rustc has no object-file
  format for the target's crate metadata, so an `rlib` carries it as raw archive members, which `ld.lld` warns
  about (an error with `--fatal-warnings`); a `staticlib` doesn't.

## Building the target

```toml
# bootstrap.toml
[target.x86_64-unknown-linux-gnu]
llvm-config = "/path/to/pi32v2-llvm/bin/llvm-config"
```

```sh
./x.py build --stage 1 library
```

## Building Rust programs

```sh
RUSTC=/path/to/rust/build/host/stage1/bin/rustc RUSTC_BOOTSTRAP=1 \
  cargo build --release --target pi32v2-unknown-none -Zbuild-std=core
```

Inline assembly (`asm!`, with `#![feature(asm_experimental_arch)]`) has the register class `reg` (`r0`-`r15`; `r7`
can't be named, as it is the frame base in a function with a frame pointer, but in one without, LLVM may give it to a
`reg` operand like any other) and the clobber-only `sreg` (`rets`); `clobber_abi("C")` clobbers `r0`-`r3` and `rets`, and
the flags in `psr` are clobbered unless `preserves_flags`. Interrupt handlers are `extern "pi32v2-interrupt" fn()`
(`#![feature(abi_pi32v2_interrupt)]`).

## Testing

There's no test runner for the target. Programs run on the hardware, for example from RAM with M-Flasher's
`run-ram`, as [m-apps](https://github.com/jvitkauskas/m-apps) does.

## Cross-compilation toolchains and C code

C code for the target is built with the same LLVM's clang, `clang -target pi32v2 -mcpu=r3`, and can be linked with
Rust, including with cross-language LTO (`-C linker-plugin-lto`; rustc emits ThinLTO bitcode).
