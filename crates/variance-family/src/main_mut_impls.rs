//! Implementations for `&'a mut T`, `&'varying mut T` (as [`VaryingRefMut<T>`]), and `*mut T`.

#![expect(unsafe_code, reason = "assert variance and permission to impl traits for `core` types")]

use core::marker::PhantomData;

use crate::{generic_wrapper, phantom_zst_methods, varying_ref_mut_wrapper};


// ================================================================
//  &'a mut T
// ================================================================

/// Get `&'a mut T` into the standard shape expected by `generic_wrapper`.
type RefMut<'a, T> = &'a mut T;

generic_wrapper! {
    impl<{'a, #[unvarying] T (Is: 'a)}> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] RefMut<..>
    where {T: ?Sized}
}


// ================================================================
//  &'varying mut T    (VaryingRefMut<T>)
// ================================================================

/// The `&'varying mut T<'varying>` lifetime family.
///
/// If `T<'varying>` does not actually use `'varying` at all (making it some fixed type `U`
/// regardless of `'varying`), then `&'varying mut T<'varying>` is covariant over `'varying`.
///
/// This lifetime family is never contravariant over `'varying`.
///
/// Note that this type itself is just a marker ZST for the family.
///
/// # Limitations
/// Due to current limitations of the trait solver, `VaryingRefMut<T>` requires that
/// `T<'varying>` outlives the `'upper` bound, instead of requiring that `T<'varying>` outlives
/// `'varying`.
///
/// As a result, if you want a more complicated lifetime family
/// (such as `&'varying mut Cow<'varying, [u8]>`),
/// you may have to define your own lifetime family type instead of composing
/// `VaryingRefMut<_>` with other lifetime family types.
pub struct VaryingRefMut<T: ?Sized>(PhantomData<fn(*mut T)>);

phantom_zst_methods!(impl<{T: ?Sized}> _ for VaryingRefMut<{T}>);

varying_ref_mut_wrapper! {
    impl<T> (Co+Contra)variantFamily<'_, _>
    // SAFETY: `VaryingRefMut` is defined in this crate.
    for #[unsafe(not_a_foreign_fundamental_type)] VaryingRefMut<T>
    as RefMut<'_, #[unvarying] T>
    where {T: ?Sized}
}


// ================================================================
//  *mut T
// ================================================================

/// Get `*mut T` into the standard shape expected by `generic_wrapper`.
type Mut<T> = *mut T;

generic_wrapper! {
    impl<{#[unvarying] T}> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] Mut<..>
    where {T: ?Sized}
}
