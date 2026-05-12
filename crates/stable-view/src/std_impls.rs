//! Implementations for:
//!
//! - `std::sync::{MutexGuard, RwLockReadGuard, RwLockWriteGuard}`.

#![expect(unsafe_code, reason = "implement the unsafe `aliasable-view` traits")]
#![warn(clippy::missing_inline_in_public_items, reason = "trivial impls")]

use core::mem::transmute;
use std::sync::{MutexGuard, RwLockReadGuard, RwLockWriteGuard};

use variance_family::{Unvarying, VaryingRef, VaryingRefMut};

use crate::{
    traits::{StableView, StableViewMut},
    view_kinds::{PointerViewKind, SetDefaultView, SetDefaultViewMut},
};


// ================================================================
//  `sync::MutexGuard`
// ================================================================

// SAFETY:
// Until the source value's `'b` lifetime parameter expires *or* the source value is dropped
// and releases its exclusive borrow rights over the referent, we know that:
// - Moving the `MutexGuard` does not invalidate immutable references to its `T` referent, because a
//   `MutexGuard` argument doesn't hold exclusivity for its whole scope, only until it drops;
//   therefore, it cannot have `Box`'s `noalias` semantics.
// - Coercing the `MutexGuard` does not invalidate its `T` referent for the same reason above.
// - No sound immutable operation on the `MutexGuard` can invalidate shared references to the `T`
//   referent; otherwise, that could invalidate other references obtained from other shared
//   references to the same `MutexGuard`.
//
// (After the `'b` lifetime parameter expires, the source `Mutex` could be dropped,
// and that's fine.)
unsafe impl<'a, 'b, 'other_data, T> StableView<'a, 'other_data, MutexGuard<'b, T>>
for PointerViewKind
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a MutexGuard<'_, T>) -> &'stable T
    where
        'other_data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a T = data;

        // SAFETY: See the "`transmute` in `view` Implementation" section of the `StableView` docs.
        unsafe {
            transmute::<
                &'a T,
                &'stable T,
            >(stable_eq_a)
        }
    }
}

impl<'b, 'other_data, T> SetDefaultView<'_, 'other_data> for MutexGuard<'b, T>
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type Default = PointerViewKind;
}

// SAFETY:
// Until the source value's `'b` lifetime parameter expires *or* the source value is dropped
// and releases its exclusive borrow rights over the referent, we know that:
// - Moving the `MutexGuard` does not invalidate immutable references to its `T` referent, because a
//   `MutexGuard` argument doesn't hold exclusivity for its whole scope, only until it drops;
//   therefore, it cannot have `Box`'s `noalias` semantics.
// - Coercing the `MutexGuard` does not invalidate its `T` referent for the same reason above,
//   noting that the referent is not stored inline.
// - No-ops on the source data value are fine.
//
// (After the `'b` lifetime parameter expires, the source `RefCell` could be dropped,
// and that's fine.)
unsafe impl<'a, 'b, 'other_data, T> StableViewMut<'a, 'other_data, MutexGuard<'b, T>>
for PointerViewKind
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type ViewMut = VaryingRefMut<Unvarying<T>>;

    #[inline]
    unsafe fn view_mut<'stable>(data: &'a mut MutexGuard<'b, T>) -> &'stable mut T
    where
        'other_data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a mut T = data;

        // SAFETY: See the "`transmute` in `view_mut` Implementation" section of the
        // `StableViewMut` docs.
        unsafe {
            transmute::<
                &'a mut T,
                &'stable mut T,
            >(stable_eq_a)
        }
    }
}

impl<'b, 'other_data, T> SetDefaultViewMut<'_, 'other_data> for MutexGuard<'b, T>
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type DefaultMut = PointerViewKind;
}


// ================================================================
//  `sync::RwLockReadGuard`
// ================================================================

// SAFETY:
// Until the source value's `'b` lifetime parameter expires *or* the source value is dropped
// and releases its shared borrow rights over the referent, we know that:
// - Moving the `RwLockReadGuard` does not invalidate immutable references to its `T` referent,
//   since the `T` referent is stored elsewhere (in a `RwLock`) and `cell::RwLockReadGuard` is
//   expected to alias other shared references (and thus does not assert `noalias` when moved),
// - Coercing the `RwLockReadGuard` does not invalidate its `T` referent for the same reasons above
//   and below,
// - No sound immutable operation on the `RwLockReadGuard` can invalidate shared references to the
//   `T` referent; otherwise, that could invalidate other references obtained from other shared
//   references to the same `RwLockReadGuard`.
//
// (After the `'b` lifetime parameter expires, the source `RwLock` could be dropped,
// and that's fine.)
unsafe impl<'a, 'b, 'other_data, T> StableView<'a, 'other_data, RwLockReadGuard<'b, T>>
for PointerViewKind
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a RwLockReadGuard<'b, T>) -> &'stable T
    where
        'other_data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a T = data;

        // SAFETY: See the "`transmute` in `view` Implementation" section of the `StableView` docs.
        unsafe {
            transmute::<
                &'a T,
                &'stable T,
            >(stable_eq_a)
        }
    }
}

impl<'b, 'other_data, T> SetDefaultView<'_, 'other_data> for RwLockReadGuard<'b, T>
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type Default = PointerViewKind;
}


// ================================================================
//  `sync::RwLockWriteGuard`
// ================================================================

// SAFETY:
// Until the source value's `'b` lifetime parameter expires *or* the source value is dropped
// and releases its exclusive borrow rights over the referent, we know that:
// - Moving the `RwLockWriteGuard` does not invalidate immutable references to its `T` referent,
//   because a `RwLockWriteGuard` argument doesn't hold exclusivity for its whole scope, only until
//   it drops; therefore, it cannot have `Box`'s `noalias` semantics.
// - Coercing the `RwLockWriteGuard` does not invalidate its `T` referent for the same reason above.
// - No sound immutable operation on the `RwLockWriteGuard` can invalidate shared references to
//   the `T` referent; otherwise, that could invalidate other references obtained from other shared
//   references to the same `RwLockWriteGuard`.
//
// (After the `'b` lifetime parameter expires, the source `RwLock` could be dropped,
// and that's fine.)
unsafe impl<'a, 'b, 'other_data, T> StableView<'a, 'other_data, RwLockWriteGuard<'b, T>>
for PointerViewKind
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a RwLockWriteGuard<'_, T>) -> &'stable T
    where
        'other_data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a T = data;

        // SAFETY: See the "`transmute` in `view` Implementation" section of the `StableView` docs.
        unsafe {
            transmute::<
                &'a T,
                &'stable T,
            >(stable_eq_a)
        }
    }
}

impl<'b, 'other_data, T> SetDefaultView<'_, 'other_data> for RwLockWriteGuard<'b, T>
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type Default = PointerViewKind;
}

// SAFETY:
// Until the source value's `'b` lifetime parameter expires *or* the source value is dropped
// and releases its exclusive borrow rights over the referent, we know that:
// - Moving the `RwLockWriteGuard` does not invalidate immutable references to its `T` referent,
//   because a `RwLockWriteGuard` argument doesn't hold exclusivity for its whole scope, only until
//   it drops; therefore, it cannot have `Box`'s `noalias` semantics.
// - Coercing the `RwLockWriteGuard` does not invalidate its `T` referent for the same reason above,
//   noting that the referent is not stored inline.
// - No-ops on the source data value are fine.
//
// (After the `'b` lifetime parameter expires, the source `RefCell` could be dropped,
// and that's fine.)
unsafe impl<'a, 'b, 'other_data, T> StableViewMut<'a, 'other_data, RwLockWriteGuard<'b, T>>
for PointerViewKind
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type ViewMut = VaryingRefMut<Unvarying<T>>;

    #[inline]
    unsafe fn view_mut<'stable>(data: &'a mut RwLockWriteGuard<'b, T>) -> &'stable mut T
    where
        'other_data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a mut T = data;

        // SAFETY: See the "`transmute` in `view_mut` Implementation" section of the
        // `StableViewMut` docs.
        unsafe {
            transmute::<
                &'a mut T,
                &'stable mut T,
            >(stable_eq_a)
        }
    }
}

impl<'b, 'other_data, T> SetDefaultViewMut<'_, 'other_data> for RwLockWriteGuard<'b, T>
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type DefaultMut = PointerViewKind;
}
