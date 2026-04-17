//! Implementations for `&'a T`, `&'varying T` (as [`VaryingRef<T>`]), and `*const T`.

#![expect(unsafe_code, reason = "assert variance and permission to impl traits for `core` types")]

use core::marker::PhantomData;

use crate::{generic_wrapper, phantom_zst_methods, varying_ref_wrapper};


// Note: in below safety comments, "is covariant over" or "is contravariant over" means, more
// precisely, "is sound to covariantly (or contravariantly) cast with respect to". That is,
// manually-proven variance (and manually-proven soundness of casts) is the relevant concern,
// not compiler-assigned variance (and compiler-proven soundness of casts).

// ================================================================
//  &'a T
// ================================================================

// Safety summary:
// - `&'a T<'varying>` is covariant over `'varying` if `T<'varying>` is covariant over `'varying`.
// - `&'a T<'varying>` is contravariant over `'varying` if `T<'varying>` is contravariant over it.

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

// Safety summary:
// - `&'varying T<'varying>` is covariant over `'varying` if `T<'varying>` is covariant over it.
// - `&'varying T<'varying>` is never contravariant over `'varying`.

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

// Safety summary:
// - `*const T<'varying>` is covariant over `'varying` if `T<'varying>` is covariant over it.
// - `*const T<'varying>` is contravariant over it if `T<'varying>` is contravariant over it.

/// Get `*const T` into the standard shape expected by `generic_wrapper`.
type Const<T> = *const T;

generic_wrapper! {
    // SAFETY: `*const T` is covariant over `T``.
    impl<{#[unsafe(covariant)] T}> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] Const<..>
    where {T: ?Sized}
}
