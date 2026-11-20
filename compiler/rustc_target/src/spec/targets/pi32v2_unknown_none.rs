use crate::spec::{
    Arch, Cc, LinkerFlavor, Lld, PanicStrategy, RelocModel, Target, TargetMetadata, TargetOptions,
};

/// JieLi's pi32v2, the CPU of the AC79 chips (the AC791N in M-VAVE's FM-1 and SMK-37), bare metal, with the
/// pi32v2-llvm fork's LLVM (https://github.com/jvitkauskas/pi32v2-llvm). See the platform-support page.
pub(crate) fn target() -> Target {
    Target {
        llvm_target: "pi32v2".into(),
        metadata: TargetMetadata {
            description: Some("JieLi pi32v2 (AC79), bare metal".into()),
            tier: Some(3),
            host_tools: Some(false),
            std: Some(false),
        },
        pointer_width: 32,
        // As clang gives it for -target pi32v2.
        data_layout: "e-m:e-p:32:32-i1:32-i64:32-f64:32-n8:16:32-a:0:32-S32".into(),
        arch: Arch::Pi32v2,

        options: TargetOptions {
            // The instruction set with floating point, revision 1, as m-apps' C uses it (-mcpu=r3), so that Rust
            // and C share target attributes and inline into each other under LTO. (`-C target-cpu=ac791n`
            // exists, but the code generator doesn't use it yet.)
            cpu: "r3".into(),
            // The pi32v2 LLVM toolchain's lld.
            linker: Some("ld.lld".into()),
            linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
            // Loads and stores of up to a word are single accesses, and a fence is csync, but the backend makes
            // read-modify-write operations calls to __sync_* functions, which nothing here provides: no CAS.
            max_atomic_width: Some(32),
            atomic_cas: false,
            panic_strategy: PanicStrategy::Abort,
            relocation_model: RelocModel::Static,
            emit_debug_gdb_scripts: false,
            eh_frame_header: false,
            ..Default::default()
        },
    }
}
