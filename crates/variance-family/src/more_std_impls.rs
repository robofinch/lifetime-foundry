//! With the `more-impls` feature, also:
//!
//! - `collections::{HashMap, HashSet}`,
//! - `io::Cursor`,
//! - `sync::{OnceLock, RwLock, RwLock{Read, Write}Guard, LazyLock}`.

#![expect(unsafe_code, reason = "assert variance and permission to impl traits for `stc` types")]

use core::marker::PhantomData;

use crate::{generic_wrapper, phantom_zst_methods, varying_ref_wrapper, varying_ref_mut_wrapper};


// ================================================================
//  `collections::{HashMap, HashSet}`, `io::Cursor`
// ================================================================

generic_wrapper! {
    impl<{
        // SAFETY: `HashMap<K, V, S>` is covariant over `K`.
        #[unsafe(covariant)] K (Is: Sized),
        // SAFETY: `HashMap<K, V, S>` is covariant over `V`.
        #[unsafe(covariant)] V (Is: Sized),
        // SAFETY: `HashMap<K, V, S>` is covariant over `S`.
        #[unsafe(covariant)] S (Is: Sized),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `std`.
    for #[unsafe(not_a_foreign_fundamental_type)] std::collections::HashMap<..>
}

generic_wrapper! {
    impl<{
        // SAFETY: `HashSet<T, S>` is covariant over `T`.
        #[unsafe(covariant)] T (Is: Sized),
        // SAFETY: `HashSet<T, S>` is covariant over `S`.
        #[unsafe(covariant)] S (Is: Sized),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `std`.
    for #[unsafe(not_a_foreign_fundamental_type)] std::collections::HashSet<..>
}

// TODO(core_io): someday, this should be moved to `more_core_impls`.
#[allow(
    clippy::std_instead_of_core,
    reason = "on nightly, `Cursor` can be in `core`, which can trigger this lint",
)]
const _: () = {
    generic_wrapper! {
        impl<{
            // SAFETY: `io::Cursor<T>` is covariant over `T`.
            #[unsafe(covariant)] T (Is: Sized),
        }> ([Co] + [Contra])variantFamily<'_, _>
        // SAFETY: `variance-family` is allowed to implement traits for this type in `std`.
        for #[unsafe(not_a_foreign_fundamental_type)] std::io::Cursor<..>
    }
};


// ================================================================
//  `sync::{OnceLock, RwLock, LazyLock}`
// ================================================================

generic_wrapper! {
    impl<{#[unvarying] T (Is: Sized)}> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `std`.
    for #[unsafe(not_a_foreign_fundamental_type)] std::sync::OnceLock<..>
}

generic_wrapper! {
    impl<{#[unvarying] T}> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `std`.
    for #[unsafe(not_a_foreign_fundamental_type)] std::sync::RwLock<..>
    where {T: ?Sized}
}

generic_wrapper! {
    impl<{
        #[unvarying] T (Is: Sized),
        #[unvarying] F (Is: Sized),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `std`.
    for #[unsafe(not_a_foreign_fundamental_type)] std::sync::LazyLock<..>
}


// ================================================================
//  `RwLockReadGuard<'a, T>`
// ================================================================

generic_wrapper! {
    impl<{
        'a,
        // SAFETY: `RwLockReadGuard<'a, T>` is covariant over `T``.
        #[unsafe(covariant)] T (Is: 'a),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `std`.
    for #[unsafe(not_a_foreign_fundamental_type)] std::sync::RwLockReadGuard<..>
    where {T: ?Sized}
}


// ================================================================
//  `RwLockReadGuard<'varying, T<'varying>>` (VaryingRwLockReadGuard<T>)
// ================================================================

/// The `RwLockReadGuard<'varying, T<'varying>>` lifetime family.
///
/// If `T<'varying>` is covariant over `'varying`, then `RwLockReadGuard<'varying, T<'varying>>` is
/// covariant over `'varying`.
///
/// This lifetime family is never contravariant over `'varying`.
///
/// Note that this type itself is just a marker ZST for the family.
///
/// # Limitations
/// Due to current limitations of the trait solver, `VaryingRwLockReadGuard<T>` requires that
/// `T<'varying>` outlives the `'upper` bound, instead of requiring that `T<'varying>` outlives
/// `'varying`.
///
/// As a result, if you want a more complicated lifetime family, you may have to define your own
/// lifetime family type instead of composing `VaryingRwLockReadGuard<_>` with other lifetime
/// family types.
pub struct VaryingRwLockReadGuard<T: ?Sized>(PhantomData<fn() -> T>);

phantom_zst_methods!(impl<{T: ?Sized}> _ for VaryingRwLockReadGuard<{T}>);

varying_ref_wrapper! {
    impl<T> CovariantFamily<'_, _>
    // SAFETY: `VaryingRwLockReadGuard` is defined in this crate.
    for #[unsafe(not_a_foreign_fundamental_type)] VaryingRwLockReadGuard<T>
    // SAFETY: `RwLockReadGuard<'varying, T>` is covariant over both `'varying` and `T`.
    as std::sync::RwLockReadGuard<#[unsafe(covariant)] '_, #[unsafe(covariant)] T>
    where {T: ?Sized}
}


// ================================================================
//  `RwLockWriteGuard<'a, T>`
// ================================================================

generic_wrapper! {
    impl<{'a, #[unvarying] T (Is: 'a)}> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `std`.
    for #[unsafe(not_a_foreign_fundamental_type)] std::sync::RwLockWriteGuard<..>
    where {T: ?Sized}
}


// ================================================================
//  `RwLockWriteGuard<'varying, T<'varying>>` (VaryingRwLockWriteGuard<T>)
// ================================================================

/// The `RwLockWriteGuard<'varying, T<'varying>>` lifetime family.
///
/// If `T<'varying>` does not actually use `'varying` at all (making it some fixed type `U`
/// regardless of `'varying`), then `RwLockWriteGuard<'varying, T<'varying>>` is covariant over
/// `'varying`.
///
/// This lifetime family is never contravariant over `'varying`.
///
/// Note that this type itself is just a marker ZST for the family.
///
/// # Limitations
/// Due to current limitations of the trait solver, `VaryingRwLockWriteGuard<T>` requires that
/// `T<'varying>` outlives the `'upper` bound, instead of requiring that `T<'varying>` outlives
/// `'varying`.
///
/// As a result, if you want a more complicated lifetime family, you may have to define your own
/// lifetime family type instead of composing `VaryingRwLockWriteGuard<_>` with other lifetime
/// family types.
pub struct VaryingRwLockWriteGuard<T: ?Sized>(PhantomData<fn(*mut T)>);

phantom_zst_methods!(impl<{T: ?Sized}> _ for VaryingRwLockWriteGuard<{T}>);

varying_ref_mut_wrapper! {
    impl<T> (Co+Contra)variantFamily<'_, _>
    // SAFETY: `VaryingRwLockWriteGuard` is defined in this crate.
    for #[unsafe(not_a_foreign_fundamental_type)] VaryingRwLockWriteGuard<T>
    as std::sync::RwLockWriteGuard<'_, #[unvarying] T>
    where {T: ?Sized}
}
