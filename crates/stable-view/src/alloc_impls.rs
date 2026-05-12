//! Implementations for:
//!
//! - `borrow::Cow`,
//! - `rc::{Rc, Weak}`,
//! - `string::String`,
//! - `arc::{Arc, Weak}`,
//! - `vec::Vec<T>`.
//!
//! TODO, if it becomes possible:
//! - `collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque}`.

#![expect(unsafe_code, reason = "implement the unsafe `aliasable-view` traits")]
#![warn(clippy::missing_inline_in_public_items, reason = "trivial impls")]
// We *don't* disable `clippy::absolute_paths`, since the types are repeated quite a bit
// due to complicated bounds. (Plus, they're well-known.)

use core::mem::transmute;
use alloc::{
    borrow::{Cow, ToOwned},
    vec::Vec,
};

use variance_family::{Unvarying, Varying, VaryingRef, VaryingRefMut, WithLifetime};

use crate::{
    traits::{StableClone, StableView, StableViewMut},
    view_kinds::{PointerViewKind, RecursiveViewKind, SetDefaultView, SetDefaultViewMut},
};


// ================================================================
//  `borrow::Cow`
// ================================================================

// SAFETY: We basically have two separate cases (borrowed and owned). We know that the borrowed
// branch is `&'b B`, which can provide a `&'stable B` reference since `'b: 'other_data`.
// The crazy-complicated trait bounds require that the owned branch has a pointer view
// to `&'stable B`. We then just match the cases.
//
// By the implementation of a `StableView` for the owned branch, we know that the returned
// `&'stable B` in the owned branch is not invalidated by the three operations applied to the
// source `Cow::Owned(owned)` value (for at least the `'other_data` upper boudn). By the safety
// comment for the `StableView` impl of `&'b T` for `PointerViewKind`, the same holds for the
// borrowed branch, though it's simpler to use the `unsafe`-free shortening of `&'b B` to
// `&'stable B` instead of calling `view`.
unsafe impl<'a, 'b, 'other_data, B> StableView<'a, 'other_data, Cow<'b, B>> for PointerViewKind
where
    'b: 'other_data,
    B: 'b + ?Sized + ToOwned,
    Self: StableView<
        'a, 'other_data, B::Owned,
        View: for<'stable> WithLifetime<'stable, 'a, &'other_data (), Is = &'stable B>,
    >,
{
    type View = VaryingRef<Unvarying<B>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a Cow<'b, B>) -> &'stable B
    where
        'other_data: 'stable,
        'stable: 'a,
    {
        match data {
            Cow::Borrowed(borrowed) => borrowed,
            Cow::Owned(owned) => {
                // SAFETY: The returned view can only be used at a given time if, from just after
                // this function returns until the time of use, only the three operations are
                // performed, and if `'other_data` has not ended. This constraint is precisely what
                // *our* `view` caller unsafely asserts, so this is sound.
                // In other words, we have simply forwarded the safety preconditions to the caller.
                unsafe { <Self as StableView<'a, 'other_data, B::Owned>>::view(owned) }
            }
        }
    }
}

impl<'a, 'b, 'other_data, B> SetDefaultView<'a, 'other_data> for Cow<'b, B>
where
    'b: 'other_data,
    B: 'b + ?Sized + ToOwned,
    PointerViewKind: StableView<
        'a, 'other_data, B::Owned,
        View: for<'stable> WithLifetime<'stable, 'a, &'other_data (), Is = &'stable B>,
    >,
{
    type Default = PointerViewKind;
}

// It *can* impl `StableClone` when `Owned: Clone` (though not in general, since `ToOwned`
// only uses `Clone::clone` when `Owned: Clone`)

// SAFETY: We are essentially deferring to the `PointerViewKind` impl of `StableClone` for
// either `&'b B` or `B::Owned`. Note that `Cow<'b, B>: Clone` even when `B: ToOwned + !Clone`,
// but when `B: Clone`, the `Clone` impl of `Cow` either:
// - copies (so, `Clone::clone`s) the `&'b B` in a `Cow::Borrowed`
// - `Clone::clone`s the `B::Owned` in a `Cow::Owned`.
// Since our `view` impl is deferred in the same way, this impl is correct.
// More rigorously, note that the three requirements are clearly satisfied in *either* the
// `Cow::Borrowed` or `Cow::Owned` cases. Additionally, operations done through `&` references
// to parts of a `Cow` value are *entirely incapable* of switching the owned/borrowed state of that
// `Cow` value, since `Cow` is an enum, and the enum discriminant is not internally mutable.
//
/// # Robust Guarantee
/// The conceptual pool associated with [`PointerViewKind`] and a `Cow::Borrowed(data)` value
/// (where `data: &'b B`) is guaranteed to be nonempty for at least lifetime `'b`, but may be e
/// mptied after `'b` ends.
///
/// The definition of conceptual pool associated with [`PointerViewKind`] and a `Cow::Owned(data)`
/// value (where `data: B::Owned`) is the conceptual pool definition used by the implementation
/// of `StableClone<'_, '_, B::Owned>` for `PointerViewKind`. In other words, the conceptual pool
/// definition for `Cow::Owned` is simply deferred to `B::Owned`.
///
/// The above two cases cover the definition of conceptual pool used by any `Cow` value with
/// the [`PointerViewKind`] view kind.
unsafe impl<'a, 'b, 'other_data, B> StableClone<'a, 'other_data, Cow<'b, B>> for PointerViewKind
where
    'b: 'other_data,
    B: 'b + ?Sized + ToOwned<Owned: Clone>,
    Self: StableClone<
        'a, 'other_data, B::Owned,
        View: for<'stable> WithLifetime<'stable, 'a, &'other_data (), Is = &'stable B>,
    >,
{}

// SAFETY: We basically have two separate cases (borrowed and owned).
//
// In the borrowed case, by `VB`'s `StableView` impl for `&'b B`, we know that the views returned
// by its returned `view` are not invalidated by the three validations (for at least `'other_data`),
// and since our `view` simply returns their `view`, our impl is sound in that case.
//
// Likewise for the owned case, but `VB` becomes `VO` and `&'b B` becomes `B::Owned`.
unsafe impl<'a, 'b, 'other_data, B, VB, VO> StableView<'a, 'other_data, Cow<'b, B>>
for RecursiveViewKind<(VB, VO)>
where
    // Most comprehensible Rust where-bound.
    B: 'b + ?Sized + ToOwned,
    VB: StableView<'a, 'other_data, &'b B>,
    VO: StableView<
        'a, 'other_data, B::Owned,
        View: for<'stable> WithLifetime<
            'stable, 'a, &'other_data (),
            Is = Varying<'stable, 'a, &'other_data (), VB::View>,
        >,
    >,
{
    type View = VB::View;

    #[inline]
    unsafe fn view<'stable>(
        data: &'a Cow<'b, B>,
    ) -> Varying<'stable, 'a, &'other_data (), Self::View>
    where
        'other_data: 'stable,
        'stable: 'a,
    {
        match data {
            Cow::Borrowed(borrowed) => {
                // SAFETY: The returned view can only be used at a given time if, from just after
                // this function returns until the time of use, only the three operations are
                // performed, and if `'other_data` has not ended. This constraint is precisely what
                // *our* `view` caller unsafely asserts, so this is sound.
                // In other words, we have simply forwarded the safety preconditions to the caller.
                unsafe { VB::view(borrowed) }
            }
            Cow::Owned(owned) => {
                // SAFETY: Same as above branch.
                unsafe { VO::view(owned) }
            }
        }
    }
}

// SAFETY: We are essentially deferring to either the `VB` impl of `StableClone` for `&'b B` or
// the `VO` impl of `StableClone` for `B::Owned`. Note that `Cow<'b, B>: Clone` even when
// `B: ToOwned + !Clone`, but when `B: Clone`, the `Clone` impl of `Cow` either:
// - copies (so, `Clone::clone`s) the `&'b B` in a `Cow::Borrowed`
// - `Clone::clone`s the `B::Owned` in a `Cow::Owned`.
// Since our `view` impl is deferred in the same way, this impl is correct.
// More rigorously, note that the three requirements are clearly satisfied in *either* the
// `Cow::Borrowed` or `Cow::Owned` cases. Additionally, operations done through `&` references
// to parts of a `Cow` value are *entirely incapable* of switching the owned/borrowed state of that
// `Cow` value, since `Cow` is an enum, and the enum discriminant is not internally mutable.
//
/// # Robust Guarantee
/// The definition of conceptual pool associated with `RecursiveViewKind<(VB, VO)>` and a
/// `Cow::Borrowed(data)` value (where `data: &'b B`) is the conceptual pool definition used by the
/// implementation of `StableClone<'_, '_, &'b B>` for `VB`. In other words, the conceptual pool
/// definition for `Cow::Borrowed` is simply deferred to the one associated with `VB` and `&'b B`.
///
/// The definition of conceptual pool associated with `RecursiveViewKind<(VB, VO)>` and a
/// `Cow::Owned(data)` value (where `data: B::Owned`) is the conceptual pool definition used by the
/// implementation of `StableClone<'_, '_, B::Owned>` for `VO`. In other words, the conceptual pool
/// definition for `Cow::Owned` is simply deferred to the one associated with `VO` and `B::Owned`.
///
/// The above two cases cover the definition of conceptual pool used by any `Cow` value with
/// the `RecursiveViewKind<(VB, VO)>` view kind.
unsafe impl<'a, 'b, 'other_data, B, VB, VO> StableClone<'a, 'other_data, Cow<'b, B>>
for RecursiveViewKind<(VB, VO)>
where
    B: 'b + ?Sized + ToOwned<Owned: Clone>,
    VB: StableClone<'a, 'other_data, &'b B>,
    VO: StableClone<
        'a, 'other_data, B::Owned,
        View: for<'stable> WithLifetime<
            'stable, 'a, &'other_data (),
            Is = Varying<'stable, 'a, &'other_data (), VB::View>,
        >,
    >,
{}

// ================================================================
//  `vec::Vec`
// ================================================================

// SAFETY: We will go through each of the three operations. The `'other_data` upper bound
// doesn't particularly matter in the case of the owned `Vec<T>` type.
//
// First, moves. Moving a `Vec<T>` does not currently invalidate references to its contents,
// and that is *very* unlikely to ever change, due to concern about breaking existing code making
// it "out of the question": https://github.com/rust-lang/rfcs/pull/3712#issuecomment-3715013712
//
// Second, coercions. As noted by `StableView`, it should be covered by the first and third cases.
//
// Third, operations done to data derived from parts of `Vec<T>` only through `&` references. Since
// `Vec<T>` doesn't use internal mutability, operations done on shared references to part or all of
// a `Vec<T>` value cannot invalidate operations done on a shared reference to its `[T]` contents.
// Note that operations done on one `&Vec<T>` **CAN** invalidate a different `&Vec<T>`... if the
// latter was derived from an older `&mut Vec<T>` (or other `Unique`-tagged pointer to `Vec<T>`).
// TODO: Miri test; pointers are hard. I need to make sure that passing a `&mut Vec<T>` to
// `StableView::view` doesn't cause a problem.
unsafe impl<'a, 'other_data, T: 'other_data> StableView<'a, 'other_data, Vec<T>>
for PointerViewKind
{
    type View = VaryingRef<Unvarying<[T]>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a Vec<T>) -> &'stable [T]
    where
        'other_data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a [T] = data;

        // SAFETY: See the safety comment of the above `unsafe` trait impl.
        // The caller of `view` unsafely asserts that the returned view is only used when the source
        // data has only been moved, coerced, or immutably operated on (in any quantity and order)
        // from just after this function returns (and, therefore, also starting from now, since we
        // have a `&` borrow of the source data) until the time of use, and that `'other_data` has
        // not ended when it's used. By the same reasoning that enables the `unsafe` trait impl, we
        // know that those uses do not invalidate `'stable` data and that lifetime extension of the
        // `'stable` lifetime parameter is sound. Any further soundness concerns are the
        // responsibility of the caller of `view`.
        unsafe {
            transmute::<
                &'a [T],
                &'stable [T],
            >(stable_eq_a)
        }
    }
}

impl<'other_data, T: 'other_data> SetDefaultView<'_, 'other_data> for Vec<T> {
    type Default = PointerViewKind;
}

// SAFETY: We will go through each of the three operations. The `'other_data` upper bound
// doesn't particularly matter. As noted by `StableViewMut`, the second and third operations
// aren't particularly noteworthy either; our `view` impl doesn't do something strange that would
// break them while somehow still supporting the first operation.
//
// Then, moves. Moving a `Vec<T>` does not currently invalidate references to its contents,
// and that is *very* unlikely to ever change, due to concern about breaking existing code making
// it "out of the question": https://github.com/rust-lang/rfcs/pull/3712#issuecomment-3715013712
unsafe impl<'a, 'other_data, T: 'other_data> StableViewMut<'a, 'other_data, Vec<T>>
for PointerViewKind
{
    type ViewMut = VaryingRefMut<Unvarying<[T]>>;

    #[inline]
    unsafe fn view_mut<'stable>(data: &'a mut Vec<T>) -> &'stable mut [T]
    where
        'other_data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a mut [T] = data;

        // SAFETY: See the safety comment of the above `unsafe` trait impl.
        // The caller of `view_mut` unsafely asserts that the returned view is only used when the
        // source data has only been moved or coerced (or had no-ops occur) from just after this
        // function returns (and, therefore, also starting from now, since we have a `&mut` borrow
        // of the source data) until the time of use, and that `'other_data` has not ended when it's
        // used. By the same reasoning that enables the `unsafe` trait impl, we know that those uses
        // do not invalidate `'stable` data and that lifetime extension of the `'stable` lifetime
        // parameter is sound. Any further soundness concerns are the responsibility of the caller
        // of `view_mut`.
        unsafe {
            transmute::<
                &'a mut [T],
                &'stable mut [T],
            >(stable_eq_a)
        }
    }
}

impl<'other_data, T: 'other_data> SetDefaultViewMut<'_, 'other_data> for Vec<T> {
    type DefaultMut = PointerViewKind;
}
