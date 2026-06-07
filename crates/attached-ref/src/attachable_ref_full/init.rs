#![expect(unsafe_code, reason = "perform unsafe lifetime erasure and extension of self-refs")]

use core::convert::Infallible;

use stable_view::{CustomView, CustomViewMut, DefaultViewKind, StableView, StableViewMut};
use variance_family::{LendFamily, Unvarying, Varying};

use crate::slot::{SelfRefCases, SelfRefSlot};
use crate::init_support::{TryAttachError, ViewMutToLend, ViewToLend};
use super::full_struct::{AttachableRefFull, SpeedBump};


impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    /// Construct an [`AttachableRefFull`] in the [`NoRef`] state, with no self-reference to
    /// the given `data: Data` backing data.
    ///
    /// [`NoRef`]: SelfRefCases::NoRef
    #[inline]
    #[must_use]
    pub const fn unattached(data: Data, no_ref: N) -> Self {
        Self::unattached_slot(data, SelfRefCases::NoRef(no_ref))
    }

    /// Construct an [`AttachableRefFull`] with the given slot, with no actual self-reference
    /// to the given `data: Data` backing data.
    ///
    /// (Self-references may later be added through mutations of the returned `Self`.)
    #[inline]
    #[must_use]
    pub const fn unattached_slot(data: Data, slot: SelfRefSlot<'data, 'upper, N, R, M>) -> Self {
        let data = SpeedBump {
            speed_bump_inner: data,
        };

        unsafe { Self::from_slot(data, slot) }
    }

    /// Construct an [`AttachableRefFull`] in the [`Ref`] state with an immutable/shared
    /// self-reference to the given `data: Data` backing data.
    ///
    /// The provided callback can access short-lived data, mutate its environment, and so on; the
    /// trait bounds enforce that its return value's `'stable` data consists only of:
    /// - self-references to the given `data: Data` value, obtained via [`StableView::view`], and/or
    /// - data that lives for at least `'data`.
    ///
    /// (The return value may also contain non-`'stable` data using other lifetimes in `R`.)
    ///
    /// See [`AttachableRefFull::try_attach_ref`] for fallible initialization,
    /// [`AttachableRefFull::attach_ref_with`] to use a non-default view kind, or
    /// [`AttachableRefFull::attach_mut`] for mutable/exclusive self-references.
    ///
    /// # Details
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
    /// [`Ref`]: SelfRefCases::Ref
    #[inline]
    #[must_use]
    pub fn attach_ref<F>(data: Data, f: F) -> Self
    where
        DefaultViewKind: for<'a> StableView<'a, 'data, Data>,
        F:     ViewToLend<'data, 'upper, Data, DefaultViewKind, R>,
        'data: 'upper,
    {
        Self::attach_ref_with::<DefaultViewKind, F>(data, f)
    }

    /// Try to construct an [`AttachableRefFull`] in the [`Ref`] state with an immutable/shared
    /// self-reference to the given `data: Data` backing data.
    ///
    /// # Errors
    /// Any error from the callback producing the self-reference is passed up.
    ///
    /// [`Ref`]: SelfRefCases::Ref
    #[inline]
    pub fn try_attach_ref<Error, F>(data: Data, f: F) -> Result<Self, TryAttachError<Data, Error>>
    where
        DefaultViewKind: for<'a> StableView<'a, 'data, Data>,
        F: ViewToLend<'data, 'upper, Data, DefaultViewKind, Result<R, Unvarying<Error>>>,
    {
        Self::try_attach_ref_with::<DefaultViewKind, Error, F>(data, f)
    }

    /// Construct an [`AttachableRefFull`] in the [`Ref`] state with an immutable/shared
    /// self-reference to the given `data: Data` backing data.
    ///
    /// The provided callback can access short-lived data, mutate its environment, and so on; the
    /// trait bounds enforce that its return value's `'stable` data consists only of:
    /// - self-references to the given `data: Data` value, obtained via `StableView::view`, and/or
    /// - data that lives for at least `'data`.
    ///
    /// (The return value may also contain non-`'stable` data using other lifetimes in `R`.)
    ///
    /// The second argument of the `FnOnce` bound on `F` is simply to allow `'stable` to be
    /// mentioned in its return type; that argument can be ignored.
    ///
    /// See [`AttachableRefFull::try_attach_ref_with`] for fallible initialization.
    ///
    /// [`Ref`]: SelfRefCases::Ref
    #[inline]
    #[must_use]
    pub fn attach_ref_with<V, F>(data: Data, f: F) -> Self
    where
        V: for<'a> StableView<'a, 'data, Data>,
        F: ViewToLend<'data, 'upper, Data, V, R>,
    {
        /// Polyfill for `|view, _phantom| Ok(f.view_to_lend(view))`,
        /// which only works (in the below scenario) in Rust 1.94.0 and above.
        #[derive(Debug)]
        #[repr(transparent)]
        struct InfallibleF<F>(F);

        impl<'data, 'upper, Data, V, R, F>
            ViewToLend<'data, 'upper, Data, V, Result<R, Unvarying<Infallible>>>
        for InfallibleF<F>
        where
            'upper: 'data,
            Data:   ?Sized,
            V:      for<'a> StableView<'a, 'data, Data>,
            R:      LendFamily<&'upper ()>,
            F:      ViewToLend<'data, 'upper, Data, V, R>,
        {
            #[inline]
            fn view_to_lend<'a, 'stable>(
                self,
                view: CustomView<'a, 'stable, 'data, Data, V>,
            ) -> Varying<'stable, 'stable, &'upper (), Result<R, Unvarying<Infallible>>>
            where
                'data:   'stable,
                'stable: 'a
            {
                Ok(self.0.view_to_lend(view))
            }
        }

        let Ok(this) = Self::try_attach_ref_with(data, InfallibleF(f));

        this
    }

    /// Try to construct an [`AttachableRefFull`] in the [`Ref`] state with an immutable/shared
    /// self-reference to the given `data: Data` backing data.
    ///
    /// # Details
    /// All details are essentially the same as [`AttachableRefFull::attach_ref_with`]. Note that
    /// `'a` and `'stable` can be arbitrarily short, while `Error` is a fixed type; therefore,
    /// references to the `data: Data` value cannot end up in the returned `Error` value.
    ///
    /// # Errors
    /// Any error from the callback producing the self-reference is passed up.
    ///
    /// [`Ref`]: SelfRefCases::Ref
    #[inline]
    pub fn try_attach_ref_with<V, Error, F>(
        data: Data,
        f:    F,
    ) -> Result<Self, TryAttachError<Data, Error>>
    where
        V: for<'a> StableView<'a, 'data, Data>,
        F: ViewToLend<'data, 'upper, Data, V, Result<R, Unvarying<Error>>>,
    {
        let data = SpeedBump {
            speed_bump_inner: data,
        };

        // Extra scope, to make sure that if `V::view` or `f.view_to_lend(view)` unwinds,
        // any references to `data` are necessarily dropped before `data` is dropped.
        let result = {
            let view = unsafe { V::view(&data.speed_bump_inner) };
            f.view_to_lend(view)
        };

        // WARNING: For the rest of this function, we should not unwind, since I haven't checked
        // the exact drop order. We need `data` to be dropped after any references to it.

        let slot = match result {
            Ok(self_ref) => SelfRefCases::Ref(self_ref),
            Err(error)   => {
                // There are no references to the contents of `data`.
                let data = data.speed_bump_inner;
                return Err(TryAttachError { data, error });
            },
        };

        let this = unsafe { Self::from_slot(data, slot) };

        Ok(this)
    }

    /// Construct an [`AttachableRefFull`] in the [`RefMut`] state with a mutable/exclusive
    /// self-reference to the given `data: Data` backing data.
    ///
    /// The provided callback can access short-lived data, mutate its environment, and so on; the
    /// trait bounds enforce that its return value's `'stable` data consists only of:
    /// - self-references to the given `data: Data` value, obtained via [`StableViewMut::view_mut`],
    ///   and/or
    /// - data that lives for at least `'data`.
    ///
    /// (The return value may also contain non-`'stable` data using other lifetimes in `R`.)
    ///
    /// See [`AttachableRefFull::try_attach_mut`] for fallible initialization,
    /// [`AttachableRefFull::attach_mut_with`] to use a non-default view kind, or
    /// [`AttachableRefFull::attach_ref`] for immutable/shared self-references.
    ///
    /// [`AttachableRefFull::attach_ref`] also describes more details about how this function's
    /// trait bounds work.
    ///
    /// [`RefMut`]: SelfRefCases::RefMut
    #[inline]
    #[must_use]
    pub fn attach_mut<F>(data: Data, f: F) -> Self
    where
        DefaultViewKind: for<'a> StableViewMut<'a, 'data, Data>,
        F:     ViewMutToLend<'data, 'upper, Data, DefaultViewKind, M>,
        'data: 'upper,
    {
        Self::attach_mut_with::<DefaultViewKind, F>(data, f)
    }

    /// Try to construct an [`AttachableRefFull`] in the [`RefMut`] state with a mutable/exclusive
    /// self-reference to the given `data: Data` backing data.
    ///
    /// # Errors
    /// Any error from the callback producing the self-reference is passed up.
    ///
    /// [`RefMut`]: SelfRefCases::RefMut
    #[inline]
    pub fn try_attach_mut<Error, F>(data: Data, f: F) -> Result<Self, TryAttachError<Data, Error>>
    where
        DefaultViewKind: for<'a> StableViewMut<'a, 'data, Data>,
        F: ViewMutToLend<'data, 'upper, Data, DefaultViewKind, Result<M, Unvarying<Error>>>,
    {
        Self::try_attach_mut_with::<DefaultViewKind, Error, F>(data, f)
    }

    /// Construct an [`AttachableRefFull`] in the [`RefMut`] state with a mutable/exclusive
    /// self-reference to the given `data: Data` backing data.
    ///
    /// The provided callback can access short-lived data, mutate its environment, and so on; the
    /// trait bounds enforce that its return value's `'stable` data consists only of:
    /// - self-references to the given `data: Data` value, obtained via `StableViewMut::view_mut`,
    ///   and/or
    /// - data that lives for at least `'data`.
    ///
    /// (The return value may also contain non-`'stable` data using other lifetimes in `R`.)
    ///
    /// The second argument of the `FnOnce` bound on `F` is simply to allow `'stable` to be
    /// mentioned in its return type; that argument can be ignored.
    ///
    /// See [`AttachableRefFull::try_attach_mut_with`] for fallible initialization.
    ///
    /// [`RefMut`]: SelfRefCases::RefMut
    #[inline]
    #[must_use]
    pub fn attach_mut_with<V, F>(data: Data, f: F) -> Self
    where
        V: for<'a> StableViewMut<'a, 'data, Data>,
        F: ViewMutToLend<'data, 'upper, Data, V, M>,
    {
        /// Polyfill for `|view_mut, _phantom| Ok(f.view_mut_to_lend(view_mut))`,
        /// which only works (in the below scenario) in Rust 1.94.0 and above.
        #[derive(Debug)]
        #[repr(transparent)]
        struct InfallibleF<F>(F);

        impl<'data, 'upper, Data, V, M, F>
            ViewMutToLend<'data, 'upper, Data, V, Result<M, Unvarying<Infallible>>>
        for InfallibleF<F>
        where
            'upper: 'data,
            Data:   ?Sized,
            V:      for<'a> StableViewMut<'a, 'data, Data>,
            M:      LendFamily<&'upper ()>,
            F:      ViewMutToLend<'data, 'upper, Data, V, M>,
        {
            #[inline]
            fn view_mut_to_lend<'a, 'stable>(
                self,
                view_mut: CustomViewMut<'a, 'stable, 'data, Data, V>,
            ) -> Varying<'stable, 'stable, &'upper (), Result<M, Unvarying<Infallible>>>
            where
                'data:   'stable,
                'stable: 'a
            {
                Ok(self.0.view_mut_to_lend(view_mut))
            }
        }

        let Ok(this) = Self::try_attach_mut_with(data, InfallibleF(f));

        this
    }

    /// Try to construct an [`AttachableRefFull`] in the [`RefMut`] state with a mutable/exclusive
    /// self-reference to the given `data: Data` backing data.
    ///
    /// # Details
    /// All details are essentially the same as [`AttachableRefFull::attach_mut_with`]. Note that
    /// `'a` and `'stable` can be arbitrarily short, while `Error` is a fixed type; therefore,
    /// references to the `data: Data` value cannot end up in the returned `Error` value.
    ///
    /// # Errors
    /// Any error from the callback producing the self-reference is passed up.
    ///
    /// [`RefMut`]: SelfRefCases::RefMut
    #[inline]
    pub fn try_attach_mut_with<V, Error, F>(
        data: Data,
        f:    F,
    ) -> Result<Self, TryAttachError<Data, Error>>
    where
        V: for<'a> StableViewMut<'a, 'data, Data>,
        F: ViewMutToLend<'data, 'upper, Data, V, Result<M, Unvarying<Error>>>,
    {
        let mut data = SpeedBump {
            speed_bump_inner: data,
        };

        // Extra scope, to make sure that if `V::view_mut` or `f.view_mut_to_lend(view)` unwinds,
        // any references to `data` are necessarily dropped before `data` is dropped.
        let result = {
            let view_mut = unsafe { V::view_mut(&mut data.speed_bump_inner) };
            f.view_mut_to_lend(view_mut)
        };

        // WARNING: For the rest of this function, we should not unwind, since I haven't checked
        // the exact drop order. We need `data` to be dropped after any references to it.

        let slot = match result {
            Ok(self_ref_mut) => SelfRefCases::RefMut(self_ref_mut),
            Err(error)       => {
                // There are no references to the contents of `data`.
                let data = data.speed_bump_inner;
                return Err(TryAttachError { data, error });
            }
        };

        let this = unsafe { Self::from_slot(data, slot) };

        Ok(this)
    }
}
