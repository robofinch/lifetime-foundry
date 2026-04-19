//! Implementations for:
//!
//! - `[T; N]`,
//! - `(T1, ..., Tn)` of arities 0-6,
//! - `&T`,
//! - `cell::{Ref, RefMut}`,
//! - `convert::Infallible`,
//! - `option::Option`,
//! - `pin::Pin<&T>`,
//! - `result::Result`.
//!
//! With the `more-impls` feature, also:
//!
//! - `(T1, ..., Tn)` of arities 7-12,

#![expect(unsafe_code, reason = "implement the unsafe `aliasable-view` traits")]
#![expect(clippy::absolute_paths, reason = "one-off uses of many different types")]

use variance_family::{Unvarying, UpperBound, VaryingRef, VaryingRefMut};

use crate::map_aliasable;
use crate::traits::{
    AliasableClone, AliasableView, AliasableViewMut, IntoAliasable, IntoAliasableMut, View,
    ViewMut,
};


// ================================================================
//  `[T; N]`
// ================================================================

type Array<const N: usize, T> = [T; N];

map_aliasable! {
    Variadics = [
        (T, map),
    ];

    // SAFETY:
    // The view components of a `[T; N]` are the `N` values of type `T`.
    //
    // - The view components of a `[T; N]` value are stored inline with no interior mutability.
    // - The view components of the clone of a `[T; N]` value are precisely the clones of
    //   each view component in the source `[T; N]` value. All source view components have at least
    //   one clone in the output, and each view component in the output is a clone.
    // - Any view components returned from `map` and `map_mut` are produced by applying the
    //   given `map` function to a view component of the source `self` value.
    unsafe impl<.., {const N: usize}> MapAliasable<_> for Array<{N}, ..> {
        fn map<..>(&self, ..) -> _ where .. {
            self.each_ref().map(map)
        }

        fn map_mut<..>(&mut self, ..) -> _ where .. {
            self.each_mut().map(map)
        }
    }
}

// ================================================================
//  `(T1, .., T12)` for arities 1..12
// ================================================================

macro_rules! aliasable_tuple {
    (
        $t_last:ident $map_last:ident $index_last:tt $(,)?
        $($t:ident $map:ident $index:tt),* $(,)?
    ) => {
        const _: () = {
            type Tuple<$($t,)* $t_last> = ($($t,)* $t_last,);

            map_aliasable! {
                Variadics = [
                    $(
                        ($t, $map),
                    )*
                    ($t_last, $map_last),
                ];

                // SAFETY:
                // The view components of a `(T1, .., Tn)` are the `n` values of types
                // `T1, .., Tn`,
                //
                // - The view components of a tuple are stored inline with no interior mutability.
                // - The view components of the clone of a tuple are precisely the clones of
                //   each view component in the source tuple. All source view components have at
                //   least one clone in the output, and each view component in the output is a
                //   clone.
                // - Any view components returned from `map` and `map_mut` are produced by
                //   applying the given `Ti` map function to a view component of the source `self`
                //   value.
                unsafe impl<..> MapAliasable<_> for Tuple<..>
                where {$t_last: ?Sized}
                {
                    fn map<..>(&self, ..) -> _ where .. {
                        (
                            $(
                                $map(&self.$index),
                            )*
                            $map_last(&self.$index_last),
                        )
                    }

                    fn map_mut<..>(&mut self, ..) -> _ where .. {
                        (
                            $(
                                $map(&mut self.$index),
                            )*
                            $map_last(&mut self.$index_last),
                        )
                    }
                }
            }
        };
    };
}

aliasable_tuple!(T1 map_1 0);
aliasable_tuple!(T2 map_2 1, T1 map_1 0);
aliasable_tuple!(T3 map_3 2, T1 map_1 0, T2 map_2 1);
aliasable_tuple!(T4 map_4 3, T1 map_1 0, T2 map_2 1, T3 map_3 2);
aliasable_tuple!(T5 map_5 4, T1 map_1 0, T2 map_2 1, T3 map_3 2, T4 map_4 3);
aliasable_tuple!(T6 map_6 5, T1 map_1 0, T2 map_2 1, T3 map_3 2, T4 map_4 3, T5 map_5 4);

#[cfg(feature = "more-impls")]
const _: () = {
    aliasable_tuple!(
        T7 map_7 6, T1 map_1 0, T2 map_2 1, T3 map_3 2, T4 map_4 3, T5 map_5 4, T6 map_6 5,
    );
    aliasable_tuple!(
        T8 map_8 7, T1 map_1 0, T2 map_2 1, T3 map_3 2, T4 map_4 3, T5 map_5 4, T6 map_6 5,
        T7 map_7 6,
    );
    aliasable_tuple!(
        T9 map_9 8, T1 map_1 0, T2 map_2 1, T3 map_3 2, T4 map_4 3, T5 map_5 4, T6 map_6 5,
        T7 map_7 6, T8 map_8 7,
    );
    aliasable_tuple!(
        T10 map_10 9, T1 map_1 0, T2 map_2 1, T3 map_3 2, T4 map_4 3, T5 map_5 4, T6 map_6 5,
        T7 map_7 6, T8 map_8 7, T9 map_9 8,
    );
    aliasable_tuple!(
        T11 map_11 10, T1 map_1 0, T2 map_2 1, T3 map_3 2, T4 map_4 3, T5 map_5 4, T6 map_6 5,
        T7 map_7 6, T8 map_8 7, T9 map_9 8, T10 map_10 9,
    );
    aliasable_tuple!(
        T12 map_12 11, T1 map_1 0, T2 map_2 1, T3 map_3 2, T4 map_4 3, T5 map_5 4, T6 map_6 5,
        T7 map_7 6, T8 map_8 7, T9 map_9 8, T10 map_10 9, T11 map_11 10,
    );
};


// ================================================================
//  `()`
// ================================================================

type Unit = ();

map_aliasable! {
    Variadics = [];

    // SAFETY:
    // Essentially, `()` has no data, so its `()` "views" cannot be invalidated no matter what we
    // do.
    //
    // All of the zero view components in a `()` value have types from `T1, .., Tn` where `n=0`.
    //
    // - All zero view components of the unit type are stored inline with no interior mutability.
    // - All zero view components of the clone of a unit tuple are precisely the clones of
    //   each view component in the source unit tuple. All zero source view components have at
    //   least one clone in the output, and each of the zero view components in the output is a
    //   clone.
    // - Any of the zero view components returned from `map` and `map_mut` are produced by
    //   applying the given `Ti` map function to a view component of the source `self`
    //   value, where `1 <= i <= n == 0`.
    unsafe impl<..> MapAliasable<_> for Unit<..> {
        fn map<..>(&self, ..) -> _ where .. {
            ()
        }

        fn map_mut<..>(&mut self, ..) -> _ where .. {
            ()
        }
    }
}


// ================================================================
//  `&T`
// ================================================================

// SAFETY:
// Until the source value's `'_` lifetime parameter expires, we know that *no* sound operation
// (whether immutable, mutable, a move, a drop, a coercion, whatever) can invalidate immutable
// references to the `T` referent derived from the source value.
unsafe impl<'upper, T: ?Sized + 'upper> AliasableView<&'upper ()> for &T {
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    fn view(&self) -> View<'_, Self, &'upper ()> {
        self
    }
}

impl<'upper, T: ?Sized + 'upper> IntoAliasable<&'upper ()> for &T {
    type IntoAliasable = Self;

    #[inline]
    fn into_aliasable(self) -> Self::IntoAliasable {
        self
    }
}

// SAFETY:
// Until the source value's `'_` lifetime parameter expires, we know that *no* sound operation
// (whether immutable, mutable, a move, a drop, a coercion, whatever) can invalidate immutable
// references to the `T` referent derived from the source value.
//
// We can say that the pool conceptually consists of all valid immutable references and pointers to
// the referent of `self` *plus* some extra phantom pointer to the referent that is always in the
// pool... for at least lifetime `'_`. In other words, the conceptual pool is always nonempty
// for at least lifetime `'_` (and what happens after that does not matter).
unsafe impl<'upper, T: ?Sized + 'upper> AliasableClone<&'upper ()> for &T {}


// ================================================================
//  `cell::Ref`
// ================================================================

// SAFETY:
// Until the source value's `'_` lifetime parameter expires *or* the source value is dropped
// and releases its shared borrow rights over the referent, we know that:
// - Moving the `Ref` does not invalidate immutable references to its `T` referent, since the `T`
//   referent is stored elsewhere (in a `RefCell`) and `cell::Ref` is expected to alias other
//   shared references (and thus does not assert `noalias` when moved),
// - Coercing the `Ref` does not invalidate its `T` referent for the same reasons above and below,
// - No sound immutable operation on the `Ref` can invalidate shared references to the `T` referent;
//   otherwise, that could invalidate other references obtained from other shared
//   references to the same `Ref`.
//
// (After the `'_` lifetime parameter expires, the source `RefCell` could be dropped,
// and that's fine.)
unsafe impl<'upper, T: ?Sized + 'upper> AliasableView<&'upper ()> for core::cell::Ref<'_, T> {
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    fn view(&self) -> View<'_, Self, &'upper ()> {
        self
    }
}

impl<'upper, T: ?Sized + 'upper> IntoAliasable<&'upper ()> for core::cell::Ref<'_, T> {
    type IntoAliasable = Self;

    #[inline]
    fn into_aliasable(self) -> Self::IntoAliasable {
        self
    }
}


// ================================================================
//  `cell::RefMut`
// ================================================================

// SAFETY:
// Until the source value's `'_` lifetime parameter expires *or* the source value is dropped
// and releases its exclusive borrow rights over the referent, we know that:
// - Moving the `RefMut` does not invalidate immutable references to its `T` referent, because a
//   `RefMut` argument doesn't hold exclusivity for its whole scope, only until it drops; therefore,
//   it uses a `NonNull` and cannot have `Box`'s `noalias` semantics.
// - Coercing the `RefMut` does not invalidate its `T` referent for the same reason above.
// - No sound immutable operation on the `RefMut` can invalidate shared references to the `T`
//   referent; otherwise, that could invalidate other references obtained from other shared
//   references to the same `RefMut`.
//
// (After the `'_` lifetime parameter expires, the source `RefCell` could be dropped,
// and that's fine.)
unsafe impl<'upper, T: ?Sized + 'upper> AliasableView<&'upper ()> for core::cell::RefMut<'_, T> {
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    fn view(&self) -> View<'_, Self, &'upper ()> {
        self
    }
}

impl<'upper, T: ?Sized + 'upper> IntoAliasable<&'upper ()> for core::cell::RefMut<'_, T> {
    type IntoAliasable = Self;

    #[inline]
    fn into_aliasable(self) -> Self::IntoAliasable {
        self
    }
}

// SAFETY:
// Until the source value's `'_` lifetime parameter expires *or* the source value is dropped
// and releases its exclusive borrow rights over the referent, we know that:
// - Moving the `RefMut` does not invalidate immutable references to its `T` referent, because a
//   `RefMut` argument doesn't hold exclusivity for its whole scope, only until it drops; therefore,
//   it uses a `NonNull` and cannot have `Box`'s `noalias` semantics.
// - Coercing the `RefMut` does not invalidate its `T` referent for the same reason above, noting
//   that the referent is not stored inline.
//
// (After the `'_` lifetime parameter expires, the source `RefCell` could be dropped,
// and that's fine.)
unsafe impl<'upper, T: ?Sized + 'upper> AliasableViewMut<&'upper ()> for core::cell::RefMut<'_, T> {
    type ViewMut = VaryingRefMut<Unvarying<T>>;

    #[inline]
    fn view_mut(&mut self) -> ViewMut<'_, Self, &'upper ()> {
        self
    }
}

impl<'upper, T: ?Sized + 'upper> IntoAliasableMut<&'upper ()> for core::cell::RefMut<'_, T> {}


// ================================================================
//  `convert::Infallible`
// ================================================================

// SAFETY: No sound operation, whether that be moves, coercions, immutable operations, or otherwise,
// can invalidate references to views of this type... because there cannot soundly be any references
// to the uninhabited `Infallible` type.
unsafe impl<Upper: UpperBound> AliasableView<Upper> for core::convert::Infallible {
    type View = Self;

    #[inline]
    fn view(&self) -> View<'_, Self, Upper> {
        #[expect(clippy::uninhabited_references, reason = "indeed, this function is unreachable")]
        *self
    }
}

impl<Upper: UpperBound> IntoAliasable<Upper> for core::convert::Infallible {
    type IntoAliasable = Self;

    #[inline]
    fn into_aliasable(self) -> Self::IntoAliasable {
        self
    }
}

// SAFETY: No sound operation, whether that be moves, coercions, immutable operations, or otherwise,
// can invalidate references to views of this type... because there cannot soundly be any references
// to the uninhabited `Infallible` type.
unsafe impl<Upper: UpperBound> AliasableViewMut<Upper> for core::convert::Infallible {
    type ViewMut = Self;

    #[inline]
    fn view_mut(&mut self) -> ViewMut<'_, Self, Upper> {
        #[expect(clippy::uninhabited_references, reason = "indeed, this function is unreachable")]
        *self
    }
}

impl<Upper: UpperBound> IntoAliasableMut<Upper> for core::convert::Infallible {}

// SAFETY: No sound operation, whether that be moves, coercions, immutable operations, or otherwise,
// can invalidate references to views of this type... because there cannot soundly be any references
// to the uninhabited `Infallible` type.
//
// We can say that the conceptual pool is always empty, or always infinitely full... it doesn't
// particularly matter, since the three requirements are vacuously satisfied either way.
unsafe impl<Upper: UpperBound> AliasableClone<Upper> for core::convert::Infallible {}


// ================================================================
//  `option::Option`
// ================================================================

map_aliasable! {
    Variadics = [
        (T, map),
    ];

    // SAFETY:
    // The view components of an `Option<T>` are the `0` or `1` values of type `T`.
    //
    // - The view components of an `Option<T>` value are stored inline with no interior mutability.
    // - The view components of the clone of an `Option<T>` value are precisely the clones of
    //   each view component in the source `Option<T>` value. All source view components have at
    //   least one clone in the output, and each view component in the output is a clone.
    // - Any view components returned from `map` and `map_mut` are produced by applying the
    //   given `map` function to a view component of the source `self` value.
    unsafe impl<..> MapAliasable<_> for Option<..> {
        fn map<..>(&self, ..) -> _ where .. {
            self.as_ref().map(map)
        }

        fn map_mut<..>(&mut self, ..) -> _ where .. {
            self.as_mut().map(map)
        }
    }
}


// ================================================================
//  `pin::Pin<&T>`
// ================================================================

// SAFETY: We treat `core::pin::Pin<&T>` the same as `&T`. See `&T`'s implementation above
// for why this is sound.
unsafe impl<'upper, T: ?Sized + 'upper> AliasableView<&'upper ()> for core::pin::Pin<&T> {
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    fn view(&self) -> View<'_, Self, &'upper ()> {
        self
    }
}

impl<'upper, T: ?Sized + 'upper> IntoAliasable<&'upper ()> for core::pin::Pin<&T> {
    type IntoAliasable = Self;

    #[inline]
    fn into_aliasable(self) -> Self::IntoAliasable {
        self
    }
}

// SAFETY: We treat `core::pin::Pin<&T>` the same as `&T`. See `&T`'s implementation above
// for why this is sound.
unsafe impl<'upper, T: ?Sized + 'upper> AliasableClone<&'upper ()> for core::pin::Pin<&T> {}


// ================================================================
//  `result::Result`
// ================================================================

map_aliasable! {
    Variadics = [
        (T, map_ok),
        (E, map_err),
    ];

    // SAFETY:
    // The view components of a `Result<T, E>` are the `1` or `0` values of type `T`
    // and the `0` or `1` values of type `E`.
    //
    // - The view components of an `Result<T, E>` value are stored inline with no interior
    //   mutability.
    // - The view components of the clone of a `Result<T, E>` value are precisely the clones of
    //   each view component in the source `Result<T, E>` value. All source view components have at
    //   least one clone in the output, and each view component in the output is a clone.
    // - Any view components returned from `map` and `map_mut` are produced by applying the
    //   given `map_*` function to a view component of the source `self` value.
    unsafe impl<..> MapAliasable<_> for Result<..> {
        fn map<..>(&self, ..) -> _ where .. {
            self.as_ref().map(map_ok).map_err(map_err)
        }

        fn map_mut<..>(&mut self, ..) -> _ where .. {
            self.as_mut().map(map_ok).map_err(map_err)
        }
    }
}
