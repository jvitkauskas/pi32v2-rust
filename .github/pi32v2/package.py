#!/usr/bin/env python3
# Build the pi32v2 Rust package on Linux, macOS or Windows: rustc and rustdoc for this computer, its standard
# library, core and alloc for pi32v2-unknown-none, and the library sources (for -Zbuild-std), as one directory
# OUT/pi32v2-rust-VERSION-PLATFORM, packed next to it as .tar.xz (.zip on Windows).
#
# x.py builds what is for this computer. It can't build the pi32v2 libraries (its C compiler detection, the cc
# crate, doesn't know the architecture, and nothing in them is C), so the packed compiler does, through cargo's
# -Zbuild-std, as a project would.
#
#   package.py --src RUST --llvm PI32V2_LLVM --build BUILD_DIR --out OUT_DIR --version VERSION
#              [--llvm-release TAG] [--toolchain DIR] [--tests] [--launcher ccache]
#
# rustc is linked against the LLVM in PI32V2_LLVM (a checkout of jvitkauskas/pi32v2-llvm), which this builds
# first, with the backends for this computer and for pi32v2. --toolchain is an unpacked pi32v2 LLVM toolchain
# release: with it, the checks also link pi32v2 programs with its ld.lld. --tests also runs the port's compiler
# tests.
import argparse
import os
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import zipfile
from pathlib import Path

TARGET = "pi32v2-unknown-none"
WINDOWS = sys.platform == "win32"
EXE = ".exe" if WINDOWS else ""
# (the Rust host, LLVM's backend for it, the package's name for it)
HOSTS = {
    ("linux", "x86_64"): ("x86_64-unknown-linux-gnu", "X86", "linux-x64"),
    ("linux", "aarch64"): ("aarch64-unknown-linux-gnu", "AArch64", "linux-arm64"),
    ("darwin", "arm64"): ("aarch64-apple-darwin", "AArch64", "macos-arm64"),
    ("darwin", "x86_64"): ("x86_64-apple-darwin", "X86", "macos-x64"),
    ("win32", "amd64"): ("x86_64-pc-windows-msvc", "X86", "windows-x64"),
}
# The port's tests (README-PI32V2.md), and the ones that list every target.
TESTS = [
    "tests/assembly-llvm/asm/pi32v2-types.rs", "tests/assembly-llvm/c-variadic/pi32v2.rs",
    "tests/assembly-llvm/pi32v2-interrupt-trap.rs", "tests/assembly-llvm/pi32v2-wide-runtime-call.rs",
    "tests/assembly-llvm/targets/targets-elf.rs", "tests/codegen-llvm/asm/pi32v2-clobbers.rs",
    "tests/codegen-llvm/pi32v2-atomic-cfg.rs", "tests/codegen-llvm/pi32v2-interrupt.rs",
    "tests/codegen-llvm/pi32v2-wide-int-abi.rs", "tests/codegen-llvm/pi32v2-zst-abi.rs",
    "tests/ui/abi/pi32v2-zst-return.rs", "tests/ui/asm/pi32v2",
    "tests/ui/feature-gates/feature-gate-abi-pi32v2-interrupt.rs", "tests/ui/check-cfg/well-known-values.rs",
    "tests/ui/print-request/print-calling-conventions.rs",
]


def run(*cmd, **kw):
    print("+", " ".join(str(c) for c in cmd), flush=True)
    return subprocess.run([str(c) for c in cmd], check=True, **kw)


def output(*cmd, **kw):
    return run(*cmd, capture_output=True, text=True, **kw).stdout


def describe(path):
    r = subprocess.run(["git", "-C", str(path), "rev-parse", "HEAD"], capture_output=True, text=True)
    return r.stdout.strip() or "unknown"


def llvm(src, build, prefix, backend, tests, launcher):
    """LLVM's libraries, headers and llvm-config, as rustc's build takes them; static, so rustc carries them."""
    args = [
        "-G", "Ninja", "-S", src / "llvm", "-B", build,
        "-DCMAKE_BUILD_TYPE=Release", "-DLLVM_ENABLE_ASSERTIONS=OFF",
        f"-DLLVM_TARGETS_TO_BUILD={backend}", "-DLLVM_EXPERIMENTAL_TARGETS_TO_BUILD=PI32V2",
        # nothing of the system's but its C and C++ libraries: rustc would have to find the same ones where it runs
        "-DLLVM_ENABLE_ZLIB=OFF", "-DLLVM_ENABLE_ZSTD=OFF", "-DLLVM_ENABLE_LIBXML2=OFF",
        "-DLLVM_ENABLE_LIBEDIT=OFF", "-DLLVM_ENABLE_LIBPFM=OFF", "-DLLVM_ENABLE_HTTPLIB=OFF",
        "-DLLVM_ENABLE_CURL=OFF", "-DLLVM_ENABLE_PLUGINS=OFF", "-DLLVM_ENABLE_BINDINGS=OFF",
        "-DLLVM_INCLUDE_TESTS=OFF", "-DLLVM_INCLUDE_EXAMPLES=OFF", "-DLLVM_INCLUDE_BENCHMARKS=OFF",
        "-DLLVM_INCLUDE_DOCS=OFF",
        "-DLLVM_DISTRIBUTION_COMPONENTS=llvm-config;llvm-headers;llvm-libraries;cmake-exports",
        f"-DCMAKE_INSTALL_PREFIX={prefix}",
    ]
    if launcher:
        args += [f"-DCMAKE_C_COMPILER_LAUNCHER={launcher}", f"-DCMAKE_CXX_COMPILER_LAUNCHER={launcher}"]
    if sys.platform == "darwin":
        args += [f"-DCMAKE_OSX_DEPLOYMENT_TARGET={os.environ['MACOSX_DEPLOYMENT_TARGET']}"]
    elif WINDOWS:
        # the C runtime rustc itself is built with there
        args += ["-DCMAKE_C_COMPILER=cl", "-DCMAKE_CXX_COMPILER=cl", "-DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded"]
    run("cmake", *args)
    run("cmake", "--build", build, "--target", "install-distribution", *(["FileCheck"] if tests else []))
    if tests:
        shutil.copy2(build / "bin" / f"FileCheck{EXE}", prefix / "bin")


def toml(s):
    return '"' + str(s).replace("\\", "/") + '"'


def configure(path, host, llvm_prefix, version, tests):
    """bootstrap.toml for the release build: see bootstrap.pi32v2.toml for a developer's."""
    text = f"""change-id = "ignore"

[llvm]
download-ci-llvm = false
# Linux: no libstdc++.so of some version asked of the computer rustc runs on
static-libstdcpp = {'true' if sys.platform.startswith('linux') else 'false'}

[build]
host = [{toml(host)}]
target = [{toml(host)}]
extended = false
docs = false
description = {toml('pi32v2-rust ' + version)}

[rust]
# pi32v2 needs feature gates (asm_experimental_arch), which only a nightly compiler opens
channel = "nightly"
codegen-units-std = 1
# paths in the libraries' panic messages and debug information don't name the build's directory
remap-debuginfo = true
# the pi32v2 LLVM toolchain has LLVM's tools and the linker
llvm-tools = false
lld = false

[dist]
# repacked below
compression-formats = ["gz"]
compression-profile = "fast"

[target.{host}]
llvm-config = {toml(llvm_prefix / 'bin' / f'llvm-config{EXE}')}
"""
    if tests:
        text += f"llvm-filecheck = {toml(llvm_prefix / 'bin' / f'FileCheck{EXE}')}\n"
    path.write_text(text)
    print(text, flush=True)


def x(src, config, build, *args):
    # As on a developer's computer: told that it runs in CI, x.py takes it for the Rust project's own (warnings
    # in the code that wraps LLVM become errors, for one).
    env = {k: v for k, v in os.environ.items() if k not in ("CI", "GITHUB_ACTIONS")}
    run(sys.executable, src / "x.py", *args, "--config", config, "--build-dir", build, cwd=src, env=env)


def merge(dist, prefix):
    """The components of the dist tarballs (what rustup would install), in one directory."""
    tarballs = sorted(dist.glob("*.tar.gz"))
    if not tarballs:
        sys.exit(f"no tarballs in {dist}")
    for tarball in tarballs:
        with tarfile.open(tarball) as t:
            members = t.getmembers()
            top = members[0].name.split("/")[0]
            components = t.extractfile(f"{top}/components").read().decode().split()
            count = 0
            for m in members:
                parts = m.name.split("/")
                if len(parts) < 3 or parts[1] not in components or parts[2] == "manifest.in":
                    continue
                to = prefix.joinpath(*parts[2:])
                if m.isdir():
                    to.mkdir(parents=True, exist_ok=True)
                elif m.isfile():
                    to.parent.mkdir(parents=True, exist_ok=True)
                    with t.extractfile(m) as f, open(to, "wb") as o:
                        shutil.copyfileobj(f, o)
                    to.chmod(m.mode & 0o777)
                    count += 1
                else:
                    sys.exit(f"{tarball.name}: {m.name} is neither a file nor a directory")
        print(f"{tarball.name}: {', '.join(components)} ({count} files)")


def libraries(src, prefix, cargo, work):
    """core, alloc and compiler_builtins for pi32v2, built by the packed compiler from the packed sources"""
    if work.exists():
        shutil.rmtree(work)
    (work / "src").mkdir(parents=True)
    # optimised for size, as firmware for these chips is; with memcpy and its kind, which no C library brings
    (work / "Cargo.toml").write_text('[package]\nname = "empty"\nversion = "0.0.0"\nedition = "2024"\n\n'
                                     '[profile.release]\nopt-level = "s"\ncodegen-units = 1\n')
    (work / "src" / "lib.rs").write_text("#![no_std]\n")
    sources = prefix / "lib" / "rustlib" / "src" / "rust"
    # panic messages name the sources as this computer's library does; rustc's own LTO (-C lto) finds bitcode
    flags = [f"--remap-path-prefix={sources}=/rustc/{describe(src)}", "-Cembed-bitcode=yes"]
    env = {k: v for k, v in os.environ.items() if not k.startswith(("CARGO_", "RUSTUP_")) and k != "RUSTFLAGS"}
    env.update(RUSTC=str(prefix / "bin" / f"rustc{EXE}"), RUSTC_BOOTSTRAP="1",
               CARGO_ENCODED_RUSTFLAGS="\x1f".join(flags))
    run(cargo, "build", "--release", "--target", TARGET, "-Zbuild-std=core,alloc",
        "-Zbuild-std-features=compiler-builtins-mem", cwd=work, env=env)
    lib = prefix / "lib" / "rustlib" / TARGET / "lib"
    lib.mkdir(parents=True)
    built = sorted(f for f in (work / "target" / TARGET / "release" / "deps").glob("*.rlib")
                   if not f.name.startswith("libempty-"))
    for f in built:
        shutil.copy2(f, lib)
    names = {f.name.split("-")[0] for f in built}
    if names != {"libcore", "liballoc", "libcompiler_builtins"}:
        sys.exit(f"the pi32v2 libraries are not the three expected: {sorted(names)}")
    print(f"{TARGET}: {', '.join(f.name for f in built)}")


def glibc(prefix):
    """the newest glibc version the Linux binaries ask for"""
    versions = set()
    for f in [prefix / "bin" / "rustc", *prefix.glob("lib/librustc_driver-*.so")]:
        versions.update(tuple(int(n) for n in v.split(b".")) for v in re.findall(rb"GLIBC_(\d+\.\d+)", f.read_bytes()))
    return ".".join(str(n) for n in max(versions))


def notes(src, llvm_src, prefix, name, host, llvm_release):
    rustc = output(prefix / "bin" / f"rustc{EXE}", "-vV")
    needs = f"glibc {glibc(prefix)} or later" if sys.platform.startswith("linux") else None
    (prefix / "VERSION").write_text(
        f"{name}\npi32v2-rust: {describe(src)}\nllvm-project (pi32v2-llvm{', ' + llvm_release if llvm_release else ''}): "
        f"{describe(llvm_src)}\n" + (f"needs: {needs}\n" if needs else "") + rustc)
    shutil.copy(llvm_src / "llvm" / "LICENSE.TXT", prefix / "LICENSE-LLVM.TXT")
    for f in ("COPYRIGHT", "LICENSE-MIT", "LICENSE-APACHE"):
        shutil.copy(src / f, prefix / f)
    paired = (f"the pi32v2 LLVM toolchain {llvm_release} (https://github.com/jvitkauskas/pi32v2-llvm/releases)"
              if llvm_release else "the pi32v2 LLVM toolchain of the same LLVM (VERSION)")
    (prefix / "README.txt").write_text(f"""pi32v2 Rust ({name})

rustc with the target pi32v2-unknown-none: pi32v2 is the CPU core of JieLi's AC79 chips (the AC791N in the
M-VAVE FM-1 and SMK-37). From https://github.com/jvitkauskas/pi32v2-rust. Unofficial; not affiliated with the
Rust project, JieLi or M-VAVE.

  bin/rustc, bin/rustdoc                  for {host}{' (' + needs + ')' if needs else ''}
  lib/rustlib/{host}/lib   this computer's standard library, for build scripts and macros
  lib/rustlib/{TARGET}/lib      core, alloc and compiler_builtins for pi32v2 (opt-level "s")
  lib/rustlib/src/rust                    the libraries' sources, for cargo's -Zbuild-std

There is no cargo here; any recent one works. With rustup:

  rustup toolchain link pi32v2 <this directory>
  cargo +pi32v2 build --release --target {TARGET}

or without it: RUSTC=<this directory>/bin/rustc cargo build --release --target {TARGET}

It is a nightly compiler (the port needs #![feature(asm_experimental_arch)] for asm!), so -Z options work:
cargo's -Zbuild-std=core builds core with your own options instead of using the one here (a stable cargo
takes it with RUSTC_BOOTSTRAP=1).

Linking is {paired}'s
ld.lld, which rustc runs as "ld.lld": put that toolchain's bin/ on PATH, or name it with -C linker=. With
-C linker-plugin-lto its LLVM generates the code, so it has to be that release or a later one of the same LLVM.
This computer needs its own C toolchain too, for build scripts and macros.

Documentation: https://github.com/jvitkauskas/pi32v2-rust/blob/pi32v2/README-PI32V2.md and
src/doc/rustc/src/platform-support/pi32v2-unknown-none.md there.

Licence: Rust's, MIT OR Apache-2.0 (COPYRIGHT, LICENSE-MIT, LICENSE-APACHE). rustc contains LLVM, which is
Apache-2.0 with LLVM exceptions (LICENSE-LLVM.TXT).
""")


FIRMWARE = """#![no_std]
#![no_main]
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicU32, Ordering};

static COUNT: AtomicU32 = AtomicU32::new(0);

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn divide(a: u64, b: u64) -> u64 {
    a / b
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> u32 {
    let values = [3u32, 1, 2];
    let i = COUNT.load(Ordering::Relaxed) as usize;
    COUNT.store(values[i], Ordering::Relaxed);
    divide(u64::from(values[i]) << 40, core::hint::black_box(7)) as u32
}
"""


def smoke(prefix, host, work, toolchain):
    """the packed compiler builds for this computer and for pi32v2"""
    rustc = prefix / "bin" / f"rustc{EXE}"
    if work.exists():
        shutil.rmtree(work)
    work.mkdir(parents=True)
    version = output(rustc, "-vV")
    print(version)
    if f"host: {host}" not in version or "pi32v2-rust" not in version:
        sys.exit("smoke test: not the compiler that was built")
    if TARGET not in output(rustc, "--print", "target-list").split():
        sys.exit(f"smoke test: no {TARGET} in the target list")

    hello = work / "hello.rs"
    hello.write_text('fn main() {\n    println!("{}", (1..=10).map(|n: u64| n * n).sum::<u64>());\n}\n')
    run(rustc, "-O", hello, "-o", work / f"hello{EXE}", cwd=work)
    if output(work / f"hello{EXE}").strip() != "385":
        sys.exit("smoke test: the program for this computer printed something else")

    fw = work / "firmware.rs"
    fw.write_text(FIRMWARE)
    flags = ["--target", TARGET, "--edition", "2024", "-C", "opt-level=s", "-C", "panic=abort"]
    run(rustc, *flags, "--crate-type", "lib", "--emit", "obj,asm", fw, "--out-dir", work / "lib", cwd=work)
    obj = (work / "lib" / "firmware.o").read_bytes()
    # ELF, 32 bits, little-endian, machine 241
    if obj[:6] != b"\x7fELF\x01\x01" or obj[18:20] != b"\xf1\x00":
        sys.exit("smoke test: the object isn't a pi32v2 one")
    if "__udivdi3" not in (work / "lib" / "firmware.s").read_text():
        sys.exit("smoke test: the 64-bit division isn't the call it should be")

    if not toolchain:
        print("smoke test passed (nothing linked for pi32v2: no --toolchain)")
        return
    lld, nm = toolchain / "bin" / f"ld.lld{EXE}", toolchain / "bin" / f"llvm-nm{EXE}"
    link = ["-C", f"linker={lld}", "-C", "link-arg=-e", "-C", "link-arg=main", "--crate-type", "bin"]
    for name, extra in (("plain", []), ("lto", ["-C", "linker-plugin-lto", "-C", "lto=fat"])):
        elf = work / f"firmware-{name}.elf"
        run(rustc, *flags, *link, *extra, fw, "-o", elf, cwd=work)
        syms = output(nm, elf)
        # main, and the division from the packed compiler_builtins
        for s in ("main", "__udivdi3"):
            if not re.search(rf" [Tt] {re.escape(s)}$", syms, re.M):
                sys.exit(f"smoke test ({name}): {s} missing from the linked program:\n{syms}")
    print("smoke test passed")


def pack(prefix, out, name):
    if WINDOWS:
        archive = out / f"{name}.zip"
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as z:
            for f in sorted(prefix.rglob("*")):
                z.write(f, Path(name) / f.relative_to(prefix))
    else:
        archive = out / f"{name}.tar.xz"
        with tarfile.open(archive, "w:xz") as t:
            t.add(prefix, arcname=name)
    print(f"package: {archive} ({archive.stat().st_size // (1 << 20)} MiB)")
    return archive


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", type=Path, required=True, help="this repository")
    ap.add_argument("--llvm", type=Path, required=True, help="a checkout of pi32v2-llvm")
    ap.add_argument("--build", type=Path, required=True)
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--version", required=True, help="the package's version, such as 1.100.0-1")
    ap.add_argument("--llvm-release", default=None, help="the pi32v2-llvm release --llvm is, for the notes")
    ap.add_argument("--toolchain", type=Path, default=None, help="an unpacked pi32v2 LLVM toolchain, to link with")
    ap.add_argument("--tests", action="store_true")
    ap.add_argument("--launcher", default=None, help="compiler launcher for LLVM's build (ccache, sccache)")
    a = ap.parse_args()
    src, llvm_src, build, out = a.src.resolve(), a.llvm.resolve(), a.build.resolve(), a.out.resolve()
    toolchain = a.toolchain.resolve() if a.toolchain else None
    key = ("linux" if sys.platform.startswith("linux") else sys.platform, platform.machine().lower())
    if key not in HOSTS:
        sys.exit(f"no package for {key[0]} on {key[1]}")
    host, backend, platform_name = HOSTS[key]
    name = f"pi32v2-rust-{a.version}-{platform_name}"
    if sys.platform == "darwin":
        # one deployment target for LLVM, rustc and the standard library
        os.environ.setdefault("MACOSX_DEPLOYMENT_TARGET", "11.0")
    prefix = out / name
    if prefix.exists():
        shutil.rmtree(prefix)
    out.mkdir(parents=True, exist_ok=True)
    build.mkdir(parents=True, exist_ok=True)

    llvm_prefix = build / "llvm-install"
    llvm(llvm_src, build / "llvm", llvm_prefix, backend, a.tests, a.launcher)
    config, rust_build = build / "bootstrap.toml", build / "rust"
    configure(config, host, llvm_prefix, a.version, a.tests)
    dist = rust_build / "dist"
    if dist.exists():
        shutil.rmtree(dist)
    x(src, config, rust_build, "dist", "rustc", "rust-std", "rust-src")
    merge(dist, prefix)
    # the cargo x.py built everything with
    libraries(src, prefix, rust_build / host / "stage0" / "bin" / f"cargo{EXE}", build / "pi32v2-libraries")
    notes(src, llvm_src, prefix, name, host, a.llvm_release)
    smoke(prefix, host, build / "smoke", toolchain)
    if a.tests:
        x(src, config, rust_build, "test", "--stage", "2", *TESTS)
    pack(prefix, out, name)


if __name__ == "__main__":
    main()
