//! Implementations of [`variance_family`] traits for the aliasable versions of `&'a mut T`,
//! `&'varying mut T`, and `Box<T>`.
//
// Currently, even though the macros emit `unsafe` tokens -- regardless of whether they're passed
// from the macros' callers (I tried) -- the lint is not triggered. I don't know exactly why, since
// the lints work within the `variance-family` crate.
// #![expect(unsafe_code, reason = "assert variance and that types are local to this crate")]

use core::marker::PhantomData;

use variance_family::{generic_wrapper, phantom_zst_methods, varying_ref_mut_wrapper};

use super::aliasable_ref_mut::AliasableRefMut;
#[cfg(feature = "alloc")]
use super::aliasable_box::AliasableBox;


generic_wrapper! {
    impl<{'a, #[unvarying] T (Is: 'a)}> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `AliasableRefMut` is defined in this crate.
    for #[unsafe(not_a_foreign_fundamental_type)] AliasableRefMut<..>
    where {T: ?Sized}
}

/// The `AliasableRefMut<'varying, T<'varying>>` lifetime family.
///
/// If `T<'varying>` does not actually use `'varying` at all (making it some fixed type `U`
/// regardless of `'varying`), then `AliasableRefMut<'varying, T<'varying>>` is covariant over
/// `'varying`.
///
/// This lifetime family is never contravariant over `'varying`.
///
/// Note that this type itself is just a marker ZST for the family.
///
/// # Limitations
/// Due to current limitations of the trait solver, `VaryingAliasableRefMut<T>` requires that
/// `T<'varying>` outlives the `'upper` bound, instead of requiring that `T<'varying>` outlives
/// `'varying`.
///
/// As a result, if you want a more complicated lifetime family
/// (such as `AliasableRefMut<'varying, Cow<'varying, [u8]>>`),
/// you may have to define your own lifetime family type instead of composing
/// `VaryingAliasableRefMut<_>` with other lifetime family types.
pub struct VaryingAliasableRefMut<T: ?Sized>(PhantomData<fn(*mut T)>);

phantom_zst_methods!(impl<{T: ?Sized}> _ for VaryingAliasableRefMut<{T}>);

varying_ref_mut_wrapper! {
    impl<T> (Co+Contra)variantFamily<'_, _>
    // SAFETY: `VaryingAliasableRefMut` is defined in this crate.
    for #[unsafe(not_a_foreign_fundamental_type)] VaryingAliasableRefMut<T>
    as AliasableRefMut<'_, #[unvarying] T>
    where {T: ?Sized}
}

#[cfg(feature = "alloc")]
generic_wrapper! {
    impl<{
        // SAFETY: `AliasableBox<T>` is covariant over `T`.
        #[unsafe(covariant)] T,
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `AliasableBox` is defined in this crate.
    for #[unsafe(not_a_foreign_fundamental_type)] AliasableBox<..>
    where {T: ?Sized}
}
