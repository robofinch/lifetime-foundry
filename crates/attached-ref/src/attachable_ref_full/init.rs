//! Temporary organization module.

use core::convert::Infallible;

use stable_view::{
    ReferenceViewKind, StableReferenceView, StableReferenceViewMut, StableViewer, StableViewerMut,
};
use variance_family::{Lend, LendFamily};

use crate::{error::TryAttachError, outlives::OutlivesChain, pre_1_94_closure_hack::LendWrapper};
use crate::slot::{SelfRefCases, SelfRefSlot};
use super::full_struct::{AttachableRefFull, SpeedBump};


impl<N, R, M, Data> AttachableRefFull<'static, 'static, N, R, M, Data>
where
    R: LendFamily<&'static ()>,
    M: LendFamily<&'static ()>,
{
    /// Construct an [`AttachableRefFull`] in the [`Ref`] state from an immutable/shared
    /// self-reference to the given `data: Data` backing data.
    ///
    /// The value returned by the `f` callback will be stored alongside the `data` value in
    /// a self-referential struct; the trait bounds prevent unsoundness.
    ///
    /// See [`AttachableRefFull::try_attach_ref`] for fallible initialization or
    /// [`AttachableRefFull::attach_mut`] for mutable/exclusive self-references.
    ///
    /// This constructor is a special case; see [`AttachableRefFull::attach_ref_with`] for
    /// greater flexibility.
    ///
    /// The provided callback can access short-lived data, mutate its environment, and so on; the
    /// trait bounds enforce that its return value's `'stable` data consists only of:
    /// - self-references to the given `data: Data` value obtained via [`StableView`], and/or
    /// - data that could live for `'static` (i.e., that met a `'static` bound).
    ///
    /// (The return value may also contain non-`'stable` data using other lifetimes in `R`.)
    ///
    /// [`StableView`]: stable_view::StableView
    /// [`Ref`]: SelfRefCases::Ref
    #[inline]
    #[must_use]
    pub fn attach_ref<F>(data: Data, f: F) -> Self
    where
        Data: StableReferenceView<'static>,
        F:    for<'stable> FnOnce(&'stable Data::Pointee) -> Lend<'stable, &'static (), R>,
    {
        Self::attach_ref_with(data, |viewer, _outlives| {
            let view = viewer.view_with::<ReferenceViewKind>();
            // Since `Data: StableReferenceView<'static>` implies that `Data::Pointee: 'static`,
            // we don't need to provide `f` with something like `Outlives` or `OutlivesChain`;
            // the callback is too simple for there to be any real risk of unsoundness from compiler
            // bugs.
            LendWrapper::new(f(view))
        })
    }

    /// Try to construct an [`AttachableRefFull`] in the [`Ref`] state with an immutable/shared
    /// self-reference to the given `data: Data` backing data.
    ///
    /// This constructor is a special case; see [`AttachableRefFull::try_attach_ref_with`] for
    /// greater flexibility.
    ///
    /// # Errors
    /// Any error from the callback producing the self-reference is passed up.
    ///
    /// [`Ref`]: SelfRefCases::Ref
    #[inline]
    pub fn try_attach_ref<F, E>(data: Data, f: F) -> Result<Self, TryAttachError<Data, E>>
    where
        Data: StableReferenceView<'static>,
        F:  for<'stable> FnOnce(
                &'stable Data::Pointee,
            ) -> Result<Lend<'stable, &'static (), R>, E>,
    {
        Self::try_attach_ref_with(data, |viewer, _outlives| {
            let view = viewer.view_with::<ReferenceViewKind>();
            f(view).map(LendWrapper::new)
        })
    }

    /// Construct an [`AttachableRefFull`] in the [`RefMut`] state with a mutable/exclusive
    /// self-reference to the given `data: Data` backing data.
    ///
    /// The value returned by the `f` callback will be stored alongside the `data` value in
    /// a self-referential struct; the trait bounds prevent unsoundness.
    ///
    /// See [`AttachableRefFull::try_attach_mut`] for fallible initialization or
    /// [`AttachableRefFull::attach_ref`] for immutable/shared self-references.
    ///
    /// This constructor is a special case; see [`AttachableRefFull::attach_mut_with`] for
    /// greater flexibility.
    ///
    /// The provided callback can access short-lived data, mutate its environment, and so on; the
    /// trait bounds enforce that its return value's `'stable` data consists only of:
    /// - self-references to the given `data: Data` value, obtained via [`StableView`] or
    ///   [`StableViewMut`], and/or
    /// - data that could live for `'static` (i.e., that met a `'static` bound).
    ///
    /// (The return value may also contain non-`'stable` data using other lifetimes in `M`.)
    ///
    /// [`RefMut`]: SelfRefCases::RefMut
    /// [`StableView`]: stable_view::StableView
    /// [`StableViewMut`]: stable_view::StableViewMut
    #[inline]
    #[must_use]
    pub fn attach_mut<F>(data: Data, f: F) -> Self
    where
        Data: StableReferenceViewMut<'static>,
        F:    for<'stable> FnOnce(&'stable mut Data::MutPointee) -> Lend<'stable, &'static (), M>,
    {
        Self::attach_mut_with(data, |viewer, _outlives| {
            let view = viewer.view_mut_with::<ReferenceViewKind>();
            LendWrapper::new(f(view))
        })
    }

    /// Try to construct an [`AttachableRefFull`] in the [`RefMut`] state with a mutable/exclusive
    /// self-reference to the given `data: Data` backing data.
    ///
    /// This constructor is a special case; see [`AttachableRefFull::try_attach_mut_with`] for
    /// greater flexibility.
    ///
    /// # Errors
    /// Any error from the callback producing the self-reference is passed up.
    ///
    /// [`RefMut`]: SelfRefCases::RefMut
    #[inline]
    pub fn try_attach_mut<F, E>(data: Data, f: F) -> Result<Self, TryAttachError<Data, E>>
    where
        Data: StableReferenceViewMut<'static>,
        F:  for<'stable> FnOnce(
                &'stable mut Data::MutPointee,
            ) -> Result<Lend<'stable, &'static (), M>, E>,
    {
        Self::try_attach_mut_with(data, |viewer, _outlives| {
            let view = viewer.view_mut_with::<ReferenceViewKind>();
            f(view).map(LendWrapper::new)
        })
    }
}


impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    /// Construct an [`AttachableRefFull`] with the given slot, with no actual self-reference
    /// to the given `data: Data` backing data.
    ///
    /// (Self-references may later be added through mutations of the returned `Self`.)
    #[inline]
    #[must_use]
    pub const fn unattached_slot(data: Data, slot: SelfRefSlot<'data, 'upper, N, R, M>) -> Self {
        let data = SpeedBump {
            speed_bump: data,
        };

        unsafe { Self::from_slot(data, slot) }
    }

    /// Construct an [`AttachableRefFull`] in the [`NoRef`] state, with no self-reference to
    /// the given `data: Data` backing data.
    ///
    /// [`NoRef`]: SelfRefCases::NoRef
    #[inline]
    #[must_use]
    pub const fn unattached(data: Data, no_ref: N) -> Self {
        Self::unattached_slot(data, SelfRefCases::NoRef(no_ref))
    }

    /// Construct an [`AttachableRefFull`] in the [`Ref`] state from an immutable/shared
    /// self-reference to the given `data: Data` backing data.
    ///
    /// The value returned by the `f` callback will be stored alongside the `data` value in
    /// a self-referential struct; the complicated trait bound prevents unsoundness.
    ///
    /// See [`AttachableRefFull::try_attach_ref_with`] for fallible initialization or
    /// [`AttachableRefFull::attach_mut_with`] for mutable/exclusive self-references.
    ///
    /// Additionally, [`AttachableRefFull::attach_ref`] is available as a common special case.
    ///
    /// The provided callback can access short-lived data, mutate its environment, and so on; the
    /// trait bounds enforce that its return value's `'stable` data consists only of:
    /// - self-references to the given `data: Data` value obtained via [`StableView`], and/or
    /// - data that could live for at least `'data` (i.e., that met a `'data` bound).
    ///
    /// (The return value may also contain non-`'stable` data using other lifetimes in `R`.)
    ///
    /// # Details
    ///
    /// ## Compiler bug workarounds
    ///
    /// See the documentation of [`OutlivesChain`] and [`LendWrapper`]. Someday, those types
    /// should be unnecessary here.
    ///
    /// ## Bounds
    ///
    /// The bound on the view kind requires that `Data` values can be viewed through
    /// `DefaultViewKind` for any lifetime `'a` which is at most as long as `'data`.
    ///
    /// The bound on `F` requires that `f` can convert an immutable/shared `'stable` view to `Data`
    /// into an `R<'stable>` self-reference. `f` is required to work for any `'a` and `'stable`
    /// lifetimes such that `'a: 'stable` and `'stable: 'data`.
    ///
    /// Note that `R` is short for "Ref".
    ///
    /// ## Lifetimes in the Self-Reference
    ///
    /// We need to ensure that the operations described by [`StableView`] on the source `Data`
    /// do not invalidate the `R<'stable>` self-reference while `'data` has not ended.
    ///
    /// First, we consider data with a `'a` lifetime. Because the range which `'a` varies over has
    /// no lower bound, it could be arbitrarily short; in particular, it could be shorter than
    /// `'stable`, `'data`, or any lifetime in `R`. As a result, `'a` could be shorter than
    /// any lifetimes in `R<'stable>`, so it cannot appear in the produced `R<'stable>` value.
    /// (Data *derived* from `'a` data obtained from the view could still make it into the
    /// `R<'stable>` value; in particular, that would be possible for data contravariant over `'a`.)
    ///
    /// Next, consider data with a `'stable` lifetime. Such data must be covariant over `'stable`
    /// by the bounds on `R`; therefore, such data must either derive from the input `'stable`
    /// data from the view or have been covariantly shortened from some longer lifetime.
    ///
    /// The view's `'stable` data is required to not be invalidated by the described by
    /// [`StableView`] on the source `Data` while `'data` has not ended. Any `'stable`
    /// data derived from the view's `'stable` data should not be invalidated any sooner.
    ///
    /// Since `f` is required to work when `'stable = 'data`, any `'stable` data in `f`'s
    /// returned `R<'stable>` value obtained via covariant lifetime shortening (rather than from
    /// the view) must in fact be valid for at least `'data`. (The borrow checker can't somehow see
    /// whether the data is in fact borrowed for any shorter than `'data`, and neither should any
    /// `unsafe` code in `f` assume so; the bounds enforce that `'stable` **truly could** be
    /// `'data`.) Therefore, such data won't be invalidated while `'data` has not ended, regardless
    /// of what is done to the source `Data`.
    ///
    /// This fact holds even when `F` does not outlive `'data` (which *is* a possible scenario).
    ///
    /// Note that the possibility of `'data` references getting into the `R<'stable>` value implies
    /// that we must not assign a lifetime longer than `'data` to `'stable`; even though
    /// `R<'upper>` would be well-formed, it might be left dangling after `'data` ends.
    ///
    /// [`StableView`]: stable_view::StableView
    /// [`Ref`]: SelfRefCases::Ref
    #[inline]
    #[must_use]
    pub fn attach_ref_with<F>(data: Data, f: F) -> Self
    where
        F: for<'a, 'stable> FnOnce(
            StableViewer<'a, 'stable, 'data, Data>,
            OutlivesChain<'data, 'stable, 'a>,
        ) -> LendWrapper<'stable, 'upper, R>,
    {
        let Ok(this) = Self::try_attach_ref_with::<_, Infallible>(
            data,
            |viewer, outlives| Ok(f(viewer, outlives)),
        );
        this
    }

    /// Try to construct an [`AttachableRefFull`] in the [`Ref`] state with an immutable/shared
    /// self-reference to the given `data: Data` backing data.
    ///
    /// See also [`AttachableRefFull::try_attach_ref`] as a common special case.
    ///
    /// # Details
    /// All details are essentially the same as [`AttachableRefFull::attach_ref_with`]. Note that
    /// `'a` and `'stable` can be arbitrarily short, while `E` is a fixed type; therefore,
    /// references to the `data: Data` value cannot end up in the returned `E` value.
    ///
    /// # Errors
    /// Any error from the callback producing the self-reference is passed up.
    ///
    /// [`Ref`]: SelfRefCases::Ref
    #[inline]
    pub fn try_attach_ref_with<F, E>(data: Data, f: F) -> Result<Self, TryAttachError<Data, E>>
    where
        F: for<'a, 'stable> FnOnce(
            StableViewer<'a, 'stable, 'data, Data>,
            OutlivesChain<'data, 'stable, 'a>,
        ) -> Result<LendWrapper<'stable, 'upper, R>, E>,
    {
        let data = SpeedBump {
            speed_bump: data,
        };

        // Extra scope, to doubly make sure that if `f(viewer)` unwinds,
        // any references to `data` are necessarily dropped before `data` is dropped.
        let result = {
            let viewer = unsafe { StableViewer::new(&data.speed_bump) };
            f(viewer, OutlivesChain::new())
        };

        // WARNING: For the rest of this function, we should not unwind, since I haven't checked
        // the exact drop order. We need `data` to be dropped after any references to it.

        let slot = match result {
            Ok(self_ref) => SelfRefCases::Ref(self_ref.into_lend()),
            Err(error)   => {
                // There are no references to the contents of `data`.
                let data = data.speed_bump;
                return Err(TryAttachError { data, error });
            },
        };

        let this = unsafe { Self::from_slot(data, slot) };

        Ok(this)
    }

    /// Construct an [`AttachableRefFull`] in the [`RefMut`] state with a mutable/exclusive
    /// self-reference to the given `data: Data` backing data.
    ///
    /// The value returned by the `f` callback will be stored alongside the `data` value in
    /// a self-referential struct; the complicated trait bound prevents unsoundness.
    ///
    /// See [`AttachableRefFull::try_attach_mut_with`] for fallible initialization or
    /// [`AttachableRefFull::attach_ref_with`] for immutable/shared self-references.
    /// [`AttachableRefFull::attach_ref_with`] also describes more details about how this function's
    /// trait bounds work.
    ///
    /// Additionally, [`AttachableRefFull::attach_mut`] is available as a common special case.
    ///
    /// The provided callback can access short-lived data, mutate its environment, and so on; the
    /// trait bounds enforce that its return value's `'stable` data consists only of:
    /// - self-references to the given `data: Data` value, obtained via [`StableView`] or
    ///   [`StableViewMut`], and/or
    /// - data that could live for at least `'data` (i.e., that met a `'data` bound).
    ///
    /// (The return value may also contain non-`'stable` data using other lifetimes in `M`.)
    ///
    /// [`RefMut`]: SelfRefCases::RefMut
    /// [`StableView`]: stable_view::StableView
    /// [`StableViewMut`]: stable_view::StableViewMut
    #[inline]
    #[must_use]
    pub fn attach_mut_with<F>(data: Data, f: F) -> Self
    where
        F: for<'a, 'stable> FnOnce(
            StableViewerMut<'a, 'stable, 'data, Data>,
            OutlivesChain<'data, 'stable, 'a>,
        ) -> LendWrapper<'stable, 'upper, M>,
    {
        let Ok(this) = Self::try_attach_mut_with::<_, Infallible>(
            data,
            |viewer, outlives| Ok(f(viewer, outlives)),
        );
        this
    }

    /// Try to construct an [`AttachableRefFull`] in the [`RefMut`] state with a mutable/exclusive
    /// self-reference to the given `data: Data` backing data.
    ///
    /// See also [`AttachableRefFull::try_attach_mut`] as a common special case.
    ///
    /// # Errors
    /// Any error from the callback producing the self-reference is passed up.
    ///
    /// [`RefMut`]: SelfRefCases::RefMut
    #[inline]
    pub fn try_attach_mut_with<F, E>(data: Data, f: F) -> Result<Self, TryAttachError<Data, E>>
    where
        F: for<'a, 'stable> FnOnce(
            StableViewerMut<'a, 'stable, 'data, Data>,
            OutlivesChain<'data, 'stable, 'a>,
        ) -> Result<LendWrapper<'stable, 'upper, M>, E>,
    {
        let mut data = SpeedBump {
            speed_bump: data,
        };

        // Extra scope, to doubly make sure that if `f(viewer)` unwinds,
        // any references to `data` are necessarily dropped before `data` is dropped.
        let result = {
            let viewer = unsafe { StableViewerMut::new(&mut data.speed_bump) };
            f(viewer, OutlivesChain::new())
        };

        // WARNING: For the rest of this function, we should not unwind, since I haven't checked
        // the exact drop order. We need `data` to be dropped after any references to it.

        let slot = match result {
            Ok(self_ref_mut) => SelfRefCases::RefMut(self_ref_mut.into_lend()),
            Err(error)       => {
                // There are no references to the contents of `data`.
                let data = data.speed_bump;
                return Err(TryAttachError { data, error });
            }
        };

        let this = unsafe { Self::from_slot(data, slot) };

        Ok(this)
    }
}
