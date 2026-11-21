pi32v2 Rust @VERSION@: rustc @RUST@ with the target `pi32v2-unknown-none`, for **pi32v2**, the CPU core of JieLi's
AC79 chips (the AC791N in the M-VAVE FM-1 and SMK-37 Pro / Elite), built from this repository's `pi32v2` branch
(@COMMIT@) on the LLVM of the pi32v2 LLVM toolchain
[@LLVM@](https://github.com/jvitkauskas/pi32v2-llvm/releases/tag/@LLVM@). Unofficial; not affiliated with the
Rust project, JieLi or M-VAVE.

| Package | For |
| --- | --- |
| `pi32v2-rust-@VERSION@-linux-x64.tar.xz` | Linux x86-64 (glibc as in Ubuntu 24.04 or later) |
| `pi32v2-rust-@VERSION@-linux-arm64.tar.xz` | Linux AArch64 (glibc as in Ubuntu 24.04 or later) |
| `pi32v2-rust-@VERSION@-windows-x64.zip` | Windows x64 |
| `pi32v2-rust-@VERSION@-macos-arm64.tar.xz` | macOS 11 or later on Apple silicon |
| `pi32v2-rust-@VERSION@-macos-x64.tar.xz` | macOS 11 or later on Intel |

Each unpacks to one directory, which can live anywhere:

- `bin/rustc` and `bin/rustdoc`, with the computer's own standard library (for build scripts and macros);
- `core`, `alloc` and `compiler_builtins` built for `pi32v2-unknown-none`;
- the libraries' sources, for cargo's `-Zbuild-std`.

There is no cargo in it; any recent one works:

```sh
rustup toolchain link pi32v2 pi32v2-rust-@VERSION@-linux-x64
cargo +pi32v2 build --release --target pi32v2-unknown-none
```

or, without rustup, `RUSTC=<directory>/bin/rustc cargo build ...`. It is a nightly compiler, as the port needs
`#![feature(asm_experimental_arch)]` for `asm!`.

Linking takes `ld.lld` of the pi32v2 LLVM toolchain, @LLVM@ or a later one of the same LLVM, on `PATH` or named
with `-C linker=`. The computer's own C toolchain is needed as with any rustc. How to use the target:
[README-PI32V2.md](https://github.com/jvitkauskas/pi32v2-rust/blob/@COMMIT@/README-PI32V2.md) and its
[platform page](https://github.com/jvitkauskas/pi32v2-rust/blob/@COMMIT@/src/doc/rustc/src/platform-support/pi32v2-unknown-none.md).

macOS marks downloads from a browser as quarantined; `xattr -dr com.apple.quarantine pi32v2-rust-@VERSION@-macos-*`
lets the unsigned tools run.

Check a download with `sha256sum -c SHA256SUMS --ignore-missing` (macOS: `shasum -a 256 -c SHA256SUMS
--ignore-missing`).

Licence: Rust's, MIT OR Apache-2.0; rustc contains LLVM, which is Apache-2.0 with LLVM exceptions (the licence
files are in each package).
