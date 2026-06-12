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

#![expect(unsafe_code, reason = "implement the unsafe `stable-view` traits")]
#![expect(clippy::absolute_paths, reason = "one-off uses of many different types")]
#![warn(clippy::missing_inline_in_public_items, reason = "trivial impls")]

use core::mem::transmute;

use variance_family::{Unvarying, VaryingRef, VaryingRefMut};

use crate::recursive_view;
use crate::provided_view_kinds::UnstableViewKind;
use crate::{
    traits::{StableClone, StableView, StableViewMut},
    view_kinds::{DefaultStableView, DefaultStableViewMut, ReferenceViewKind, ZeroSizedViewKind},
};


// TODO: If there's any demand, consider making helper types for implementing
// `StableView<'_, '_, [T]>` for `RecursiveViewKind`.

// ================================================================
//  `[T; N]`
// ================================================================

/// Get `[T; N]` into a shape usable with [`recursive_view`].
type Array<const N: usize, T> = [T; N];

recursive_view! {
    Variadics = [
        (T, V, map),
    ];

    Default = true;
    StableClone = true;

    // SAFETY:
    // The view components of a `[T; N]` are the `N` values of type `T`.
    //
    // - The view components of a `[T; N]` value are not wrapped in interior mutability within
    //   the array.
    // - The view components of the clone of a `[T; N]` value are precisely the clones of
    //   each view component in the source `[T; N]` value. All source view components have at least
    //   one clone in the output, and each view component in the output is a clone.
    // - Stable data cannot reference the inline data of a `[T; N]` and must therefore come from
    //   a view component, regardless of view kind used.
    // - Any view components returned from `map` and `map_mut` are produced by applying the
    //   given `map` function to a view component of the source `self` value.
    unsafe impl<.., {const N: usize,}> MapView<..> for Array<{N}, ..> {
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

/// Utility for reducing repetition. Calling this macro should be entirely safe.
macro_rules! stable_tuple {
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
                StableClone = true;

                // SAFETY:
                // The view components of a `(T1, .., Tn)` are the `n` values of types
                // `T1, .., Tn`,
                //
                // - The view components of a tuple are not wrapped in interior mutability within
                //   the tuple.
                // - The view components of the clone of a tuple are precisely the clones of
                //   each view component in the source tuple. All source view components have at
                //   least one clone in the output, and each view component in the output is a
                //   clone.
                // - Stable data cannot reference the inline data of a tuple and must therefore come
                //   from a view component, regardless of view kind used.
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

stable_tuple!(T1 V1 map_1 0);
stable_tuple!(T2 V2 map_2 1, T1 V1 map_1 0);
stable_tuple!(T3 V3 map_3 2, T1 V1 map_1 0, T2 V2 map_2 1);
stable_tuple!(T4 V4 map_4 3, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2);
stable_tuple!(T5 V5 map_5 4, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2, T4 V4 map_4 3);
stable_tuple!(
    T6 V6 map_6 5, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2, T4 V4 map_4 3, T5 V5 map_5 4,
);

#[cfg(feature = "more-impls")]
const _: () = {
    stable_tuple!(
        T7 V7 map_7 6, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2, T4 V4 map_4 3, T5 V5 map_5 4,
        T6 V6 map_6 5,
    );
    stable_tuple!(
        T8 V8 map_8 7, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2, T4 V4 map_4 3, T5 V5 map_5 4,
        T6 V6 map_6 5, T7 V7 map_7 6,
    );
    stable_tuple!(
        T9 V9 map_9 8, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2, T4 V4 map_4 3, T5 V5 map_5 4,
        T6 V6 map_6 5, T7 V7 map_7 6, T8 V8 map_8 7,
    );
    stable_tuple!(
        T10 V10 map_10 9, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2, T4 V4 map_4 3, T5 V5 map_5 4,
        T6 V6 map_6 5, T7 V7 map_7 6, T8 V8 map_8 7, T9 V9 map_9 8,
    );
    stable_tuple!(
        T11 V11 map_11 10, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2, T4 V4 map_4 3,
        T5 V5 map_5 4, T6 V6 map_6 5, T7 V7 map_7 6, T8 V8 map_8 7, T9 V9 map_9 8, T10 V10 map_10 9,
    );
    stable_tuple!(
        T12 V12 map_12 11, T1 V1 map_1 0, T2 V2 map_2 1, T3 V3 map_3 2, T4 V4 map_4 3,
        T5 V5 map_5 4, T6 V6 map_6 5, T7 V7 map_7 6, T8 V8 map_8 7, T9 V9 map_9 8, T10 V10 map_10 9,
        T11 V11 map_11 10,
    );
};


// ================================================================
//  `()`
// ================================================================

/// Get `()` into a shape usable with [`recursive_view`].
type Unit = ();

// For consistency with other tuples.
recursive_view! {
    Variadics = [];

    Default = false;
    StableClone = true;

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
    // - Stable data cannot reference the inline data of a `()`, of which there is none.
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

impl<'a, 'data> StableView<'a, 'data, ()> for ZeroSizedViewKind {
    type View = ();

    #[inline]
    unsafe fn view<'stable>(_data: &'a ())
    where
        'data: 'stable,
        'stable: 'a,
    {}
}

impl DefaultStableView<'_, '_> for () {
    type Default = ZeroSizedViewKind;
}

impl<'a, 'data> StableViewMut<'a, 'data, ()> for ZeroSizedViewKind {
    type ViewMut = ();

    #[inline]
    unsafe fn view_mut<'stable>(_data: &'a mut ())
    where
        'data: 'stable,
        'stable: 'a,
    {}
}

impl DefaultStableViewMut<'_, '_> for () {
    type DefaultMut = ZeroSizedViewKind;
}


// ================================================================
//  `&T`
// ================================================================

impl<'a, 'b, 'data, T> StableView<'a, 'data, &'b T> for ReferenceViewKind
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a &'b T) -> &'stable T
    where
        'data: 'stable,
        'stable: 'a,
    {
        data
    }
}

impl<'b, 'data, T> DefaultStableView<'_, 'data> for &'b T
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type Default = ReferenceViewKind;
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
/// The conceptual pool associated with `&'b T` is guaranteed to be nonempty for at least lifetime
/// `'b`, but may be emptied after `'b` ends.
unsafe impl<'b, 'data, T> StableClone<'data> for &'b T
where
    'b: 'data,
    T: ?Sized + 'b,
{}


// ================================================================
//  `&mut T`
// ================================================================

/// Used to put `&'b mut T` into the format required by `recursive_view`.
type RefMut<'b, T> = &'b mut T;

/// Used to trivially pass through a view family in the syntax required by `recursive_view`.
type TrivialFamily<T> = T;

// Note that a `&mut String` can provide a `&'stable str`, a `&mut &mut &mut &mut Rc<T>` can
// provide a `&'stable T`, and so on. This may feel risky *at first*, but `&mut T` (or a `noalias`
// `Box<T>`) can genuinely be treated equivalent to a moved-around `T` as far as `'stable` views go.
//
// Note that the below safety comment doesn't rely on this flimsy reasoning. This discussion is just
// intended to indicate that the `recursive_view` macro's safety conditions aren't too weak.
recursive_view! {
    Variadics = [
        (T, V, map),
    ];

    Default = false;
    StableClone = false;

    // SAFETY:
    // The view component of an `&'b mut T` is the value of type `T`.
    //
    // - The view components of a `&'b mut T` value are are not wrapped in interior
    //   mutability within `&'b mut T`.
    // - `StableClone` is not set to `true`.
    // - `StableClone` is not set to `true`.
    // - Any view components returned from `map` and `map_mut` are produced by applying the
    //   given `map` function to a view component of the source `self` value.
    unsafe impl<.., {'b,}> MapView<..> for RefMut<{'b}, ..>
    where {T: ?Sized + 'b}
    {
        type WithParamsFamily<..> = TrivialFamily<..>;

        fn map<..>(this: &Self, ..) -> _ where .. {
            map(this)
        }

        fn map_mut<..>(this: &mut Self, ..) -> _ where .. {
            map(this)
        }
    }
}

impl<'a, 'b, T> DefaultStableView<'a, '_> for &'b mut T
where
    'b: 'a,
    T: ?Sized + 'b,
{
    type Default = UnstableViewKind;
}

impl<'a, 'b, T> DefaultStableViewMut<'a, '_> for &'b mut T
where
    'b: 'a,
    T: ?Sized + 'b,
{
    type DefaultMut = UnstableViewKind;
}


// ================================================================
//  `cell::Ref`
// ================================================================

impl<'a, 'b, 'data, T> StableView<'a, 'data, core::cell::Ref<'b, T>> for ReferenceViewKind
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a core::cell::Ref<'b, T>) -> &'stable T
    where
        'data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a T = data;

        // SAFETY: See "`transmute` in `view(_mut)` Implementation" in the `StableView::view` docs.
        // Until the source value's `'b` lifetime parameter expires *or* the source value is dropped
        // and releases its shared borrow rights over the referent, we know that:
        // - Moving the `Ref` does not invalidate immutable references to its `T` referent, since
        //   the `T` referent is stored elsewhere (in a `RefCell`) and `cell::Ref` is expected to
        //   alias other shared references (and thus does not assert `noalias` when moved),
        // - Coercing the `Ref` (via non-deref Rust 1.85 coercions), which only reads/writes/moves
        //   inline data, does not invalidate its `T` referent for the same reason above,
        // - No sound immutable operation on the `Ref` can invalidate shared references to the `T`
        //   referent; otherwise, that could invalidate other references obtained from other shared
        //   references to the same `Ref`.
        //
        // The same applies to any `Ref<'_, U>` which this type could coerce to.
        //
        // (After the `'b` lifetime parameter expires, the source `RefCell` could be dropped,
        // and that's fine.)
        unsafe {
            transmute::<
                &'a T,
                &'stable T,
            >(stable_eq_a)
        }
    }
}

impl<'b, 'data, T> DefaultStableView<'_, 'data> for core::cell::Ref<'b, T>
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type Default = ReferenceViewKind;
}


// ================================================================
//  `cell::RefMut`
// ================================================================

impl<'a, 'b, 'data, T> StableView<'a, 'data, core::cell::RefMut<'b, T>> for ReferenceViewKind
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a core::cell::RefMut<'_, T>) -> &'stable T
    where
        'data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a T = data;

        // SAFETY: See "`transmute` in `view(_mut)` Implementation" in the `StableView::view` docs.
        // Until the source value's `'b` lifetime parameter expires *or* the source value is dropped
        // and releases its exclusive borrow rights over the referent, we know that:
        // - Moving the `RefMut` does not invalidate mutable references to its `T` referent,
        //   because a `RefMut` argument doesn't hold exclusivity for its whole scope, only until
        //   it drops; therefore, it cannot have `Box`'s `noalias` semantics.
        // - Coercing the `RefMut` via permitted coercions, which only reads/writes/moves inline
        //   data, does not invalidate its `T` referent for the same reason above.
        // - No sound immutable operation on the `RefMut` can invalidate shared references to the
        //   `T` referent; otherwise, that could invalidate other references obtained from other
        //   shared references to the same `RefMut`.
        //
        // The same applies to any `RefMut<'_, U>` which this type could coerce to.
        //
        // (After the `'b` lifetime parameter expires, the source `RefCell` could be dropped,
        // and that's fine.)
        unsafe {
            transmute::<
                &'a T,
                &'stable T,
            >(stable_eq_a)
        }
    }
}

impl<'b, 'data, T> DefaultStableView<'_, 'data> for core::cell::RefMut<'b, T>
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type Default = ReferenceViewKind;
}

impl<'a, 'b, 'data, T> StableViewMut<'a, 'data, core::cell::RefMut<'b, T>> for ReferenceViewKind
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type ViewMut = VaryingRefMut<Unvarying<T>>;

    #[inline]
    unsafe fn view_mut<'stable>(data: &'a mut core::cell::RefMut<'b, T>) -> &'stable mut T
    where
        'data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a mut T = data;

        // SAFETY: See "`transmute` in `view(_mut)` Implementation" in the `StableView::view` docs.
        // Until the source value's `'b` lifetime parameter expires *or* the source value is dropped
        // and releases its exclusive borrow rights over the referent, we know that:
        // - Moving the `RefMut` does not invalidate immutable references to its `T` referent,
        //   because a `RefMut` argument doesn't hold exclusivity for its whole scope, only until it
        //   drops; therefore, it cannot have `&'b mut T`'s `noalias` semantics.
        // - Coercing the `RefMut` via permitted coercions, which only reads/writes/moves inline
        //   data, does not invalidate its `T` referent for the same reason above.
        // - No-ops on the source data value are fine.
        //
        // The same applies to any `RefMut<'_, U>` which this type could coerce to.
        //
        // (After the `'b` lifetime parameter expires, the source `RefCell` could be dropped,
        // and that's fine.)
        unsafe {
            transmute::<
                &'a mut T,
                &'stable mut T,
            >(stable_eq_a)
        }
    }
}

impl<'b, 'data, T> DefaultStableViewMut<'_, 'data> for core::cell::RefMut<'b, T>
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type DefaultMut = ReferenceViewKind;
}


// ================================================================
//  `convert::Infallible`
// ================================================================

impl<'a, 'data> StableView<'a, 'data, core::convert::Infallible> for ZeroSizedViewKind {
    type View = core::convert::Infallible;

    #[inline]
    unsafe fn view<'stable>(&data: &'a core::convert::Infallible) -> core::convert::Infallible
    where
        'data: 'stable,
        'stable: 'a,
    {
        data
    }
}

impl DefaultStableView<'_, '_> for core::convert::Infallible {
    type Default = ZeroSizedViewKind;
}

impl<'a, 'data> StableViewMut<'a, 'data, core::convert::Infallible> for ZeroSizedViewKind {
    type ViewMut = core::convert::Infallible;

    #[inline]
    unsafe fn view_mut<'stable>(
        &mut data: &'a mut core::convert::Infallible,
    ) -> core::convert::Infallible
    where
        'data: 'stable,
        'stable: 'a,
    {
        data
    }
}

impl DefaultStableViewMut<'_, '_> for core::convert::Infallible {
    type DefaultMut = ZeroSizedViewKind;
}

// SAFETY: The below definition trivially satisfies the first, second, and third requirements.
// The fourth requirement is trivially satisfied because the view type contains no data, so it
// cannot ever be invalidated.
//
/// # Robust Guarantee
/// The definition of conceptual pool associated with the data type `Infallible` is that every
/// value is always in one pool, which is always nonempty.
unsafe impl StableClone<'_> for core::convert::Infallible {}


// ================================================================
//  `option::Option`
// ================================================================

recursive_view! {
    Variadics = [
        (T, V, map),
    ];

    Default = true;
    StableClone = true;

    // SAFETY:
    // The view components of an `Option<T>` are the `0` or `1` values of type `T`.
    //
    // - The view components of an `Option<T>` value are are not wrapped in interior
    //   mutability within `Option`.
    // - The view components of the clone of an `Option<T>` value are precisely the clones of
    //   each view component in the source `Option<T>` value. All source view components have at
    //   least one clone in the output, and each view component in the output is a clone.
    // - Stable data cannot reference the inline data of an `Option<T>` and must therefore come from
    //   a view component, regardless of view kind used.
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

impl<'a, 'b, 'data, T> StableView<'a, 'data, core::pin::Pin<&'b T>> for ReferenceViewKind
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a core::pin::Pin<&'b T>) -> &'stable T
    where
        'data: 'stable,
        'stable: 'a,
    {
        data.get_ref()
    }
}

impl<'b, 'data, T> DefaultStableView<'_, 'data> for core::pin::Pin<&'b T>
where
    'b: 'data,
    T: ?Sized + 'b,
{
    type Default = ReferenceViewKind;
}

// SAFETY: We treat `core::pin::Pin<&T>` the same as `&T`. See `&T`'s implementation above
// for why this is sound.
//
/// # Robust Guarantee
/// The conceptual pool associated with `Pin<&'b T>` is guaranteed to be
/// nonempty for at least lifetime `'b`, but may be emptied after `'b` ends.
unsafe impl<'b, 'data, T> StableClone<'data> for core::pin::Pin<&'b T>
where
    'b: 'data,
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
    StableClone = true;

    // SAFETY:
    // The view components of a `Result<T, E>` are the `1` or `0` values of type `T`
    // and the `0` or `1` values of type `E`.
    //
    // - The view components of an `Result<T, E>` value are are not wrapped in interior
    //   mutability within `Result`.
    // - The view components of the clone of a `Result<T, E>` value are precisely the clones of
    //   each view component in the source `Result<T, E>` value. All source view components have at
    //   least one clone in the output, and each view component in the output is a clone.
    // - Stable data cannot reference the inline data of a `Result<T, E>` and must therefore come
    //   from a view component, regardless of view kind used.
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
