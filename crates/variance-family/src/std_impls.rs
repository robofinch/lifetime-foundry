//! Implementations for:
//!
//! - `path::{Path, PathBuf}`,
//! - `sync::{Mutex, MutexGuard}`.

#![expect(unsafe_code, reason = "assert variance and permission to impl traits for `stc` types")]
#![expect(clippy::absolute_paths, reason = "one-off uses of many different types")]

use core::marker::PhantomData;

use crate::{generic_wrapper, macros::concrete_types, phantom_zst_methods, varying_ref_mut_wrapper};


// ================================================================
//  `path::{Path, PathBuf}`
// ================================================================

concrete_types!(std::path::Path, std::path::PathBuf);

// ================================================================
//  `sync::Mutex`
// ================================================================

generic_wrapper! {
    impl<{#[unvarying] T}> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `std`.
    for #[unsafe(not_a_foreign_fundamental_type)] std::sync::Mutex<..>
    where {T: ?Sized}
}


// ================================================================
//  `MutexGuard<'a, T>`
// ================================================================

generic_wrapper! {
    impl<{'a, #[unvarying] T (Is: 'a)}> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `std`.
    for #[unsafe(not_a_foreign_fundamental_type)] std::sync::MutexGuard<..>
    where {T: ?Sized}
}


// ================================================================
//  `MutexGuard<'varying, T<'varying>>` (VaryingMutexGuard<T>)
// ================================================================

/// The `MutexGuard<'varying, T<'varying>>` lifetime family.
///
/// If `T<'varying>` does not actually use `'varying` at all (making it some fixed type `U`
/// regardless of `'varying`), then `MutexGuard<'varying, T<'varying>>` is covariant over
/// `'varying`.
///
/// This lifetime family is never contravariant over `'varying`.
///
/// Note that this type itself is just a marker ZST for the family.
///
/// # Limitations
/// Due to current limitations of the trait solver, `VaryingMutexGuard<T>` requires that
/// `T<'varying>` outlives the `'upper` bound, instead of requiring that `T<'varying>` outlives
/// `'varying`.
///
/// As a result, if you want a more complicated lifetime family, you may have to define your own
/// lifetime family type instead of composing `VaryingMutexGuard<_>` with other lifetime family
/// types.
pub struct VaryingMutexGuard<T: ?Sized>(PhantomData<fn(*mut T)>);

phantom_zst_methods!(impl<{T: ?Sized}> _ for VaryingMutexGuard<{T}>);

varying_ref_mut_wrapper! {
    impl<T> (Co+Contra)variantFamily<'_, _>
    // SAFETY: `VaryingMutexGuard` is defined in this crate.
    for #[unsafe(not_a_foreign_fundamental_type)] VaryingMutexGuard<T>
    as std::sync::MutexGuard<'_, #[unvarying] T>
    where {T: ?Sized}
}
