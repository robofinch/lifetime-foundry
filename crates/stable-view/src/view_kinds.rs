//! Vocabulary types for convenient usage of this crate.

#![expect(unsafe_code, reason = "defer to other unsafe impls, and a trivial unsafe impl")]

use core::marker::PhantomData;
use core::fmt::{Debug, Formatter, Result as FmtResult};

use variance_family::Unvarying;

use crate::traits::{CustomView, CustomViewMut, StableClone, StableView, StableViewMut};


/// The view kind (or mutable view kind) chosen by a `Data` type as its default.
///
/// The behavior of this view kind should be configured via [`SetDefaultView`]
/// and [`SetDefaultViewMut`].
///
/// # Robust Guarantee
///
/// If `Data` implements [`SetDefaultView`], then for the purposes of [`StableClone`], the
/// definition of conceptual pools associated with this view kind and `Data` is the definition of
/// conceptual pools used by <code><Data as [SetDefaultView]>::[Default]></code> (if the latter
/// implements [`StableClone`]).
///
/// [Default]: SetDefaultView::Default
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultViewKind;

/// The [`StableView::View`] chosen by a `Data` type as its default.
pub type View<'a, 'stable, 'data, Data>
    = CustomView<'a, 'stable, 'data, Data, DefaultViewKind>;

/// The [`StableViewMut::ViewMut`] chosen by a `Data` type as its default.
pub type ViewMut<'a, 'stable, 'data, Data>
    = CustomViewMut<'a, 'stable, 'data, Data, DefaultViewKind>;

/// Choose the default view kind of the implementing type.
///
/// # `__ImplyBound`
/// It is not required for soundness that `__ImpliedBound` be left at its default of
/// `&'a &'data ()` (which implies `'data: 'a`); that bound is solely to improve
/// the usability of this trait. (No other implied bound should be necessary.)
pub trait SetDefaultView<'a, 'data, __ImplyBound = &'a &'data ()> {
    /// The view which [`DefaultViewKind`] will defer to for `Data = Self`.
    type Default: StableView<'a, 'data, Self, __ImplyBound>;
}

/// Choose the default mutable view kind of the implementing type.
///
/// # `__ImplyBound`
/// It is not required for soundness that `__ImpliedBound` be left at its default of
/// `&'a &'data ()` (which implies `'data: 'a`); that bound is solely to improve
/// the usability of this trait. (No other implied bound should be necessary.)
pub trait SetDefaultViewMut<
    'a, 'data, __ImplyBound = &'a &'data (),
>: SetDefaultView<'a, 'data, __ImplyBound> {
    /// The mutable view which [`DefaultViewKind`] will defer to for `Data = Self`.
    type DefaultMut: StableViewMut<'a, 'data, Self, __ImplyBound>;
}

// SAFETY: Since `Data::Default: StableView<'a, 'data, Data>`, we know that the
// `'stable` data in the view returned by `Data::Default::view(data)` is not invalidated by
// the three operations applied to `data` (up to the `'data` upper bound). Our `view` impl
// simply defers to that implementation, so our views are not invalidated by the three operations
// (and we have the same `'data` upper bound).
unsafe impl<'a, 'data, Data> StableView<'a, 'data, Data>
for DefaultViewKind
where
    Data: ?Sized + SetDefaultView<'a, 'data>,
{
    type View = <Data::Default as StableView<'a, 'data, Data>>::View;

    #[inline]
    unsafe fn view<'stable>(data: &'a Data) -> CustomView<'a, 'stable, 'data, Data, Self> {
        // SAFETY: The returned view can only be used at a given time if, from just after this
        // function returns until the time of use, only the three operations are performed,
        // and if `'data` has not ended.
        // This constraint is precisely what *our* `view` caller unsafely asserts, so this is sound.
        // In other words, we have simply forwarded the safety preconditions to the caller.
        unsafe { Data::Default::view(data) }
    }
}

// SAFETY: Since `Data::DefaultMut: StableViewMut<'a, 'data, Data>`, we know that the
// `'stable` data in the view returned by `Data::DefaultMut::view_mut(data)` is not invalidated by
// the three operations applied to `data` (up to the `'data` upper bound). Our `view_mut` impl
// simply defers to that implementation, so our views are not invalidated by the three operations
// (and we have the same `'data` upper bound).
unsafe impl<'a, 'data, Data> StableViewMut<'a, 'data, Data>
for DefaultViewKind
where
    Data: ?Sized + SetDefaultViewMut<'a, 'data>,
{
    type ViewMut = <Data::DefaultMut as StableViewMut<'a, 'data, Data>>::ViewMut;

    #[inline]
    unsafe fn view_mut<'stable>(
        data: &'a mut Data,
    ) -> CustomViewMut<'a, 'stable, 'data, Data, Self> {
        // SAFETY: The returned view can only be used at a given time if, from just after this
        // function returns until the time of use, only the three operations are performed,
        // and if `'data` has not ended.
        // This constraint is precisely what *our* `view_mut` caller unsafely asserts, so this is
        // sound. In other words, we have simply forwarded the safety preconditions to the caller.
        unsafe { Data::DefaultMut::view_mut(data) }
    }
}

// SAFETY: Since our conceptual pool definition defers to a different `StableClone` impl, the first
// two requirements of the definition must be met. Since our `view` impl *also* defers to that
// same `StableClone` impl's `view` function, the third requirement must also be met.
//
/// # Robust Guarantee
///
/// If `Data` implements [`SetDefaultView`], then for the purposes of [`StableClone`], the
/// definition of conceptual pools associated with this view kind and `Data` is the definition of
/// conceptual pools used by <code><Data as [SetDefaultView]>::[Default]></code> (if the latter
/// implements [`StableClone`]).
///
/// [Default]: SetDefaultView::Default
unsafe impl<'a, 'data, Data> StableClone<'a, 'data, Data>
for DefaultViewKind
where
    Data: Clone + SetDefaultView<'a, 'data, Default: StableClone<'a, 'data, Data>>
{}

/// A trivial view kind (or mutable view kind) whose returned views have no `'stable` references.
///
/// Its view methods are no-ops.
///
/// # Robust Guarantee
///
/// For the purposes of [`StableClone`], the definition of conceptual pools associated with
/// [`UnstableViewKind`] and any `Data` type is as follows: all values (regardless of type) are in
/// a single conceptual pool which is always nonempty. (In other words, all of the zero `'stable`
/// data in unstable views is valid forever.)
#[derive(Debug, Default, Clone, Copy)]
pub struct UnstableViewKind;

// SAFETY: There is no `'stable` data in the returned view; as such, it vacuously holds that
// none of the three operations are capable of invalidating `'stable` data in the returned views.
unsafe impl<'a, Data: ?Sized + 'a> StableView<'a, '_, Data> for UnstableViewKind {
    type View = &'a Unvarying<Data>;

    #[inline]
    unsafe fn view<'stable: 'stable>(data: &'a Data) -> &'a Data {
        data
    }
}

// SAFETY: There is no `'stable` data in the returned view; as such, it vacuously holds that
// none of the three operations are capable of invalidating `'stable` data in the returned views.
unsafe impl<'a, Data: ?Sized + 'a> StableViewMut<'a, '_, Data> for UnstableViewKind {
    type ViewMut = &'a mut Unvarying<Data>;

    #[inline]
    unsafe fn view_mut<'stable: 'stable>(data: &'a mut Data) -> &'a mut Data {
        data
    }
}

// SAFETY: The trivial pool definition used for `UnstableViewKind` and any `Data` type easily
// satisfies the first two requirements: all values are added to the pool, and are never removed
// while they exist. For the third requirement, the associated pool is always nonempty, and
// all of the `'stable` data in returned views is (vacuously) always valid.
unsafe impl<'a, Data: Clone + 'a> StableClone<'a, '_, Data> for UnstableViewKind {}

/// A composite view kind intended for propagating the guarantees unsafely asserted by the caller
/// of [`StableView::view`] or [`StableViewMut::view_mut`].
///
/// This view kind is most useful for types like `Option<T>` or `Result<T, E>` which cannot
/// provide `'stable` references to their direct contents, but want to allow getting `'stable`
/// references to indirect contents, such as getting an `Option<&'stable T>` from `Option<Rc<T>>`.
///
/// While the behavior of this view kind could be soundly replicated by using [`UnstableViewKind`]
/// together with additional `unsafe` code, this type reduces the quantity of `unsafe` code needed
/// in downstream usage.
///
/// # Conceptual Pools
/// The definition of conceptual pool used by `RecursiveViewKind<ViewKinds>` for `Data` can be
/// customized per-impl. The definition used by the [`recursive_view`] macro should be considered
/// the standard definition of a conceptual pool for this view kind, but unless an implementor
/// robustly guarantees that this definition is used, you should not rely on it for soundness.
///
/// (All users of [`recursive_view`] do make the robust guarantee.)
///
/// [`recursive_view`]: crate::recursive_view
pub struct RecursiveViewKind<ViewKinds>(PhantomData<fn() -> ViewKinds>);

impl<ViewKinds> RecursiveViewKind<ViewKinds> {
    /// Get a new [`RecursiveViewKind`] marker ZST.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<ViewKinds> Debug for RecursiveViewKind<ViewKinds> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("RecursiveView").field(&self.0).finish()
    }
}

impl<ViewKinds> Default for RecursiveViewKind<ViewKinds> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<ViewKinds> Copy for RecursiveViewKind<ViewKinds> {}

impl<ViewKinds> Clone for RecursiveViewKind<ViewKinds> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

/// A view kind intended for providing `'stable` references to the pointees of pointer-like `Data`
/// types, such as `Rc<T>`, `&'b T`, or `Vec<T>` (as a smart pointer to `[T]`).
#[derive(Debug, Default, Clone, Copy)]
pub struct PointerViewKind;

/// A view kind intended for types whose views are ZSTs.
#[derive(Debug, Default, Clone, Copy)]
pub struct ZeroSizedViewKind;

/// A view kind intended for more complicated data structures.
#[derive(Debug, Default, Clone, Copy)]
pub struct CollectionViewKind;
