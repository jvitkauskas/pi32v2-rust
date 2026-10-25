# pi32v2-rust

Rust 1.99.1 with a proof-of-concept port to **pi32v2**, the CPU core of JieLi's AC79 chips (the AC791N in the
M-VAVE FM-1 and SMK-37 Pro). Unofficial; not affiliated with the Rust project, JieLi or M-VAVE.

This branch (`pi32v2`) is the release tag `1.99.1` with one commit on top:

- `compiler/rustc_llvm`: pi32v2 among the LLVM targets rustc initializes;
- `compiler/rustc_target/src/callconv/pi32v2.rs`: the C calling convention for `"arch": "pi32v2"` JSON target
  specifications, as clang lowers it for the target (aggregates by reference to a copy, results through a hidden
  pointer, scalars directly with integers widened to 32 bits), and its dispatch in `callconv/mod.rs`.

The code generator is LLVM's PI32V2 target from [jvitkauskas/pi32v2-llvm](https://github.com/jvitkauskas/pi32v2-llvm)
(LLVM 23.1.4, as Rust 1.99 uses): rustc is built against that LLVM (`bootstrap.pi32v2.toml`), not against its own
`src/llvm-project`, which this build does not need.

```sh
cp bootstrap.pi32v2.toml bootstrap.toml     # set llvm-config in it
./x.py build --stage 1 library              # build/host/stage1/bin/rustc
```

`no_std` code is built with any recent cargo and `-Zbuild-std=core` for a JSON target. A complete example, run on
an FM-1 (an interrupt-driven blink and a picture on the screen): `examples/fm1-blink-rust` in
[jvitkauskas/pi32v2-isa](https://github.com/jvitkauskas/pi32v2-isa/tree/main/examples/fm1-blink-rust), with the
target specification and the build. Not there yet: inline `asm!` (rustc supports it per architecture) and an
interrupt ABI.

Licence: Rust's, MIT OR Apache-2.0 ([COPYRIGHT](COPYRIGHT), [LICENSE-MIT](LICENSE-MIT),
[LICENSE-APACHE](LICENSE-APACHE)).
