# `pi32v2-unknown-none`

**Tier: 3**

JieLi's pi32v2, the CPU of the AC79 chips (such as the AC791N in M-VAVE's FM-1 and SMK-37), bare metal.

## Target maintainers

[@jvitkauskas](https://github.com/jvitkauskas)

## Requirements

The code generator is the PI32V2 target of the [pi32v2-llvm](https://github.com/jvitkauskas/pi32v2-llvm) fork of
LLVM, which upstream LLVM doesn't have: build rustc against that LLVM (`llvm-config` in `bootstrap.toml`), not against
its own `src/llvm-project`.

- `no_std` only: `core` (and `alloc` with an allocator), built with `-Zbuild-std=core`.
- The CPU is `r3` (the instruction set with floating point, revision 1), as C code for these chips is usually built
  (`clang -target pi32v2 -mcpu=r3`). `-C target-cpu=ac791n` also leaves out the instruction forms that chip's hardware
  doesn't do as documented.
- The C ABI is clang's for the target: arguments in `r0`-`r3`, then the stack, aggregates by reference to a copy,
  results through a hidden pointer, and `r4`-`r15` and `rets` preserved.
- Atomics: loads and stores of up to 32 bits, and fences (`csync`). Read-modify-write atomics are calls to `__sync_*`
  functions in this backend, which nothing provides, so compare-and-swap isn't offered (`atomic_cas: false`).
- Panics: a trap is `btbclr`, which the AC791N takes as an illegal instruction (its CPU exception, with `EMU_MSG` 2),
  followed by a branch to itself. `-C panic=immediate-abort` needs no `abort` function.
- Linking: `ld.lld` from the pi32v2 LLVM toolchain, with a linker script of your own.

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
RUSTC_BOOTSTRAP=1 cargo build --release --target pi32v2-unknown-none -Zbuild-std=core
```

Inline assembly (`asm!`, with `#![feature(asm_experimental_arch)]`) has the register class `reg` (`r0`-`r15`, but not
`r7`, the frame base) and the clobber-only `sreg` (`rets`); `clobber_abi("C")` clobbers `r0`-`r3` and `rets`, and
the flags in `psr` are clobbered unless `preserves_flags`. Interrupt handlers are `extern "pi32v2-interrupt" fn()`
(`#![feature(abi_pi32v2_interrupt)]`).

## Testing

There's no test runner for the target. Programs run on the hardware, for example from RAM with M-Flasher's
`run-ram`, as [m-apps](https://github.com/jvitkauskas/m-apps) does.

## Cross-compilation toolchains and C code

C code for the target is built with the same LLVM's clang, `clang -target pi32v2 -mcpu=r3`, and can be linked with
Rust, including with cross-language LTO (`-C linker-plugin-lto`; rustc emits ThinLTO bitcode).
