//! If the `either` feature is enabled, implementations for:
//!
//! - `either::Either`.

#![expect(unsafe_code, reason = "implement the unsafe `aliasable-view` traits")]
#![warn(clippy::missing_inline_in_public_items, reason = "trivial impls")]

use either::Either;

use variance_family::EitherFamily;

use crate::recursive_view;


// ================================================================
//  `either::Either`
// ================================================================

recursive_view! {
    Variadics = [
        (L, VL, map_left),
        (R, VR, map_right),
    ];

    Default = true;

    // SAFETY:
    // The view components of an `Either<L, R>` are the `1` or `0` values of type `L`
    // and the `0` or `1` values of type `R`.
    //
    // - The view components of an `Either<L, R>` value are stored inline with no interior
    //   mutability.
    // - The view components of the clone of a `Either<L, R>` value are precisely the clones of
    //   each view component in the source `Either<L, R>` value. All source view components have at
    //   least one clone in the output, and each view component in the output is a clone.
    // - Any view components returned from `map` and `map_mut` are produced by applying the
    //   given `map_*` function to a view component of the source `self` value.
    unsafe impl<..> MapView<..> for Either<..> {
        type WithParamsFamily<..> = EitherFamily<..>;

        fn map<..>(this: &Self, ..) -> _ where .. {
            this.as_ref().map_left(map_left).map_right(map_right)
        }

        fn map_mut<..>(this: &mut Self, ..) -> _ where .. {
            this.as_mut().map_left(map_left).map_right(map_right)
        }
    }
}
