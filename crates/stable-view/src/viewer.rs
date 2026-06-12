//! Assert the `unsafe` preconditions of [`StableView::view`] or [`StableViewMut::view_mut`] on
//! construction, so that safe code can choose how to view the data.

#![expect(unsafe_code, reason = "use `StableView(Mut)::view(_mut)`")]
#![warn(clippy::missing_inline_in_public_items, reason = "methods are trivial")]

use core::marker::PhantomData;
use core::fmt::{Debug, Formatter, Result as FmtResult};

use crate::{
    traits::{CustomView, CustomViewMut, StableView, StableViewMut},
    view_kinds::{DefaultViewKind, View, ViewMut},
};


/// Obtain views to a `Data` value. The `'stable` data of the views is suitable for self-references
/// to that value.
///
/// This functionality is accomplished via [`StableView`], which guarantees that the covariant
/// `'stable` data of the views can be soundly lifetime-extended under specific conditions (which
/// can be satisfied in self-referential structs).
///
/// # Advanced Details
///
/// ## Robust Guarantee
/// On construction, this type asserts the `unsafe` preconditions of [`StableView::view`].
///
/// (Note that `'stable` may be overly-long, but the caller of [`StableViewer::new`] is
/// responsible for ensuring that it cannot cause unsoundness. You **do not** have unconditional
/// permission to `unsafe`ly lifetime-extend `'stable` to `'data` in views returned by this type.)
///
/// Until `'a` ends, it is sound to call [`StableView'a, 'data, Data>::view<'stable>`] on the
/// wrapped `&'a Data` value, with any view kind.
///
/// ## Variance
///
/// Note that this type is covariant over all of its type parameters. This is perfectly fine.
/// - Shortening `'a` or `Data` does not grant any additional power; else, the reliance of
///   [`StableView::view`] on the covariant `&'a Data` would be unsound).
/// - Views are already covariant over `'stable`, so shortening `'stable` doesn't grant additional
///   power (and only gives better ergonomics, at best).
/// - Shortening `'data` does not retroactively reduce the strength of the preconditions of
///   [`StableViewer::new`], and does not grant any additional power to users of a `StableViewer`.
///
/// [`StableView'a, 'data, Data>::view<'stable>`]: StableView::view
#[repr(transparent)]
pub struct StableViewer<'a, 'stable, 'data, Data: ?Sized> {
    /// Included for implied bounds (and to covariantly mention these lifetimes).
    _bounds: PhantomData<&'a &'stable &'data ()>,
    /// # Safety Invariant
    /// The conditions required by [`Self::new`] of its given `data` value must always hold of
    /// `self.data`.
    data:    &'a Data,
}

impl<'a, 'stable, 'data, Data: ?Sized> StableViewer<'a, 'stable, 'data, Data> {
    /// Assert the `unsafe` preconditions of [`StableView::view`], for later application.
    ///
    /// This method is only intended to be used by experienced authors of `unsafe` code.
    ///
    /// # Safety
    /// While `'data` has not yet ended, the stable data of views obtained by calling
    /// [`StableViewer::view`] or [`StableViewer::view_with`] on the returned `StableViewer`
    /// ***must*** only be used at a given moment so long as, starting from when the `StableViewer`
    /// is returned from this constructor up to when the `'stable` data is used, only the three
    /// kinds of operations permitted by [`StableView::view`] are performed on the source `Data`
    /// value (in any quantity and ordering).
    ///
    /// # Sound Usage
    ///
    /// Note that callers of [`StableViewer::view`] and [`StableViewer::view_with`] do **not** have
    /// permission to arbitrarily extend `'stable` to `'data`. (With sufficient control of the
    /// surrounding code, including the caller of this method, soundly doing so might be possible
    /// for users of the `StableViewer`; don't worry about such scenarios, their `unsafe` code is
    /// their responsibility.)
    ///
    /// Therefore, you don't need to worry about views managing to live longer than `'stable`.
    ///
    /// However, calling this method does still require a fair amount of control over `'a`,
    /// `'stable`, and `Data`. Notably, views can only be obtained during lifetime `'a`, so you
    /// may be able to tightly constrain *where* views can be obtained from the returned
    /// `StableViewer`; in combination with constraints on `'stable`, you can constrain where the
    /// stable data of those views escapes to.
    ///
    /// If you know the full set of possible places where stable data from views obtained
    /// from the returned `StableViewer` could have ended up, you can soundly perform invalidating
    /// operations on the backing `Data` value after ensuring that **all** possible stable
    /// view data has been discarded.
    ///
    /// [coercions]: https://doc.rust-lang.org/reference/type-coercions.html
    #[inline]
    #[must_use]
    pub const unsafe fn new(data: &'a Data) -> Self {
        Self {
            _bounds: PhantomData,
            data,
        }
    }

    /// View the `Data` value with its [default] view. The `'stable` data of the view is suitable
    /// for self-references to the `Data` value.
    ///
    /// This functionality is accomplished via [`StableView`], which guarantees that the covariant
    /// `'stable` data in the view can be soundly lifetime-extended under specific conditions
    /// (which can be satisfied in self-referential structs).
    ///
    /// [default]: DefaultViewKind
    #[inline]
    #[must_use]
    pub fn view(self) -> View<'a, 'stable, 'data, Data>
    where
        DefaultViewKind: StableView<'a, 'data, Data>,
    {
        self.view_with::<DefaultViewKind>()
    }

    /// View the `Data` value using the indicated view kind. The `'stable` data of the view is
    /// suitable for self-references to the `Data` value.
    ///
    /// This functionality is accomplished via [`StableView`], which guarantees that the covariant
    /// `'stable` data in the view can be soundly lifetime-extended under specific conditions
    /// (which can be satisfied in self-referential structs).
    #[inline]
    #[must_use]
    pub fn view_with<V: StableView<'a, 'data, Data>>(
        self,
    ) -> CustomView<'a, 'stable, 'data, Data, V> {
        // SAFETY: See the safety invariant of `self.data`, and the safety preconditions of
        // `Self::new`, and the reasoning about covariance described in the type-level
        // documentation.
        // The soundness of this call is implied by the assertions unsafely made by the caller of
        // `Self::new`.
        unsafe { V::view(self.data) }
    }

    /// Get the inner `data` reference.
    ///
    /// Note that this function is also possible in non-`const` code view
    /// `self.view_with::<UnstableViewKind>()`. Pretty cool how things tie together!
    #[inline]
    #[must_use]
    pub const fn inner(self) -> &'a Data {
        self.data
    }
}

impl<Data: ?Sized> Clone for StableViewer<'_, '_, '_, Data> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Data: ?Sized> Copy for StableViewer<'_, '_, '_, Data> {}

impl<Data: ?Sized + Debug> Debug for StableViewer<'_, '_, '_, Data> {
    #[expect(clippy::missing_inline_in_public_items, reason = "don't inline formatting machinery")]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("StableViewer").field(&self.data).finish()
    }
}

/// Obtain mutable views to a `Data` value. The `'stable` data of the views is suitable for
/// mutable self-references to that value.
///
/// This viewer can also be downgraded to a [`StableViewer`], which can yield multiple
/// immutable/shared views instead of one mutable/exclusive view.
///
/// This functionality is accomplished via [`StableViewMut`], which guarantees that the covariant
/// `'stable` data of the views can be soundly lifetime-extended under specific conditions (which
/// can be satisfied in self-referential structs).
///
/// See [`StableViewMut`] and [`StableViewer`] for more.
///
/// # Advanced Details
///
/// ## Robust Guarantee
/// This type asserts the `unsafe` preconditions of [`StableViewMut::view_mut`] on its construction.
///
/// Until `'a` ends, it is sound to call [`StableViewMut'a, 'data, Data>::view_mut<'stable>`] on the
/// wrapped `&'a Data` value, with any view kind, **at most once** (since additional calls would
/// invalidate the previously-obtained mutable views).
///
/// ## Variance
///
/// Note that this type is covariant over all of its type parameters, except `Data`.
/// This is perfectly fine.
/// - Shortening `'a` does not grant any additional power; else, the reliance of
///   [`StableViewMut::view_mut`] on the covariant `&'a Data` would be unsound).
/// - Views are already covariant over `'stable`, so shortening `'stable` doesn't grant additional
///   power (and only gives better ergonomics, at best).
/// - Shortening `'data` does not retroactively reduce the strength of the preconditions of
///   [`StableViewMut::view_mut`], and does not grant any additional power to users of a
///   `StableViewerMut`.
///
/// [`StableViewMut'a, 'data, Data>::view_mut<'stable>`]: StableViewMut::view_mut
#[repr(transparent)]
pub struct StableViewerMut<'a, 'stable, 'data, Data: ?Sized> {
    /// Included for implied bounds (and to covariantly mention these lifetimes).
    _bounds: PhantomData<&'a &'stable &'data ()>,
    /// # Safety Invariant
    /// The conditions required by [`Self::new`] of its given `data` value must always hold of
    /// `self.data`.
    data:    &'a mut Data,
}

impl<'a, 'stable, 'data, Data: ?Sized> StableViewerMut<'a, 'stable, 'data, Data> {
    /// Assert the `unsafe` preconditions of [`StableViewMut::view_mut`], for later application.
    ///
    /// This method is only intended to be used by experienced authors of `unsafe` code.
    ///
    /// # Safety
    /// While `'data` has not yet ended, the stable data of a view obtained by calling
    /// [`StableViewerMut::view_mut`] or [`StableViewerMut::view_mut_with`] on the returned
    /// `StableViewer` ***must*** only be used at a given moment so long as, starting from when the
    /// `StableViewerMut` is returned from this constructor up to when the stable data is used,
    /// only the three kinds operations permitted by [`StableViewMut::view_mut`] are performed on
    /// the source `Data` value (in any quantity and ordering):
    /// - moves, including any accompanying retag and other effects in the aliasing model, and
    /// - operations which only read or write the inline data of the source `Data` value.
    ///
    /// # Sound Usage
    ///
    /// Note that callers of [`StableViewerMut::view_mut`] and [`StableViewerMut::view_mut_with`]
    /// do **not** have permission to arbitrarily extend `'stable` to `'data`. (With sufficient
    /// control of the surrounding code, including the caller of this method, soundly doing so might
    /// be possible for users of the `StableViewerMut`; don't worry about such scenarios, their
    /// `unsafe` code is their responsibility.)
    ///
    /// Therefore, you don't need to worry about views managing to live longer than `'stable`.
    ///
    /// However, calling this method does still require a fair amount of control over `'a`,
    /// `'stable`, and `Data`. Notably, views can only be obtained during lifetime `'a`, so you
    /// may be able to tightly constrain *where* views can be obtained from the returned
    /// `StableViewerMut`; in combination with constraints on `'stable`, you can constrain where the
    /// stable data of those views escapes to.
    ///
    /// If you know the full set of possible places where stable data from views obtained
    /// from the returned `StableViewerMut` could have ended up, you can soundly perform
    /// invalidating operations on the backing `Data` value after ensuring that **all** possible
    /// stable view data has been discarded.
    ///
    /// [coercions]: https://doc.rust-lang.org/reference/type-coercions.html
    #[inline]
    #[must_use]
    pub const unsafe fn new(data: &'a mut Data) -> Self {
        Self {
            _bounds: PhantomData,
            data,
        }
    }

    /// View the `Data` value with its [default] mutable view.
    ///
    /// See [`StableViewMut`] for more.
    ///
    /// Note: this type does **not** guarantee that you have permission to `unsafe`ly
    /// lifetime-extend `'stable` to `'data`. Only usage of the `'stable` data *up to* this
    /// `StableViewerMut`'s `'stable` lifetime is guaranteed sound.
    ///
    /// [default]: DefaultViewKind
    #[inline]
    #[must_use]
    pub fn view_mut(self) -> ViewMut<'a, 'stable, 'data, Data>
    where
        DefaultViewKind: StableViewMut<'a, 'data, Data>,
    {
        self.view_mut_with::<DefaultViewKind>()
    }

    /// View the `Data` value using the indicated view kind.
    ///
    /// See [`StableViewMut`] for more.
    ///
    /// Note: this type does **not** guarantee that you have permission to `unsafe`ly
    /// lifetime-extend `'stable` to `'data`. Only usage of the `'stable` data *up to* this
    /// `StableViewerMut`'s `'stable` lifetime is guaranteed sound.
    #[inline]
    #[must_use]
    pub fn view_mut_with<V: StableViewMut<'a, 'data, Data>>(
        self,
    ) -> CustomViewMut<'a, 'stable, 'data, Data, V> {
        // SAFETY: See the safety invariant of `self.data`, and the safety preconditions of
        // `Self::new`, and the reasoning about covariance described in the type-level
        // documentation.
        // The soundness of this call is implied by the assertions unsafely made by the caller of
        // `Self::new`.
        unsafe { V::view_mut(self.data) }
    }

    /// Get immutable access to the inner `data`.
    ///
    /// Note that `'stable` views cannot be obtained from the returned reference; the reference
    /// may be invalidated when a mutable view is obtained via this `StableViewerMut`.
    #[inline]
    #[must_use]
    pub const fn inner(&self) -> &Data {
        // NOTE: We do not promise that `'stable` references can be obtained from the returned
        // reference, and since `self` still exists, we know that a mutable view has not yet
        // been obtained (which could be invalidated by this).
        self.data
    }

    /// Get the inner `data` reference.
    ///
    /// Note that this function is also possible in non-`const` code view
    /// `self.view_mut_with::<UnstableViewKind>()`. Pretty cool how things tie together!
    #[inline]
    #[must_use]
    pub const fn into_inner(self) -> &'a mut Data {
        self.data
    }

    /// Downgrade this viewer to a [`StableViewer`], enabling the creation of multiple
    /// immutable/shared `'stable` views.
    #[inline]
    #[must_use]
    pub const fn into_viewer(self) -> StableViewer<'a, 'stable, 'data, Data> {
        // SAFETY: The constraints of `StableViewerMut::new` are *strictly* stronger than
        // those of `StableViewer::new`, and by the safety invariant of `self.data`, we know that
        // those constraints still hold.
        // In particular, `StableViewer` additionally allows operations on `&Data` to be performed.
        // No additional operations (compared to `StableViewerMut`) are prohibited.
        // Moreover, since `self` still exists, we know that a mutable view has not yet
        // been obtained (which could be invalidated by this).
        unsafe { StableViewer::new(self.data) }
    }
}

impl<Data: ?Sized + Debug> Debug for StableViewerMut<'_, '_, '_, Data> {
    #[expect(clippy::missing_inline_in_public_items, reason = "don't inline formatting machinery")]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("StableViewerMut").field(&self.inner()).finish()
    }
}

impl<'a, 'stable, 'data, Data: ?Sized> From<StableViewerMut<'a, 'stable, 'data, Data>>
for StableViewer<'a, 'stable, 'data, Data>
{
    #[inline]
    fn from(value: StableViewerMut<'a, 'stable, 'data, Data>) -> Self {
        value.into_viewer()
    }
}
