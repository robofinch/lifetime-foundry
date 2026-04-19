//! With the `more-impls` feature, also:
//!
//! - `cell::{OnceCell, LazyCell}`,
//! - `cmp::Ordering`,
//! - `mem::{ManuallyDrop, MaybeUninit}`,
//! - `num::NonZero*`,
//! - `ptr::NonNull`,
//! - `slice::{Iter, IterMut}`,
//! - `sync::atomic::*`.

#![expect(unsafe_code, reason = "assert variance and permission to impl traits for `core` types")]
#![expect(clippy::absolute_paths, reason = "one-off uses of many different types")]

use core::marker::PhantomData;

use crate::{
    generic_wrapper, macros::concrete_types, phantom_zst_methods, unvarying,
    varying_ref_mut_wrapper, varying_ref_wrapper,
};


// ================================================================
//  `cmp::Ordering`, `num::NonZero*`, `sync::atomic::*`
// ================================================================

concrete_types!(
    core::cmp::Ordering,
    core::num::NonZeroI8, core::num::NonZeroI16, core::num::NonZeroI32, core::num::NonZeroI64,
    core::num::NonZeroI128, core::num::NonZeroIsize,
    core::num::NonZeroU8, core::num::NonZeroU16, core::num::NonZeroU32, core::num::NonZeroU64,
    core::num::NonZeroU128, core::num::NonZeroUsize,
    core::sync::atomic::Ordering,
);

#[cfg(target_has_atomic = "8")]
concrete_types!(
    core::sync::atomic::AtomicBool,
    core::sync::atomic::AtomicI8,
    core::sync::atomic::AtomicU8,
);

#[cfg(target_has_atomic = "16")]
concrete_types!(core::sync::atomic::AtomicI16, core::sync::atomic::AtomicU16);

#[cfg(target_has_atomic = "32")]
concrete_types!(core::sync::atomic::AtomicI32, core::sync::atomic::AtomicU32);

#[cfg(target_has_atomic = "64")]
concrete_types!(core::sync::atomic::AtomicI64, core::sync::atomic::AtomicU64);

#[cfg(target_has_atomic = "ptr")]
unvarying! {
    impl<{T}> (Co+Contra)variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this atomic type
    // in `core`, `alloc`, `std`, or `variance-family`.
    for #[unsafe(not_a_foreign_fundamental_type)] core::sync::atomic::AtomicPtr<T>
}


// ================================================================
//  `cell::{OnceCell, LazyCell}`,
//  `mem::{ManuallyDrop, MaybeUninit}`, `ptr::NonNull`
// ================================================================

generic_wrapper! {
    impl<{#[unvarying] T (Is: Sized)}> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] core::cell::OnceCell<..>
}

generic_wrapper! {
    impl<{
        #[unvarying] T (Is: Sized),
        #[unvarying] F (Is: Sized),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] core::cell::LazyCell<..>
}

generic_wrapper! {
    impl<{
        // SAFETY: `ManuallyDrop<T>` is covariant over `T`.
        #[unsafe(covariant)] T,
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] core::mem::ManuallyDrop<..>
    where {T: ?Sized}
}

generic_wrapper! {
    impl<{
        // SAFETY: `MaybeUninit<T>` is covariant over `T`.
        #[unsafe(covariant)] T (Is: Sized),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] core::mem::MaybeUninit<..>
}

generic_wrapper! {
    impl<{
        // SAFETY: `NonNull<T>` is covariant over `T`.
        #[unsafe(covariant)] T,
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] core::ptr::NonNull<..>
    where {T: ?Sized}
}


// ================================================================
//  `slice::Iter<'a, T>`
// ================================================================

generic_wrapper! {
    impl<{
        'a,
        // SAFETY: `slice::Iter<'a, T>` is covariant over `T`.
        #[unsafe(covariant)] T (Is: Sized + 'a),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] core::slice::Iter<..>
}


// ================================================================
//  `slice::Iter<'varying, T<'varying>>` (VaryingSliceIter<T>)
// ================================================================

/// The `slice::Iter<'varying, T>` lifetime family.
///
/// If `T<'varying>` is covariant over `'varying`, then `slice::Iter<'varying, T<'varying>>` is
/// covariant over `'varying`.
///
/// This lifetime family is never contravariant over `'varying`.
///
/// Note that this type itself is just a marker ZST for the family.
///
/// # Limitations
/// Due to current limitations of the trait solver, `VaryingSliceIter<T>` requires that
/// `T<'varying>` outlives the `'upper` bound, instead of requiring that `T<'varying>` outlives
/// `'varying`.
///
/// As a result, if you want a more complicated lifetime family, you may have to define your own
/// lifetime family type instead of composing `VaryingSliceIter<_>` with other lifetime family
/// types.
pub struct VaryingSliceIter<T>(PhantomData<fn() -> T>);

phantom_zst_methods!(impl<{T}> _ for VaryingSliceIter<{T}>);

varying_ref_wrapper! {
    impl<T (Is: Sized)> CovariantFamily<'_, _>
    // SAFETY: `VaryingSliceIter` is defined in this crate.
    for #[unsafe(not_a_foreign_fundamental_type)] VaryingSliceIter<T>
    // SAFETY: `cell::slice::Iter<'varying, T>` is covariant over both `'varying` and `T`.
    as core::slice::Iter<#[unsafe(covariant)] '_, #[unsafe(covariant)] T>
}


// ================================================================
//  `slice::IterMut<'varying, T<'varying>>` (VaryingSliceIterMut<T>)
// ================================================================

/// The `slice::IterMut<'varying, T<'varying>>` lifetime family.
///
/// If `T<'varying>` does not actually use `'varying` at all (making it some fixed type `U`
/// regardless of `'varying`), then `slice::IterMut<'varying, T<'varying>>` is covariant over
/// `'varying`.
///
/// This lifetime family is never contravariant over `'varying`.
///
/// Note that this type itself is just a marker ZST for the family.
///
/// # Limitations
/// Due to current limitations of the trait solver, `VaryingSliceIterMut<T>` requires that
/// `T<'varying>` outlives the `'upper` bound, instead of requiring that `T<'varying>` outlives
/// `'varying`.
///
/// As a result, if you want a more complicated lifetime family, you may have to define your own
/// lifetime family type instead of composing `VaryingSliceIterMut<_>` with other lifetime family
/// types.
pub struct VaryingSliceIterMut<T>(PhantomData<fn() -> T>);

phantom_zst_methods!(impl<{T}> _ for VaryingSliceIterMut<{T}>);

varying_ref_mut_wrapper! {
    impl<T (Is: Sized)> (Co+Contra)variantFamily<'_, _>
    // SAFETY: `VaryingSliceIterMut` is defined in this crate.
    for #[unsafe(not_a_foreign_fundamental_type)] VaryingSliceIterMut<T>
    as core::slice::IterMut<'_, #[unvarying] T>
}
