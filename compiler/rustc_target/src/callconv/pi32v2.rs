// JieLi pi32v2 C calling convention, as LLVM's clang lowers it for the pi32v2 target
// (clang/lib/CodeGen/Targets/PI32V2.cpp in the pi32v2-llvm fork; observed ABI: pi32v2-isa docs/ABI.md):
// - aggregates, as arguments and as results, are passed by reference to a caller-made copy (not `byval`),
//   a result through a hidden pointer argument (sret);
// - scalars are passed directly; integers narrower than 32 bits are extended to 32 (the backend promotes
//   i1/i8/i16 to i32, and passes i32/f32 in r0..r3, then on the stack; i64/f64 take two slots);
// - wider scalars (i128) go as aggregates do, as clang's _BitInt(65..=128);
// - an empty struct takes an argument slot, by reference to a copy.

use rustc_abi::TyAbiInterface;

use crate::callconv::{ArgAbi, FnAbi};

fn classify_ret<Ty>(ret: &mut ArgAbi<'_, Ty>) {
    if !ret.layout.is_sized() {
        return;
    }
    if ret.is_ignore() {
        // clang returns an empty struct through a hidden pointer, as any aggregate, where Rust returns nothing.
        // rustc's codegen has no indirect return of a zero-sized value, so such a signature is refused instead
        // (rustc_monomorphize's ABI check).
        return;
    }
    // An integer wider than 64 bits (i128 here, _BitInt(65..=128) in clang) comes back through a hidden
    // pointer too.
    if ret.layout.is_aggregate() || ret.layout.size.bits() > 64 {
        ret.make_indirect();
    } else {
        ret.extend_integer_width_to(32);
    }
}

fn classify_arg<'a, Ty, C>(cx: &C, arg: &mut ArgAbi<'a, Ty>)
where
    Ty: TyAbiInterface<'a, C> + Copy,
{
    if !arg.layout.is_sized() {
        return;
    }
    if arg.is_ignore() {
        // clang passes an empty struct as it passes any aggregate, by reference to a copy: it takes an argument
        // slot (also among variadic arguments), so ignoring it would move every later argument.
        if arg.layout.is_zst() {
            arg.make_indirect_from_ignore();
        }
        return;
    }
    // Integers wider than 64 bits (i128 here, _BitInt(65..=128) in clang) go by reference to a copy too
    // (clang's byval, which the backend passes as one pointer word).
    if arg.layout.pass_indirectly_in_non_rustic_abis(cx)
        || arg.layout.is_aggregate()
        || arg.layout.size.bits() > 64
    {
        arg.make_indirect();
    } else {
        arg.extend_integer_width_to(32);
    }
}

pub(crate) fn compute_abi_info<'a, Ty, C>(cx: &C, fn_abi: &mut FnAbi<'a, Ty>)
where
    Ty: TyAbiInterface<'a, C> + Copy,
{
    classify_ret(&mut fn_abi.ret);
    for arg in fn_abi.args.iter_mut() {
        classify_arg(cx, arg);
    }
}
