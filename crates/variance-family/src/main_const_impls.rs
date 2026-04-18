//! Implementations for `&'a T`, `&'varying T` (as [`VaryingRef<T>`]), and `*const T`.

#![expect(unsafe_code, reason = "assert variance and permission to impl traits for `core` types")]

use core::marker::PhantomData;

use crate::{generic_wrapper, phantom_zst_methods, varying_ref_wrapper};


// ================================================================
//  &'a T
// ================================================================

/// Get `&'a T` into the standard shape expected by `generic_wrapper`.
type Ref<'a, T> = &'a T;

generic_wrapper! {
    impl<{
        'a,
        // SAFETY: `&'a T` is covariant over `T``.
        #[unsafe(covariant)] T (Is: 'a),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] Ref<..>
    where {T: ?Sized}
}


// ================================================================
//  &'varying T    (VaryingRef<T>)
// ================================================================

/// The `&'varying T<'varying>` lifetime family.
///
/// If `T<'varying>` is covariant over `'varying`, then `&'varying T<'varying>` is covariant
/// over `'varying`.
///
/// This lifetime family is never contravariant over `'varying`.
///
/// Note that this type itself is just a marker ZST for the family.
///
/// # Limitations
/// Due to current limitations of the trait solver, `VaryingRef<T>` requires that
/// `T<'varying>` outlives the `'upper` bound, instead of requiring that `T<'varying>` outlives
/// `'varying`.
///
/// As a result, if you want a more complicated lifetime family
/// (such as `&'varying Cow<'varying, [u8]>`),
/// you may have to define your own lifetime family type instead of composing `VaryingRef<_>` with
/// other lifetime family types.
pub struct VaryingRef<T: ?Sized>(PhantomData<fn() -> T>);

phantom_zst_methods!(impl<{T: ?Sized}> _ for VaryingRef<{T}>);

varying_ref_wrapper! {
    impl<T> CovariantFamily<'_, _>
    // SAFETY: `VaryingRef` is defined in this crate.
    for #[unsafe(not_a_foreign_fundamental_type)] VaryingRef<T>
    // SAFETY: `&'varying T` is covariant over both `'varying` and `T`.
    as Ref<#[unsafe(covariant)] '_, #[unsafe(covariant)] T>
    where {T: ?Sized}
}


// ================================================================
//  *const T
// ================================================================

/// Get `*const T` into the standard shape expected by `generic_wrapper`.
type Const<T> = *const T;

generic_wrapper! {
    // SAFETY: `*const T` is covariant over `T``.
    impl<{#[unsafe(covariant)] T}> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] Const<..>
    where {T: ?Sized}
}
