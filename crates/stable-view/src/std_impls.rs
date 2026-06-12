//! Implementations for:
//!
//! - `std::sync::{MutexGuard, RwLockReadGuard, RwLockWriteGuard}`.

#![expect(unsafe_code, reason = "implement the unsafe `stable-view` traits")]
#![warn(clippy::missing_inline_in_public_items, reason = "trivial impls")]

use core::mem::transmute;
use std::sync::{MutexGuard, RwLockReadGuard, RwLockWriteGuard};

use variance_family::{Unvarying, VaryingRef, VaryingRefMut};

use crate::{
    traits::{StableView, StableViewMut},
    view_kinds::{DefaultStableView, DefaultStableViewMut, ReferenceViewKind},
};


// ================================================================
//  `sync::MutexGuard`
// ================================================================

impl<'a, 'b, 'data, T> StableView<'a, 'data, MutexGuard<'b, T>> for ReferenceViewKind
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a MutexGuard<'_, T>) -> &'stable T
    where
        'data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a T = data;

        // SAFETY: See "`transmute` in `view(_mut)` Implementation" in the `StableView::view` docs.
        // This is essentially the same as the impl for `core::cell::RefMut<'b, T>`.
        unsafe {
            transmute::<
                &'a T,
                &'stable T,
            >(stable_eq_a)
        }
    }
}

impl<'b, 'data, T> DefaultStableView<'_, 'data> for MutexGuard<'b, T>
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type Default = ReferenceViewKind;
}

impl<'a, 'b, 'data, T> StableViewMut<'a, 'data, MutexGuard<'b, T>> for ReferenceViewKind
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type ViewMut = VaryingRefMut<Unvarying<T>>;

    #[inline]
    unsafe fn view_mut<'stable>(data: &'a mut MutexGuard<'b, T>) -> &'stable mut T
    where
        'data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a mut T = data;

        // SAFETY: See "`transmute` in `view(_mut)` Implementation" in the `StableView::view` docs.
        // This is essentially the same as the impl for `core::cell::RefMut<'b, T>`.
        unsafe {
            transmute::<
                &'a mut T,
                &'stable mut T,
            >(stable_eq_a)
        }
    }
}

impl<'b, 'data, T> DefaultStableViewMut<'_, 'data> for MutexGuard<'b, T>
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type DefaultMut = ReferenceViewKind;
}


// ================================================================
//  `sync::RwLockReadGuard`
// ================================================================

impl<'a, 'b, 'data, T> StableView<'a, 'data, RwLockReadGuard<'b, T>> for ReferenceViewKind
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a RwLockReadGuard<'b, T>) -> &'stable T
    where
        'data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a T = data;

        // SAFETY: See "`transmute` in `view(_mut)` Implementation" in the `StableView::view` docs.
        // This is essentially the same as the impl for `core::cell::Ref<'b, T>`.
        unsafe {
            transmute::<
                &'a T,
                &'stable T,
            >(stable_eq_a)
        }
    }
}

impl<'b, 'data, T> DefaultStableView<'_, 'data> for RwLockReadGuard<'b, T>
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type Default = ReferenceViewKind;
}


// ================================================================
//  `sync::RwLockWriteGuard`
// ================================================================

impl<'a, 'b, 'data, T> StableView<'a, 'data, RwLockWriteGuard<'b, T>> for ReferenceViewKind
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a RwLockWriteGuard<'_, T>) -> &'stable T
    where
        'data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a T = data;

        // SAFETY: See "`transmute` in `view(_mut)` Implementation" in the `StableView::view` docs.
        // This is essentially the same as the impl for `core::cell::RefMut<'b, T>`.
        unsafe {
            transmute::<
                &'a T,
                &'stable T,
            >(stable_eq_a)
        }
    }
}

impl<'b, 'data, T> DefaultStableView<'_, 'data> for RwLockWriteGuard<'b, T>
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type Default = ReferenceViewKind;
}

impl<'a, 'b, 'data, T> StableViewMut<'a, 'data, RwLockWriteGuard<'b, T>> for ReferenceViewKind
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type ViewMut = VaryingRefMut<Unvarying<T>>;

    #[inline]
    unsafe fn view_mut<'stable>(data: &'a mut RwLockWriteGuard<'b, T>) -> &'stable mut T
    where
        'data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a mut T = data;

        // SAFETY: See "`transmute` in `view(_mut)` Implementation" in the `StableView::view` docs.
        // This is essentially the same as the impl for `core::cell::RefMut<'b, T>`.
        unsafe {
            transmute::<
                &'a mut T,
                &'stable mut T,
            >(stable_eq_a)
        }
    }
}

impl<'b, 'data, T> DefaultStableViewMut<'_, 'data> for RwLockWriteGuard<'b, T>
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type DefaultMut = ReferenceViewKind;
}
