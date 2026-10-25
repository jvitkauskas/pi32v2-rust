// JieLi pi32v2 C calling convention, as LLVM's clang lowers it for the pi32v2 target
// (clang/lib/CodeGen/Targets/PI32V2.cpp in the pi32v2-llvm fork; observed ABI: pi32v2-isa docs/ABI.md):
// - aggregates, as arguments and as results, are passed by reference to a caller-made copy (not `byval`),
//   a result through a hidden pointer argument (sret);
// - scalars are passed directly; integers narrower than 32 bits are extended to 32 (the backend promotes
//   i1/i8/i16 to i32, and passes i32/f32 in r0..r3, then on the stack; i64/f64 take two slots).

use rustc_abi::TyAbiInterface;

use crate::callconv::{ArgAbi, FnAbi};

fn classify_ret<Ty>(ret: &mut ArgAbi<'_, Ty>) {
    if !ret.layout.is_sized() {
        return;
    }
    if ret.layout.is_aggregate() {
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
    if arg.layout.pass_indirectly_in_non_rustic_abis(cx) || arg.layout.is_aggregate() {
        arg.make_indirect();
    } else {
        arg.extend_integer_width_to(32);
    }
}

pub(crate) fn compute_abi_info<'a, Ty, C>(cx: &C, fn_abi: &mut FnAbi<'a, Ty>)
where
    Ty: TyAbiInterface<'a, C> + Copy,
{
    if !fn_abi.ret.is_ignore() {
        classify_ret(&mut fn_abi.ret);
    }
    for arg in fn_abi.args.iter_mut() {
        if arg.is_ignore() {
            continue;
        }
        classify_arg(cx, arg);
    }
}
