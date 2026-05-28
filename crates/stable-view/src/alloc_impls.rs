//! Implementations for:
//!
//! - `borrow::Cow`,
//! - `boxed::Box`,
//! - `rc::Rc`,
//! - `string::String`,
//! - `sync::Arc`,
//! - `vec::Vec<T>`.
//!
//! TODO, if it becomes possible:
//! - `collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque}`.

#![expect(unsafe_code, reason = "implement the unsafe `aliasable-view` traits")]
#![warn(clippy::missing_inline_in_public_items, reason = "trivial impls")]

use core::mem::transmute;
use alloc::{boxed::Box, rc::Rc, string::String, sync::Arc, vec::Vec};
use alloc::borrow::{Cow, ToOwned};

use variance_family::{Unvarying, Varying, VaryingRef, VaryingRefMut, WithLifetime};

use crate::recursive_view;
use crate::{
    traits::{StableClone, StableView, StableViewMut},
    view_kinds::{
        PointerViewKind, RecursiveViewKind, SetDefaultView, SetDefaultViewMut, UnstableViewKind,
    },
};


// ================================================================
//  `borrow::Cow`
// ================================================================

// SAFETY: We are essentially deferring to the `StableClone` impl for either `&'b B` or `B::Owned`.
// Note that `Cow<'b, B>: Clone` even when `B: ToOwned + !Clone`,
// but when `B: Clone`, the `Clone` impl of `Cow` either:
// - copies the `&'b B` in a `Cow::Borrowed`
//   (which is the same as `&'b B`'s impl of `Clone::clone`), or
// - `Clone::clone`s the `B::Owned` in a `Cow::Owned`.
//
// Note that the three requirements, when `B::Owned: StableClone<'_>`, are clearly satisfied in
// *either* the `Cow::Borrowed` or `Cow::Owned` cases. Additionally, operations done through `&`
// references to parts of a `Cow` value are *entirely incapable* of switching the owned/borrowed
// state of that `Cow` value, since `Cow` is an enum, and the enum discriminant is not internally
// mutable.
//
/// # Robust Guarantee
/// The conceptual pool associated with a `Cow::Borrowed(data)` value (where `data: &'b B`) is
/// guaranteed to be nonempty for at least lifetime `'b`, but may be emptied after `'b` ends.
///
/// The definition of conceptual pool associated with a `Cow::Owned(data)` value (where
/// `data: B::Owned`) is the conceptual pool definition used by the implementation
/// of `StableClone<'data>` for `B::Owned`. In other words, the conceptual pool
/// definition for `Cow::Owned` is simply deferred to `B::Owned`.
///
/// The above two cases cover the definition of conceptual pool used by any `Cow` value.
unsafe impl<'b, 'data, B> StableClone<'data> for Cow<'b, B>
where
    'b: 'data,
    B: 'b + ?Sized + ToOwned<Owned: StableClone<'data>>,
{}

// SAFETY: We basically have two separate cases (borrowed and owned). We know that the borrowed
// branch is `&'b B`, which can provide a `&'stable B` reference since `'b: 'data`.
// The crazy-complicated trait bounds require that the owned branch has a pointer view
// to `&'stable B`. We then just match the cases.
//
// By the implementation of a `StableView` for the owned branch, we know that the returned
// `&'stable B` in the owned branch is not invalidated by the three operations applied to the
// source `Cow::Owned(owned)` value (for at least the `'data` upper boudn). By the safety
// comment for the `StableView` impl of `&'b T` for `PointerViewKind`, the same holds for the
// borrowed branch, though it's simpler to use the `unsafe`-free shortening of `&'b B` to
// `&'stable B` instead of calling `view`.
unsafe impl<'a, 'b, 'data, B> StableView<'a, 'data, Cow<'b, B>> for PointerViewKind
where
    'b: 'data,
    B: 'b + ?Sized + ToOwned,
    Self: StableView<
        'a, 'data, B::Owned,
        View: for<'stable> WithLifetime<'stable, 'a, &'data (), Is = &'stable B>,
    >,
{
    type View = VaryingRef<Unvarying<B>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a Cow<'b, B>) -> &'stable B
    where
        'data: 'stable,
        'stable: 'a,
    {
        match data {
            Cow::Borrowed(borrowed) => borrowed,
            Cow::Owned(owned) => {
                // SAFETY: The returned view can only be used at a given time if, from just after
                // this function returns until the time of use, only the three operations are
                // performed, and if `'data` has not ended. This constraint is precisely what
                // *our* `view` caller unsafely asserts, so this is sound.
                // In other words, we have simply forwarded the safety preconditions to the caller.
                unsafe { <Self as StableView<'a, 'data, B::Owned>>::view(owned) }
            }
        }
    }
}

impl<'a, 'b, 'data, B> SetDefaultView<'a, 'data> for Cow<'b, B>
where
    'b: 'data,
    B: 'b + ?Sized + ToOwned,
    PointerViewKind: StableView<
        'a, 'data, B::Owned,
        View: for<'stable> WithLifetime<'stable, 'a, &'data (), Is = &'stable B>,
    >,
{
    type Default = PointerViewKind;
}

// SAFETY: We basically have two separate cases (borrowed and owned).
//
// In the borrowed case, by `VB`'s `StableView` impl for `&'b B`, we know that the views returned
// by its returned `view` are not invalidated by the three validations (for at least `'data`),
// and since our `view` simply returns their `view`, our impl is sound in that case.
//
// Likewise for the owned case, but `VB` becomes `VO` and `&'b B` becomes `B::Owned`.
unsafe impl<'a, 'b, 'data, B, VB, VO> StableView<'a, 'data, Cow<'b, B>>
for RecursiveViewKind<(VB, VO)>
where
    // Most comprehensible Rust where-bound.
    B: 'b + ?Sized + ToOwned,
    VB: StableView<'a, 'data, &'b B>,
    VO: StableView<
        'a, 'data, B::Owned,
        View: for<'stable> WithLifetime<
            'stable, 'a, &'data (),
            Is = Varying<'stable, 'a, &'data (), VB::View>,
        >,
    >,
{
    type View = VB::View;

    #[inline]
    unsafe fn view<'stable>(
        data: &'a Cow<'b, B>,
    ) -> Varying<'stable, 'a, &'data (), Self::View>
    where
        'data: 'stable,
        'stable: 'a,
    {
        match data {
            Cow::Borrowed(borrowed) => {
                // SAFETY: The returned view can only be used at a given time if, from just after
                // this function returns until the time of use, only the three operations are
                // performed, and if `'data` has not ended. This constraint is precisely what
                // *our* `view` caller unsafely asserts, so this is sound.
                // In other words, we have simply forwarded the safety preconditions to the caller.
                unsafe { VB::view(borrowed) }
            }
            Cow::Owned(owned) => {
                // SAFETY: Same as above branch.
                unsafe { VO::view(owned) }
            }
        }
    }
}


// ================================================================
//  `boxed::Box`
// ================================================================

/// Used to trivially pass through a view family in the syntax required by `recursive_view`.
type TrivialFamily<T> = T;

recursive_view! {
    Variadics = [
        (T, V, map),
    ];

    Default = false;
    StableClone = false;

    // SAFETY:
    // The view component of an `Box<T>` is the value of type `T`.
    //
    // - The view components of a `Box<T>` value are are not wrapped in interior
    //   mutability within `Box`.
    // - `StableClone` is not set to `true`.
    // - `StableClone` is not set to `true`.
    // - Any view components returned from `map` and `map_mut` are produced by applying the
    //   given `map` function to a view component of the source `self` value.
    unsafe impl<..> MapView<..> for Box<..>
    where {T: ?Sized}
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

impl<'a, T: ?Sized + 'a> SetDefaultView<'a, '_> for Box<T> {
    type Default = UnstableViewKind;
}

impl<'a, T: ?Sized + 'a> SetDefaultViewMut<'a, '_> for Box<T> {
    type DefaultMut = UnstableViewKind;
}


// ================================================================
//  `rc::Rc`
// ================================================================

// SAFETY: We will go through each of the three operations. The `'data` upper bound
// doesn't particularly matter in the case of the owned `Rc<T>` type.
//
// First, moves. Moving a `Rc<T>` necessarily does not invalidate references to its contents,
// since there could be other live `Rc<T>`s used to reference those contents. (If there's only
// one `Rc` left, then mutable methods can invalidate references to its contents, but moves
// cannot execute conditional logic.)
//
// Second, non-`DerefMut` coercions (among Rust 1.85 coercions). As noted by `StableView`, it
// should be covered by the first and third cases.
//
// Third, operations done to data derived from parts of `Rc<T>` only through `&` references. Since
// `Rc<T>` doesn't wrap its `T` contents in internal mutability (though its refcounts are
// internally mutable), operations done on shared references to part or all of a `Rc<T>` value
// cannot invalidate operations done on a shared reference to its `T` contents.
// Note that operations done on one `&Rc<T>` **CAN** invalidate a different `&Rc<T>`... if the
// latter was derived from an older `&mut Rc<T>` (or other `Unique`-tagged pointer) to the same
// `Rc<T>`. However, the provenance and permissions of the `&T` derive from the `Rc<T>`'s inner
// pointer, not from a `&mut Rc<T>` which may be used to access that inner pointer.
unsafe impl<'a, 'data, T: ?Sized + 'data> StableView<'a, 'data, Rc<T>>
for PointerViewKind
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a Rc<T>) -> &'stable T
    where
        'data: 'stable,
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

impl<'data, T: ?Sized + 'data> SetDefaultView<'_, 'data> for Rc<T> {
    type Default = PointerViewKind;
}

// SAFETY: We will go through each of the three operations. The `'data` upper bound
// doesn't particularly matter. As noted by `StableViewMut`, the second and third operations
// aren't particularly noteworthy either; our `view` impl doesn't do something strange that would
// break them while somehow still supporting the first operation.
//
// Then, moves. See above safety comment for `StableView` for `Rc<T>`.
unsafe impl<'a, 'data, T: ?Sized + 'data> StableViewMut<'a, 'data, Rc<T>>
for PointerViewKind
{
    type ViewMut = Option<VaryingRefMut<Unvarying<T>>>;

    /// Uses [`Rc::get_mut`].
    #[inline]
    unsafe fn view_mut<'stable>(data: &'a mut Rc<T>) -> Option<&'stable mut T>
    where
        'data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: Option<&'a mut T> = Rc::get_mut(data);

        // SAFETY: See the "`transmute` in `view_mut` Implementation" section of the
        // `StableViewMut` docs.
        unsafe {
            transmute::<
                Option<&'a mut T>,
                Option<&'stable mut T>,
            >(stable_eq_a)
        }
    }
}

impl<'data, T: ?Sized + 'data> SetDefaultViewMut<'_, 'data> for Rc<T> {
    type DefaultMut = PointerViewKind;
}

// SAFETY: We will go through each of the three requirements,
// under the definition given by the below robust guarantee.
//
// Requirement 1: A data value (`Rc<T>`) is in exactly one pool at all times; it uniquely owns
// exactly one strong ref to its allocation. (Other values might share a strong ref in interesting
// ways, or hold more than one strong ref of the same pool. Whatever. Irrelevant.) Cloning an
// `Rc<T>` yields another `Rc<T>` in the same pool, since it holds a new strong ref of the same
// `Rc` allocation and does not mutate the `T` value.
//
// Requirement 2: Moves, coercions, and operations on part or all of a data value through a `&`
// reference (performed in any quantity and order) can transform an `Rc<T>` into an `Rc<U>` pointing
// at the same `Rc` allocation but with different metadata (via unsizing coercions) and/or different
// lifetimes (via subtyping coercions), and so on. Still, the result at each step is an `Rc<U>` for
// some `U` (even if it's moved into the heap and erased into a `dyn Something`), and no move,
// coercion, or operation on a `Rc<U>` value can release its strong ref, since a live `Rc<U>` *must*
// hold a strong ref to an `Rc` allocation. Moreover, only `&mut Rc<U>` functions can change the
// `Rc`'s pointer, since the pointer is not internally mutable; this implies that the three
// operations cannot change which `Rc` allocation is referenced.
//
// Continuing this reasoning, since the value (not refcounts) of the `Rc` allocation can only
// be mutated by the owner of the last strong ref, and the source data value always has a strong
// ref (so long as only the three operations are performed on it), we thus know that nothing other
// than the source data value could mutate its value (and then, only if nothing else holds a strong
// or weak ref). However, since `Rc` doesn't have a lock or cell wrapping its value, only `&mut Rc`
// methods can mutate the value in the `Rc` allocation; since the three operations do not allow
// for mutable access to the whole source `Rc` data value through a `&mut` reference, it follows
// that the three operations leave the source `Rc` value with a strong ref to the same `Rc`
// allocation and do not mutate its value. Therefore, the three operations leave the data value
// in the same pool.
//
// Requirement 3: If the source pool of a view of an `Rc<T>` is nonempty, then the `Rc` allocation
// has a nonzero strong count and its `T` value (not refcounts) has not been mutated since the view
// was taken. Therefore, `&` references to that `T` value have not been invalidated by the `Rc`
// allocation being deallocated (which can only happen after the strong count reaches zero) or by
// mutations.
//
// See also `String` and `Vec` about how the provenance of the `&Rc<T>` used to obtain a
// `&'stable T` (or a `'stable` borrow of part of the `T`) does not matter; only the provenance of
// the inner pointer of the `Rc<T>` affects the created `&'stable T` (or smaller `'stable` borrow),
// and that inner pointer has sufficient provenance for e.g. the guarantees made by `Rc::as_ptr`.
// That is, the provenance of a `&'stable T` reference (or smaller borrow) is not unexpectedly
// invalidated by the source `Rc<T>` being dropped or mutated; only the above considerations about
// the `Rc` allocation matter.
//
// All `'stable` data in a `StableView` view of an `Rc<T>` must either be valid for at least
// `'data` or borrow part or all of the `Rc<T>`'s `T` pointee. By the above reasoning, we know that
// `&'stable T` references to (or `'stable` borrows to smaller parts of) the `Rc<T>`'s `T` pointee
// remain valid if its source pool is nonempty.
//
/// # Robust Guarantee
/// The conceptual pool associated with an `Rc<T>` value is the set of all (semantic) owners of a
/// strong ref of the `Rc` allocation (possibly including `Rc<U>` values from unsizing coercions,
/// custom types that share strong ref ownership in interesting ways, or custom types that hold more
/// than one strong ref), *except*, if the `T` value (not refcounts) of the `Rc` allocation is
/// mutated, then all data values in the pool are considered to be transferred over to a new pool
/// (while the previous pool is left empty).
///
/// In particular, the conceptual pool associated with a view is nonempty iff the strong count is
/// nonzero and the value (not refcounts) of the `Rc` allocation has not been mutated since the
/// view was taken.
unsafe impl<T: ?Sized> StableClone<'_> for Rc<T> {}


// ================================================================
//  `string::String`
// ================================================================

// SAFETY: We will go through each of the three operations. The `'data` upper bound
// doesn't particularly matter in the case of the owned `String` type.
//
// First, moves. Moving a `String` does not currently invalidate references to its contents,
// since it is a wrapper around `Vec<u8>`. Granted, that isn't a stable guarantee, but the type
// generally makes most of the same guarantees as `Vec<u8>` about its representation, so for the
// same reason as `Vec<T>`, it is *very* unlikely that moving a `String` will ever invalidate
// references to its contents: https://github.com/rust-lang/rfcs/pull/3712#issuecomment-3715013712
//
// Second, non-`DerefMut` coercions (among Rust 1.85 coercions). As noted by `StableView`, it
// should be covered by the first and third cases.
//
// Third, operations done to data derived from parts of `String` only through `&` references. Since
// `String` doesn't use internal mutability, operations done on shared references to part or all of
// a `String` value cannot invalidate a shared reference to its `str` contents.
// Note that operations done on one `&String` **CAN** invalidate a different `&String`... if the
// latter was derived from an older `&mut String` (or other `Unique`-tagged pointer) to the same
// `String`. However, the provenance and permissions of the `&str` derive from the `String`'s inner
// pointer, not from a `&mut String` which may be used to access that inner pointer.
unsafe impl<'a, 'data> StableView<'a, 'data, String> for PointerViewKind {
    type View = VaryingRef<str>;

    #[inline]
    unsafe fn view<'stable>(data: &'a String) -> &'stable str
    where
        'data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a str = data;

        // SAFETY: See the "`transmute` in `view` Implementation" section of the `StableView` docs.
        unsafe {
            transmute::<
                &'a str,
                &'stable str,
            >(stable_eq_a)
        }
    }
}

impl SetDefaultView<'_, '_> for String {
    type Default = PointerViewKind;
}

// SAFETY: We will go through each of the three operations. The `'data` upper bound
// doesn't particularly matter. As noted by `StableViewMut`, the second and third operations
// aren't particularly noteworthy either; our `view` impl doesn't do something strange that would
// break them while somehow still supporting the first operation.
//
// Then, moves. See above safety comment for `StableView` for `String`.
unsafe impl<'a, 'data> StableViewMut<'a, 'data, String> for PointerViewKind {
    type ViewMut = VaryingRefMut<str>;

    #[inline]
    unsafe fn view_mut<'stable>(data: &'a mut String) -> &'stable mut str
    where
        'data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a mut str = data;

        // SAFETY: See the "`transmute` in `view_mut` Implementation" section of the
        // `StableViewMut` docs.
        unsafe {
            transmute::<
                &'a mut str,
                &'stable mut str,
            >(stable_eq_a)
        }
    }
}

impl SetDefaultViewMut<'_, '_> for String {
    type DefaultMut = PointerViewKind;
}


// ================================================================
//  `sync::Arc`
// ================================================================

// SAFETY: Same as `PointerViewKind`'s impl of `StableView` for `Rc<T>` above.
unsafe impl<'a, 'data, T: ?Sized + 'data> StableView<'a, 'data, Arc<T>>
for PointerViewKind
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a Arc<T>) -> &'stable T
    where
        'data: 'stable,
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

impl<'data, T: ?Sized + 'data> SetDefaultView<'_, 'data> for Arc<T> {
    type Default = PointerViewKind;
}

// SAFETY: We will go through each of the three operations. The `'data` upper bound
// doesn't particularly matter. As noted by `StableViewMut`, the second and third operations
// aren't particularly noteworthy either; our `view` impl doesn't do something strange that would
// break them while somehow still supporting the first operation.
//
// Then, moves. See above safety comment for `StableView` for `Arc<T>`.
unsafe impl<'a, 'data, T: ?Sized + 'data> StableViewMut<'a, 'data, Arc<T>>
for PointerViewKind
{
    type ViewMut = Option<VaryingRefMut<Unvarying<T>>>;

    /// Uses [`Arc::get_mut`].
    #[inline]
    unsafe fn view_mut<'stable>(data: &'a mut Arc<T>) -> Option<&'stable mut T>
    where
        'data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: Option<&'a mut T> = Arc::get_mut(data);

        // SAFETY: See the "`transmute` in `view_mut` Implementation" section of the
        // `StableViewMut` docs.
        unsafe {
            transmute::<
                Option<&'a mut T>,
                Option<&'stable mut T>,
            >(stable_eq_a)
        }
    }
}

impl<'data, T: ?Sized + 'data> SetDefaultViewMut<'_, 'data> for Arc<T> {
    type DefaultMut = PointerViewKind;
}

// SAFETY: Same as that of `Rc<T>`'s impl of `StableClone` above.
//
/// # Robust Guarantee
/// The conceptual pool associated with an `Arc<T>` value is the set of all (semantic) owners of a
/// strong ref of the `Arc` allocation (possibly including `Arc<U>` values from unsizing coercions,
/// custom types that share strong ref ownership in interesting ways, or custom types that hold more
/// than one strong ref), *except*, if the `T` value (not refcounts) of the `Arc` allocation is
/// mutated, then all data values in the pool are considered to be transferred over to a new pool
/// (while the previous pool is left empty).
///
/// In particular, the conceptual pool associated with a view is nonempty iff the strong count is
/// nonzero and the value (not refcounts) of the `Arc` allocation has not been mutated since the
/// view was taken.
unsafe impl<T: ?Sized> StableClone<'_> for Arc<T> {}


// ================================================================
//  `vec::Vec`
// ================================================================

// SAFETY: We will go through each of the three operations. The `'data` upper bound
// doesn't particularly matter in the case of the owned `Vec<T>` type.
//
// First, moves. Moving a `Vec<T>` does not currently invalidate references to its contents,
// and that is *very* unlikely to ever change, due to concern about breaking existing code making
// it "out of the question": https://github.com/rust-lang/rfcs/pull/3712#issuecomment-3715013712
//
// Second, non-`DerefMut` coercions (among Rust 1.85 coercions). As noted by `StableView`, it
// should be covered by the first and third cases.
//
// Third, operations done to data derived from parts of `Vec<T>` only through `&` references. Since
// `Vec<T>` doesn't use internal mutability, operations done on shared references to part or all of
// a `Vec<T>` value cannot invalidate a shared reference to its `[T]` contents.
// Note that operations done on one `&Vec<T>` **CAN** invalidate a different `&Vec<T>`... if the
// latter was derived from an older `&mut Vec<T>` (or other `Unique`-tagged pointer) to the same
// `Vec<T>`. However, the provenance and permissions of the `&[T]` derive from the `Vec<T>`'s inner
// pointer, not from a `&mut Vec<T>` which may be used to access that inner pointer.
// TODO: Miri test; pointers are hard. I need to make sure that passing a `&mut Vec<T>` to
// `StableView::view` doesn't cause a problem.
unsafe impl<'a, 'data, T: 'data> StableView<'a, 'data, Vec<T>>
for PointerViewKind
{
    type View = VaryingRef<Unvarying<[T]>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a Vec<T>) -> &'stable [T]
    where
        'data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a [T] = data;

        // SAFETY: See the "`transmute` in `view` Implementation" section of the `StableView` docs.
        unsafe {
            transmute::<
                &'a [T],
                &'stable [T],
            >(stable_eq_a)
        }
    }
}

impl<'data, T: 'data> SetDefaultView<'_, 'data> for Vec<T> {
    type Default = PointerViewKind;
}

// SAFETY: We will go through each of the three operations. The `'data` upper bound
// doesn't particularly matter. As noted by `StableViewMut`, the second and third operations
// aren't particularly noteworthy either; our `view` impl doesn't do something strange that would
// break them while somehow still supporting the first operation.
//
// Then, moves. See above safety comment for `StableView` for `Vec`.
unsafe impl<'a, 'data, T: 'data> StableViewMut<'a, 'data, Vec<T>>
for PointerViewKind
{
    type ViewMut = VaryingRefMut<Unvarying<[T]>>;

    #[inline]
    unsafe fn view_mut<'stable>(data: &'a mut Vec<T>) -> &'stable mut [T]
    where
        'data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a mut [T] = data;

        // SAFETY: See the "`transmute` in `view_mut` Implementation" section of the
        // `StableViewMut` docs.
        unsafe {
            transmute::<
                &'a mut [T],
                &'stable mut [T],
            >(stable_eq_a)
        }
    }
}

impl<'data, T: 'data> SetDefaultViewMut<'_, 'data> for Vec<T> {
    type DefaultMut = PointerViewKind;
}
