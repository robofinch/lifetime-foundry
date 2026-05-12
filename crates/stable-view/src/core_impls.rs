//! Implementations for:
//!
//! - `[T; N]`,
//! - `(T1, ..., Tn)` of arities 0-6,
//! - `&T`,
//! - `&mut T`,
//! - `cell::{Ref, RefMut}`,
//! - `convert::Infallible`,
//! - `option::Option`,
//! - `pin::Pin<&T>`,
//! - `result::Result`.
//!
//! With the `more-impls` feature, also:
//!
//! - `(T1, ..., Tn)` of arities 7-12.

#![expect(unsafe_code, reason = "implement the unsafe `aliasable-view` traits")]
#![expect(clippy::absolute_paths, reason = "one-off uses of many different types")]
#![warn(clippy::missing_inline_in_public_items, reason = "trivial impls")]

use core::mem::transmute;

use variance_family::{Unvarying, VaryingRef, VaryingRefMut};

use crate::recursive_view;
use crate::{
    traits::{StableClone, StableView, StableViewMut},
    view_kinds::{
        PointerViewKind, SetDefaultView, SetDefaultViewMut, UnstableViewKind, ZeroSizedViewKind,
    },
};


// TODO: If there's any demand, consider making helper types for implementing
// `RecursiveViewKind` for `StableView<'_, '_, [T]>`.

// ================================================================
//  `[T; N]`
// ================================================================

type Array<const N: usize, T> = [T; N];

recursive_view! {
    Variadics = [
        (T, V, map),
    ];

    Default = true;

    // SAFETY:
    // The view components of a `[T; N]` are the `N` values of type `T`.
    //
    // - The view components of a `[T; N]` value are not wrapped in interior mutability.
    // - The view components of the clone of a `[T; N]` value are precisely the clones of
    //   each view component in the source `[T; N]` value. All source view components have at least
    //   one clone in the output, and each view component in the output is a clone.
    // - Any view components returned from `map` and `map_mut` are produced by applying the
    //   given `map` function to a view component of the source `self` value.
    unsafe impl<.., {const N: usize}> MapView<..> for Array<{N}, ..> {
        fn map<..>(this: &Self, ..) -> _ where .. {
            this.each_ref().map(map)
        }

        fn map_mut<..>(this: &mut Self, ..) -> _ where .. {
            this.each_mut().map(map)
        }
    }
}


// ================================================================
//  `(T1, .., T12)` for arities 1..12
// ================================================================

macro_rules! aliasable_tuple {
    (
        $t_last:ident $v_last:ident $map_last:ident $index_last:tt $(,)?
        $($t:ident $v:ident $map:ident $index:tt),* $(,)?
    ) => {
        const _: () = {
            type Tuple<$($t,)* $t_last> = ($($t,)* $t_last,);

            recursive_view! {
                Variadics = [
                    $(
                        ($t, $v, $map),
                    )*
                    ($t_last, $v_last, $map_last),
                ];

                Default = true;

                // SAFETY:
                // The view components of a `(T1, .., Tn)` are the `n` values of types
                // `T1, .., Tn`,
                //
                // - The view components of a tuple are not wrapped in interior mutability.
                // - The view components of the clone of a tuple are precisely the clones of
                //   each view component in the source tuple. All source view components have at
                //   least one clone in the output, and each view component in the output is a
                //   clone.
                // - Any view components returned from `map` and `map_mut` are produced by
                //   applying the given `Ti` map function to a view component of the source `self`
                //   value.
                unsafe impl<..> MapView<..> for Tuple<..>
                where {$t_last: ?Sized}
                {
                    fn map<..>(this: &Self, ..) -> _ where .. {
                        (
                            $(
                                $map(&this.$index),
                            )*
                            $map_last(&this.$index_last),
                        )
                    }

                    fn map_mut<..>(this: &mut Self, ..) -> _ where .. {
                        (
                            $(
                                $map(&mut this.$index),
                            )*
                            $map_last(&mut this.$index_last),
                        )
                    }
                }
            }
        };
    };
}

aliasable_tuple!(T1 V1 map_1 0);
aliasable_tuple!(T2 V2 map_2 1, T1 V1 map_1 0);
aliasable_tuple!(T3 V3 map_3 2, T1 V1 map_1 0, T2 V2 map_2 1);
aliasable_tuple!(T4 V4 map_4 3, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2);
aliasable_tuple!(T5 V5 map_5 4, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2, T4 V4 map_4 3);
aliasable_tuple!(
    T6 V6 map_6 5, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2, T4 V4 map_4 3, T5 V5 map_5 4,
);

#[cfg(feature = "more-impls")]
const _: () = {
    aliasable_tuple!(
        T7 V7 map_7 6, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2, T4 V4 map_4 3, T5 V5 map_5 4,
        T6 V6 map_6 5,
    );
    aliasable_tuple!(
        T8 V8 map_8 7, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2, T4 V4 map_4 3, T5 V5 map_5 4,
        T6 V6 map_6 5, T7 V7 map_7 6,
    );
    aliasable_tuple!(
        T9 V9 map_9 8, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2, T4 V4 map_4 3, T5 V5 map_5 4,
        T6 V6 map_6 5, T7 V7 map_7 6, T8 V8 map_8 7,
    );
    aliasable_tuple!(
        T10 V10 map_10 9, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2, T4 V4 map_4 3, T5 V5 map_5 4,
        T6 V6 map_6 5, T7 V7 map_7 6, T8 V8 map_8 7, T9 V9 map_9 8,
    );
    aliasable_tuple!(
        T11 V11 map_11 10, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2, T4 V4 map_4 3,
        T5 V5 map_5 4, T6 V6 map_6 5, T7 V7 map_7 6, T8 V8 map_8 7, T9 V9 map_9 8, T10 V10 map_10 9,
    );
    aliasable_tuple!(
        T12 V12 map_12 11, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2, T4 V4 map_4 3,
        T5 V5 map_5 4, T6 V6 map_6 5, T7 V7 map_7 6, T8 V8 map_8 7, T9 V9 map_9 8, T10 V10 map_10 9,
        T11 V11 map_11 10,
    );
};


// ================================================================
//  `()`
// ================================================================

type Unit = ();

// For consistency with other tuples.
recursive_view! {
    Variadics = [];

    Default = false;

    // SAFETY:
    // Essentially, `()` has no data, so its `()` "views" cannot be invalidated no matter what we
    // do.
    //
    // All of the zero view components in a `()` value have types from `T1, .., Tn` where `n=0`.
    //
    // - All zero view components of the unit type are not wrapped in interior mutability.
    // - All zero view components of the clone of a unit tuple are precisely the clones of
    //   each view component in the source unit tuple. All zero source view components have at
    //   least one clone in the output, and each of the zero view components in the output is a
    //   clone.
    // - Any of the zero view components returned from `map` and `map_mut` are produced by
    //   applying the given `Ti` map function to a view component of the source `self`
    //   value, where `1 <= i <= n == 0`.
    unsafe impl<..> MapView<..> for Unit<..> {
        fn map<..>(_this: &Self, ..) -> _ where .. {
            ()
        }

        fn map_mut<..>(_this: &mut Self, ..) -> _ where .. {
            ()
        }
    }
}

// SAFETY: The view type contains no data which could be invalidated by the three operations.
unsafe impl<'a, 'other_data> StableView<'a, 'other_data, ()> for ZeroSizedViewKind {
    type View = ();

    #[inline]
    unsafe fn view<'stable>(_data: &'a ())
    where
        'other_data: 'stable,
        'stable: 'a,
    {}
}

impl SetDefaultView<'_, '_> for () {
    type Default = ZeroSizedViewKind;
}

// SAFETY: The mutable view type contains no data which could be invalidated by the three
// operations.
unsafe impl<'a, 'other_data> StableViewMut<'a, 'other_data, ()> for ZeroSizedViewKind {
    type ViewMut = ();

    #[inline]
    unsafe fn view_mut<'stable>(_data: &'a mut ())
    where
        'other_data: 'stable,
        'stable: 'a,
    {}
}

impl SetDefaultViewMut<'_, '_> for () {
    type DefaultMut = ZeroSizedViewKind;
}

// SAFETY: The below definition trivially satisfies the first and second requirements.
// The third requirement is trivially satisfied because the view type contains no data, so it
// cannot ever be invalidated.
//
/// # Robust Guarantee
/// The definition of conceptual pool associated with the data type `()` and view kind
/// `ZeroSizedViewKind` is that every value is always in one pool, which is always nonempty.
unsafe impl StableClone<'_, '_, ()> for ZeroSizedViewKind {}


// ================================================================
//  `&T`
// ================================================================

// SAFETY:
// Until the source value's `'b` lifetime parameter expires, we know that *no* sound operation
// (whether immutable, mutable, a move, a drop, a coercion, whatever) can invalidate immutable
// references to the `T` referent derived from the source value.
// Indeed, `view` doesn't even need `unsafe` in its impl.
unsafe impl<'a, 'b, 'other_data, T> StableView<'a, 'other_data, &'b T> for PointerViewKind
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a &'b T) -> &'stable T
    where
        'other_data: 'stable,
        'stable: 'a,
    {
        data
    }
}

impl<'b, 'other_data, T> SetDefaultView<'_, 'other_data> for &'b T
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type Default = PointerViewKind;
}

// SAFETY:
// Until the source value's `'b` lifetime parameter expires, we know that *no* sound operation
// (whether immutable, mutable, a move, a drop, a coercion, whatever) can invalidate immutable
// references to the `T` referent derived from the source value.
//
// We can say that the pool conceptually consists of all valid immutable references and pointers to
// the referent of `self` *plus* some extra phantom pointer to the referent that is always in the
// pool... for at least lifetime `'b`. In other words, the conceptual pool is always nonempty
// for at least lifetime `'b` (and what happens after that does not matter).
//
/// # Robust Guarantee
/// The conceptual pool associated with [`PointerViewKind`] and `&'b T` is guaranteed to be
/// nonempty for at least lifetime `'b`, but may be emptied after `'b` ends.
unsafe impl<'b, 'other_data, T> StableClone<'_, 'other_data, &'b T> for PointerViewKind
where
    'b: 'other_data,
    T: ?Sized + 'b,
{}


// ================================================================
//  `&mut T`
// ================================================================

impl<'a, 'b, T> SetDefaultView<'a, '_> for &'b mut T
where
    'b: 'a,
    T: ?Sized + 'b,
{
    type Default = UnstableViewKind;
}

impl<'a, 'b, T> SetDefaultViewMut<'a, '_> for &'b mut T
where
    'b: 'a,
    T: ?Sized + 'b,
{
    type DefaultMut = UnstableViewKind;
}


// ================================================================
//  `cell::Ref`
// ================================================================

// SAFETY:
// Until the source value's `'b` lifetime parameter expires *or* the source value is dropped
// and releases its shared borrow rights over the referent, we know that:
// - Moving the `Ref` does not invalidate immutable references to its `T` referent, since the `T`
//   referent is stored elsewhere (in a `RefCell`) and `cell::Ref` is expected to alias other
//   shared references (and thus does not assert `noalias` when moved),
// - Coercing the `Ref` does not invalidate its `T` referent for the same reasons above and below,
// - No sound immutable operation on the `Ref` can invalidate shared references to the `T` referent;
//   otherwise, that could invalidate other references obtained from other shared
//   references to the same `Ref`.
//
// (After the `'b` lifetime parameter expires, the source `RefCell` could be dropped,
// and that's fine.)
unsafe impl<'a, 'b, 'other_data, T> StableView<'a, 'other_data, core::cell::Ref<'b, T>>
for PointerViewKind
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a core::cell::Ref<'b, T>) -> &'stable T
    where
        'other_data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a T = data;

        // SAFETY: See the "`transmute` in `view` Implementation" section of the `StableView` docs.
        unsafe {
            transmute::<
                &'a T,
                &'stable T,
            >(stable_eq_a)
        }
    }
}

impl<'b, 'other_data, T> SetDefaultView<'_, 'other_data> for core::cell::Ref<'b, T>
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type Default = PointerViewKind;
}


// ================================================================
//  `cell::RefMut`
// ================================================================

// SAFETY:
// Until the source value's `'b` lifetime parameter expires *or* the source value is dropped
// and releases its exclusive borrow rights over the referent, we know that:
// - Moving the `RefMut` does not invalidate immutable references to its `T` referent, because a
//   `RefMut` argument doesn't hold exclusivity for its whole scope, only until it drops; therefore,
//   it cannot have `Box`'s `noalias` semantics.
// - Coercing the `RefMut` does not invalidate its `T` referent for the same reason above.
// - No sound immutable operation on the `RefMut` can invalidate shared references to the `T`
//   referent; otherwise, that could invalidate other references obtained from other shared
//   references to the same `RefMut`.
//
// (After the `'b` lifetime parameter expires, the source `RefCell` could be dropped,
// and that's fine.)
unsafe impl<'a, 'b, 'other_data, T> StableView<'a, 'other_data, core::cell::RefMut<'b, T>>
for PointerViewKind
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a core::cell::RefMut<'_, T>) -> &'stable T
    where
        'other_data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a T = data;

        // SAFETY: See the "`transmute` in `view` Implementation" section of the `StableView` docs.
        unsafe {
            transmute::<
                &'a T,
                &'stable T,
            >(stable_eq_a)
        }
    }
}

impl<'b, 'other_data, T> SetDefaultView<'_, 'other_data> for core::cell::RefMut<'b, T>
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type Default = PointerViewKind;
}

// SAFETY:
// Until the source value's `'b` lifetime parameter expires *or* the source value is dropped
// and releases its exclusive borrow rights over the referent, we know that:
// - Moving the `RefMut` does not invalidate immutable references to its `T` referent, because a
//   `RefMut` argument doesn't hold exclusivity for its whole scope, only until it drops; therefore,
//   it cannot have `Box`'s `noalias` semantics.
// - Coercing the `RefMut` does not invalidate its `T` referent for the same reason above, noting
//   that the referent is not stored inline.
// - No-ops on the source data value are fine.
//
// (After the `'b` lifetime parameter expires, the source `RefCell` could be dropped,
// and that's fine.)
unsafe impl<'a, 'b, 'other_data, T> StableViewMut<'a, 'other_data, core::cell::RefMut<'b, T>>
for PointerViewKind
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type ViewMut = VaryingRefMut<Unvarying<T>>;

    #[inline]
    unsafe fn view_mut<'stable>(data: &'a mut core::cell::RefMut<'b, T>) -> &'stable mut T
    where
        'other_data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a mut T = data;

        // SAFETY: See the "`transmute` in `view_mut` Implementation" section of the
        // `StableViewMut` docs.
        unsafe {
            transmute::<
                &'a mut T,
                &'stable mut T,
            >(stable_eq_a)
        }
    }
}

impl<'b, 'other_data, T> SetDefaultViewMut<'_, 'other_data> for core::cell::RefMut<'b, T>
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type DefaultMut = PointerViewKind;
}


// ================================================================
//  `convert::Infallible`
// ================================================================

// SAFETY: The view type contains no data which could be invalidated by the three operations.
unsafe impl<'a, 'other_data> StableView<'a, 'other_data, core::convert::Infallible>
for ZeroSizedViewKind
{
    type View = core::convert::Infallible;

    #[inline]
    unsafe fn view<'stable>(data: &'a core::convert::Infallible) -> core::convert::Infallible
    where
        'other_data: 'stable,
        'stable: 'a,
    {
        #[expect(clippy::uninhabited_references, reason = "yeah, this function is unreachable")]
        *data
    }
}

impl SetDefaultView<'_, '_> for core::convert::Infallible {
    type Default = ZeroSizedViewKind;
}

// SAFETY: The mutable view type contains no data which could be invalidated by the three
// operations.
unsafe impl<'a, 'other_data> StableViewMut<'a, 'other_data, core::convert::Infallible>
for ZeroSizedViewKind
{
    type ViewMut = core::convert::Infallible;

    #[inline]
    unsafe fn view_mut<'stable>(
        data: &'a mut core::convert::Infallible,
    ) -> core::convert::Infallible
    where
        'other_data: 'stable,
        'stable: 'a,
    {
        #[expect(clippy::uninhabited_references, reason = "yeah, this function is unreachable")]
        *data
    }
}

impl SetDefaultViewMut<'_, '_> for core::convert::Infallible {
    type DefaultMut = ZeroSizedViewKind;
}

// SAFETY: The below definition trivially satisfies the first and second requirements.
// The third requirement is trivially satisfied because the view type contains no data, so it
// cannot ever be invalidated.
//
/// # Robust Guarantee
/// The definition of conceptual pool associated with the data type `Infallible` and view kind
/// `ZeroSizedViewKind` is that every value is always in one pool, which is always nonempty.
unsafe impl StableClone<'_, '_, core::convert::Infallible> for ZeroSizedViewKind {}


// ================================================================
//  `option::Option`
// ================================================================

recursive_view! {
    Variadics = [
        (T, V, map),
    ];

    Default = true;

    // SAFETY:
    // The view components of an `Option<T>` are the `0` or `1` values of type `T`.
    //
    // - The view components of an `Option<T>` value are stored inline with no interior mutability.
    // - The view components of the clone of an `Option<T>` value are precisely the clones of
    //   each view component in the source `Option<T>` value. All source view components have at
    //   least one clone in the output, and each view component in the output is a clone.
    // - Any view components returned from `map` and `map_mut` are produced by applying the
    //   given `map` function to a view component of the source `self` value.
    unsafe impl<..> MapView<..> for Option<..> {
        fn map<..>(this: &Self, ..) -> _ where .. {
            this.as_ref().map(map)
        }

        fn map_mut<..>(this: &mut Self, ..) -> _ where .. {
            this.as_mut().map(map)
        }
    }
}


// ================================================================
//  `pin::Pin<&T>`
// ================================================================

// SAFETY: We treat `core::pin::Pin<&T>` the same as `&T`. See `&T`'s implementation above
// for why this is sound. (The impl of `view` doesn't even use `unsafe`.)
unsafe impl<'a, 'b, 'other_data, T> StableView<'a, 'other_data, core::pin::Pin<&'b T>>
for PointerViewKind
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a core::pin::Pin<&'b T>) -> &'stable T
    where
        'other_data: 'stable,
        'stable: 'a,
    {
        data.get_ref()
    }
}

impl<'b, 'other_data, T> SetDefaultView<'_, 'other_data> for core::pin::Pin<&'b T>
where
    'b: 'other_data,
    T: ?Sized + 'b,
{
    type Default = PointerViewKind;
}

// SAFETY: We treat `core::pin::Pin<&T>` the same as `&T`. See `&T`'s implementation above
// for why this is sound.
//
/// # Robust Guarantee
/// The conceptual pool associated with [`PointerViewKind`] and `Pin<&'b T>` is guaranteed to be
/// nonempty for at least lifetime `'b`, but may be emptied after `'b` ends.
unsafe impl<'b, 'other_data, T> StableClone<'_, 'other_data, core::pin::Pin<&'b T>>
for PointerViewKind
where
    'b: 'other_data,
    T: ?Sized + 'b,
{}


// ================================================================
//  `result::Result`
// ================================================================

recursive_view! {
    Variadics = [
        (T, VT, map_ok),
        (E, VE, map_err),
    ];

    Default = true;

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
    unsafe impl<..> MapView<..> for Result<..> {
        fn map<..>(this: &Self, ..) -> _ where .. {
            this.as_ref().map(map_ok).map_err(map_err)
        }

        fn map_mut<..>(this: &mut Self, ..) -> _ where .. {
            this.as_mut().map(map_ok).map_err(map_err)
        }
    }
}
