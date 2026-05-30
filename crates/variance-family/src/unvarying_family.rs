//! An [`Unvarying`] type that implements [`UnvaryingFamily`],
//! greatly useful for trivial families not implemented here.
//!
//! [`UnvaryingFamily`]: crate::traits::UnvaryingFamily

#![expect(unsafe_code, reason = "assert that `Unvarying` is local to this crate")]

use core::marker::PhantomData;

use crate::{phantom_zst_methods, unvarying};


/// The `T` lifetime family (as in, a lifetime family which does not use the `'varying` parameter).
///
/// This type implements [`UnvaryingFamily`] and the other variance family traits by
/// ignoring the `'varying` parameter and always outputting `T`.
///
/// Note that this type itself is just a marker ZST for the family.
///
/// [`UnvaryingFamily`]: crate::traits::UnvaryingFamily
pub struct Unvarying<T: ?Sized>(PhantomData<fn() -> T>);

phantom_zst_methods!(impl<{T: ?Sized}> _ for Unvarying<{T}>);

unvarying! {
    impl<{T: ?Sized,}> (Co+Contra)variantFamily<'_, _>
    // SAFETY: `Unvarying` is defined in this crate.
    for #[unsafe(not_a_foreign_fundamental_type)] Unvarying<T> as T
}
