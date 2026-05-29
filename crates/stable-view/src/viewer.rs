//! Assert the `unsafe` preconditions of [`StableView::view`] on construction, so that safe code
//! can choose how to view the data.

#![expect(unsafe_code, reason = "use `StableView::view`")]

use core::marker::PhantomData;
use core::fmt::{Debug, Formatter, Result as FmtResult};

use crate::{
    traits::{CustomView, StableView},
    view_kinds::{DefaultViewKind, View},
};


/// Enable safe to choose how to view `Data` by asserting the `unsafe` preconditions of
/// [`StableView::view`] on construction.
///
/// # Robust Guarantee
/// Until `'a` ends, it is sound to call [`StableView'a, 'data, Data>::view<'stable>`] on the
/// wrapped `&'a Data` value, with any view kind.
///
/// Note that this type is covariant over all of its type parameters. This is perfectly fine.
/// - Shortening `'a` or `Data` does not grant any additional power; else, the reliance of
///   [`StableView::view`] on the covariant `&'a Data` would be unsound).
/// - Views are already covariant over `'stable`, so shortening `'stable` doesn't grant additional
///   power (and only gives better ergonomics, at best).
/// - Shortening `'data` does not retroactively reduce the strength of the preconditions of
///   [`Viewer::new`], and does not grant any additional power to users of a `Viewer`.
///
/// [`StableView'a, 'data, Data>::view<'stable>`]: StableView::view
#[repr(transparent)]
pub struct Viewer<'a, 'stable, 'data, Data: ?Sized> {
    /// Included for implied bounds (and to covariantly mention these lifetimes).
    _bounds: PhantomData<&'a &'stable &'data ()>,
    /// # Safety Invariant
    /// The conditions required by [`Self::new`] of its given `data` value must always hold of
    /// `self.data`.
    data:    &'a Data,
}


impl<'a, 'stable, 'data, Data: ?Sized> Viewer<'a, 'stable, 'data, Data> {
    /// # Safety
    /// While `'data` has not yet ended, the `'stable` data of any returned view can be used at a
    /// given moment so long as, starting from when the view is returned from this function up to
    /// when its `'stable` data is used, only the following three operations are performed on the
    /// source `Data` value (in any quantity and ordering):
    /// - moves,
    /// - non-`DerefMut` [coercions] among those available in Rust 1.85 (which may or may not
    ///   involve moves),
    /// - any (sound) operations which use data derived from the source `Data` value only through
    ///   shared/immutable `&` references to the relevant parts of `Data`. (These could be called
    ///   "immutable operations" on the source `Data` value, if not for internal mutability within
    ///   `Data`, which could escalate a `&` reference to part of `Data` to a `&mut` reference
    ///   to another part of `Data`.)
    ///
    /// While `'data` has not yet ended, the `'stable` data of views obtained by calling
    /// [`Viewer::view`] or [`Viewer::view_with`] on the returned `Viewer` ***must*** only be used
    /// at a given moment so long as, starting from when the `Viewer` is returned from this
    /// constructor up to when the `'stable` data is used, only the following three operations are
    /// performed on the source `Data` value (in any quantity and ordering):
    /// - moves,
    /// - non-`DerefMut` [coercions] among those available in Rust 1.85 (which may or may not
    ///   involve moves),
    /// - any (sound) operations which use data derived from the source `Data` value only through
    ///   shared/immutable `&` references to the relevant parts of `Data`. (These could be called
    ///   "immutable operations" on the source `Data` value, if not for internal mutability within
    ///   `Data`, which could escalate a `&` reference to part of `Data` to a `&mut` reference
    ///   to another part of `Data`.)
    ///
    /// # Sound Usage
    ///
    /// Note that callers of [`Viewer::view`] and [`Viewer::view_with`] do **not** have permission
    /// to arbitrarily extend `'stable` to `'data`. (With sufficient control of the surrounding
    /// code, including the caller of this method, soundly doing so might be possible for users
    /// of the `Viewer`; don't worry about such scenarios, their `unsafe` code is their
    /// responsibility.)
    ///
    /// Therefore, you don't need to worry about views managing to live longer than `'stable`.
    ///
    /// However, calling this method does still require a fair amount of control over `'a`,
    /// `'stable`, and `Data`. Notably, views can only be obtained during lifetime `'a`, so you
    /// may be able to tightly constrain *where* views can be obtained from the returned `Viewer`;
    /// in combination with constraints on `'stable`, you can constrain where the `'stable` data
    /// of those views escapes to.
    ///
    /// If you know the full set of possible places where `'stable` data from views obtained
    /// from the returned `Viewer` could have ended up, you can soundly perform invalidating
    /// operations on the backing `Data` value after ensuring that **all** possible `'stable`
    /// view data has been discarded.
    #[inline]
    #[must_use]
    pub const unsafe fn new(data: &'a Data) -> Self {
        Self {
            _bounds: PhantomData,
            data,
        }
    }

    /// View the `Data` value with its [default] view.
    ///
    /// Note: this type does **not** guarantee that you have permission to `unsafe`ly
    /// lifetime-extend `'stable` to `'data`. Only usage of the `'stable` data *up to* this
    /// `Viewer`'s `'stable` lifetime is guaranteed sound.
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

    /// View the `Data` value using the indicated view kind.
    ///
    /// Note: this type does **not** guarantee that you have permission to `unsafe`ly
    /// lifetime-extend `'stable` to `'data`. Only usage of the `'stable` data *up to* this
    /// `Viewer`'s `'stable` lifetime is guaranteed sound.
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

impl<Data: ?Sized> Clone for Viewer<'_, '_, '_, Data> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Data: ?Sized> Copy for Viewer<'_, '_, '_, Data> {}

impl<Data: ?Sized + Debug> Debug for Viewer<'_, '_, '_, Data> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("Viewer").field(&self.data).finish()
    }
}
