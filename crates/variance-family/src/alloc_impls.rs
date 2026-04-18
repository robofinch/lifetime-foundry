//! Implementations for:
//!
//! - `boxed::Box`,
//! - `borrow::Cow`,
//! - `rc::Rc`,
//! - `string::String`,
//! - `sync::Arc`,
//! - `vec::Vec`,

#![expect(unsafe_code, reason = "assert variance and permission to impl traits for `alloc` types")]
#![expect(clippy::absolute_paths, reason = "one-off uses of many different types")]

use core::marker::PhantomData;

use crate::{generic_wrapper, macros::concrete_types, phantom_zst_methods, varying_ref_mut_wrapper};


// ================================================================
//  `string::String`
// ================================================================

concrete_types!(alloc::string::String);

// ================================================================
//  `boxed::Box`, `rc::Rc`, `sync::Arc`, `vec::Vec`
// ================================================================

generic_wrapper! {
    impl<{
        // SAFETY: `Box<T>` is covariant over `T`.
        #[unsafe(covariant)] T,
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `alloc`.
    for #[unsafe(not_a_foreign_fundamental_type)] alloc::boxed::Box<..>
    where {T: ?Sized}
}

generic_wrapper! {
    impl<{
        // SAFETY: `Rc<T>` is covariant over `T`.
        #[unsafe(covariant)] T,
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `alloc`.
    for #[unsafe(not_a_foreign_fundamental_type)] alloc::rc::Rc<..>
    where {T: ?Sized}
}

generic_wrapper! {
    impl<{
        // SAFETY: `Arc<T>` is covariant over `T`.
        #[unsafe(covariant)] T,
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `alloc`.
    for #[unsafe(not_a_foreign_fundamental_type)] alloc::sync::Arc<..>
    where {T: ?Sized}
}

generic_wrapper! {
    impl<{
        // SAFETY: `Vec<T>` is covariant over `T`.
        #[unsafe(covariant)] T (Is: Sized),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `alloc`.
    for #[unsafe(not_a_foreign_fundamental_type)] alloc::vec::Vec<..>
}


// ================================================================
//  `Cow<'a, T>`
// ================================================================

// Unfortunately, `Cow<'a, T>` is invariant over `T`.
generic_wrapper! {
    impl<{
        'a,
        #[unvarying] T (Is: 'a + alloc::borrow::ToOwned),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] alloc::borrow::Cow<..>
    where {T: ?Sized + alloc::borrow::ToOwned}
}


// ================================================================
//  `Cow<'varying, T<'varying>>` (VaryingCow<T>)
// ================================================================

/// The `Cow<'varying, T<'varying>>` lifetime family.
///
/// Since `Cow<'a, T>` is, unfortunately, invariant over `T`, `Cow<'varying, T<'varying>>` is
/// covariant over `'varying` only when `T<'varying>` does not actually use `'varying` at all
/// (making `T<'varying>` some fixed type `U` regardless of `'varying`).
///
/// This lifetime family is never contravariant over `'varying`.
///
/// Note that this type itself is just a marker ZST for the family.
pub struct VaryingCow<T: ?Sized>(PhantomData<fn() -> T>);

phantom_zst_methods!(impl<{T: ?Sized}> _ for VaryingCow<{T}>);

varying_ref_mut_wrapper! {
    impl<T (Is: alloc::borrow::ToOwned)> (Co+Contra)variantFamily<'_, _>
    // SAFETY: `VaryingCow` is defined in this crate.
    for #[unsafe(not_a_foreign_fundamental_type)] VaryingCow<T>
    as alloc::borrow::Cow<'_, #[unvarying] T>
    where {T: ?Sized + alloc::borrow::ToOwned}
}
