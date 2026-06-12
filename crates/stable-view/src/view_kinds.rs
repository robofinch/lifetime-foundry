//! View kind types whose exact behavior is configurable.

#![expect(unsafe_code, reason = "defer to other unsafe impls")]

use core::marker::PhantomData;
use core::fmt::{Debug, Formatter, Result as FmtResult};

use implied_bounds::ImpliedPredicate;
use variance_family::{Unvarying, VaryingRef, VaryingRefMut};

use crate::traits::{CustomView, CustomViewMut, StableView, StableViewMut};


/// The view kind (or mutable view kind) chosen by a `Data` type as its default.
///
/// The behavior of this view kind should be configured via [`DefaultStableView`]
/// and [`DefaultStableViewMut`].
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
pub trait DefaultStableView<'a, 'data, __ImplyBound = &'a &'data ()> {
    /// The view which [`DefaultViewKind`] will defer to for `Data = Self`.
    type Default: StableView<'a, 'data, Self, __ImplyBound>;
}

/// Choose the default mutable view kind of the implementing type.
///
/// # `__ImplyBound`
/// It is not required for soundness that `__ImpliedBound` be left at its default of
/// `&'a &'data ()` (which implies `'data: 'a`); that bound is solely to improve
/// the usability of this trait. (No other implied bound should be necessary.)
pub trait DefaultStableViewMut<
    'a, 'data, __ImplyBound = &'a &'data (),
>: DefaultStableView<'a, 'data, __ImplyBound> {
    /// The mutable view which [`DefaultViewKind`] will defer to for `Data = Self`.
    type DefaultMut: StableViewMut<'a, 'data, Self, __ImplyBound>;
}

impl<'a, 'data, Data> StableView<'a, 'data, Data>
for DefaultViewKind
where
    Data: ?Sized + DefaultStableView<'a, 'data>,
{
    type View = <Data::Default as StableView<'a, 'data, Data>>::View;

    #[inline]
    unsafe fn view<'stable>(data: &'a Data) -> CustomView<'a, 'stable, 'data, Data, Self> {
        // SAFETY: The returned view can only be used at a given time if, from just after this
        // function returns until the time of use, only the three kinds of operations are performed,
        // and if `'data` has not ended.
        // This constraint is precisely what *our* `view` caller unsafely asserts, so this is sound.
        // In other words, we have simply forwarded the safety preconditions to the caller.
        unsafe { Data::Default::view(data) }
    }
}

impl<'a, 'data, Data> StableViewMut<'a, 'data, Data>
for DefaultViewKind
where
    Data: ?Sized + DefaultStableViewMut<'a, 'data>,
{
    type ViewMut = <Data::DefaultMut as StableViewMut<'a, 'data, Data>>::ViewMut;

    #[inline]
    unsafe fn view_mut<'stable>(
        data: &'a mut Data,
    ) -> CustomViewMut<'a, 'stable, 'data, Data, Self> {
        // SAFETY: The returned view can only be used at a given time if, from just after this
        // function returns until the time of use, only the three kinds of operations are performed,
        // and if `'data` has not ended.
        // This constraint is precisely what *our* `view_mut` caller unsafely asserts, so this is
        // sound. In other words, we have simply forwarded the safety preconditions to the caller.
        unsafe { Data::DefaultMut::view_mut(data) }
    }
}

/// A view kind intended for providing `&'stable T` or `&'stable mut T` views.
#[derive(Debug, Default, Clone, Copy)]
pub struct ReferenceViewKind;

/// Require that `Data` can provide a `&'stable T` view for some `T` via [`ReferenceViewKind`].
pub trait StableReferenceView<'data>
where
    Self: ImpliedPredicate<
        ReferenceViewKind,
        Impls: for<'a> StableView<'a, 'data, Self, View = VaryingRef<Unvarying<Self::Pointee>>>,
    >,
{
    /// `Data` can provide a `&'stable Data::Pointee` view via [`ReferenceViewKind`].
    type Pointee: ?Sized + 'data;
}

impl<'data, Data: ?Sized, Pointee: ?Sized + 'data> StableReferenceView<'data> for Data
where
    ReferenceViewKind: for<'a> StableView<
        'a, 'data, Self,
        View = VaryingRef<Unvarying<Pointee>>,
    >,
{
    type Pointee = Pointee;
}

/// Require that `Data` can provide a `&'stable mut T` mutable view for some `T` via
/// [`ReferenceViewKind`].
pub trait StableReferenceViewMut<'data>
where
    Self: ImpliedPredicate<
        ReferenceViewKind,
        Impls: for<'a> StableViewMut<
            'a, 'data, Self,
            ViewMut = VaryingRefMut<Unvarying<Self::MutPointee>>,
        >,
    >,
{
    /// `Data` can provide a `&'stable mut Data::MutPointee` mutable view via [`ReferenceViewKind`].
    type MutPointee: ?Sized + 'data;
}

impl<'data, Data: ?Sized, MutPointee: ?Sized + 'data> StableReferenceViewMut<'data> for Data
where
    ReferenceViewKind: for<'a> StableViewMut<
        'a, 'data, Self,
        ViewMut = VaryingRefMut<Unvarying<MutPointee>>,
    >,
{
    type MutPointee = MutPointee;
}

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
/// [`UnstableViewKind`]: crate::provided_view_kinds::UnstableViewKind
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

/// A view kind intended for types whose views are ZSTs.
#[derive(Debug, Default, Clone, Copy)]
pub struct ZeroSizedViewKind;

/// A view kind intended for more complicated data structures.
#[derive(Debug, Default, Clone, Copy)]
pub struct CollectionViewKind;
