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

#![expect(clippy::undocumented_unsafe_blocks, reason = "TODO")]

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
// source `Cow::Owned(owned)` value. By the safety comment for the `StableView` impl of `&'b T`
// for `PointerViewKind`, the same holds for the borrowed branch, though it's simpler to use
// the `unsafe`-free shortening of `&'b B` to `&'stable B` instead of calling `view`.
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

unsafe impl<'a, 'b, 'other_data, B> StableClone<'a, 'other_data, Cow<'b, B>> for PointerViewKind
where
    'b: 'other_data,
    B: 'b + ?Sized + ToOwned<Owned: Clone>,
    Self: StableClone<
        'a, 'other_data, B::Owned,
        View: for<'stable> WithLifetime<'stable, 'a, &'other_data (), Is = &'stable B>,
    >,
{}

unsafe impl<'a, 'b, 'other_data, B, VB, VO> StableView<'a, 'other_data, Cow<'b, B>>
for RecursiveViewKind<(VB, VO)>
// Most comprehensible Rust where-bound.
where
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
                unsafe { VB::view(borrowed) }
            }
            Cow::Owned(owned) => {
                unsafe { VO::view(owned) }
            }
        }
    }
}

unsafe impl<'a, 'b, 'other_data, B, VB, VO> StableClone<'a, 'other_data, Cow<'b, B>>
for RecursiveViewKind<(VB, VO)>
where
    B: 'b + ?Sized + ToOwned<Owned: Clone>,
    VB: StableClone<'a, 'other_data, &'b B>,
    VO: StableView<
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
