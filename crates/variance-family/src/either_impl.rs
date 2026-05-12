//! If the `either` feature is enabled, implementations for:
//!
//! - `either::Either`.
//!
//! (A `EitherFamily` helper type is needed.)

#![expect(unsafe_code, reason = "assert variance of `Either` and permission to impl a local type")]

use core::marker::PhantomData;

use either::Either;

use crate::{generic_wrapper, phantom_zst_methods};


// ================================================================
//  `EitherFamily`
// ================================================================

/// The <code>[Either]<L<'varying>, R<'varying>></code> lifetime family, covariant over both of its
/// parameters.
///
/// This lifetime family is covariant over `'varying` iff both its parameters are covariant
/// over `'varying`, and contravariant iff both are contravariant over `'varying`.
///
/// Note that this type itself is just a marker ZST for the family.
pub struct EitherFamily<L, R>(PhantomData<fn() -> (L, R)>);

phantom_zst_methods!(impl<{L, R}> _ for EitherFamily<{L, R}>);

generic_wrapper! {
    impl<{
        // SAFETY: `Either<L, R>` is covariant over `T`.
        #[unsafe(covariant)] T (Is: Sized),
        // SAFETY: `Either<L, R>` is covariant over `E`.
        #[unsafe(covariant)] E (Is: Sized),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `EitherFamily` is defined in this crate.
    for #[unsafe(not_a_foreign_fundamental_type)] EitherFamily<..>
    as Either<..>
}
