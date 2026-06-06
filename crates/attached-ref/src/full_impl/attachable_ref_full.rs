#![expect(unsafe_code, reason = "perform unsafe lifetime erasure and extension of self-refs")]
#![warn(
    clippy::missing_inline_in_public_items,
    reason = "this is basically a generic wrapper type",
)]

#![expect(clippy::undocumented_unsafe_blocks, reason = "TODO")]

use core::{convert::Infallible, marker::PhantomData};
use core::{
    fmt::{Debug, Formatter, Result as FmtResult},
    hint::{assert_unchecked, unreachable_unchecked},
};

use stable_view::{CustomView, CustomViewMut, DefaultViewKind, StableClone, StableView, StableViewMut};
use variance_family::{Lend, LendFamily, Unvarying, Varying};

use crate::{erased_slot::ErasedSelfRefSlot, error::TryAttachError};
use crate::{
    closure_traits::{ViewMutToLend, ViewToLend},
    slot::{SelfRefCases, SelfRefSlot},
};


/// Out of *extra* paranoia, disable any accidental `Debug`ing, `Clone`ing, or other immutable
/// access to `Data` (which would invalidate mutable self-references).
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "used to implement four methods of `AttachableRefFull` in `super::map_{full, slot}`",
)]
#[repr(transparent)]
pub(super) struct SpeedBump<Data: ?Sized> {
    pub(super) speed_bump_inner: Data,
}

/// # Robust Guarantee
/// This type semantically allows both covariant and contravariant casts of its `'upper`
/// parameter. That is, in many covariant, contravariant, and even invariant positions, the
/// `'upper` lifetime can be changed to any other lifetime (such that the where-bounds of this
/// struct still hold).
///
/// More precisely, `AttachableRefFull<'data, 'u1, N, R, M, Data>` can be soundly transmuted to
/// `AttachableRefFull<'data, 'u2, N, R, M, Data>`.
///
/// Notably, it is *not* generally the case that
/// `GenericType<AttachableRefFull<'data, 'u1, N, R, M, Data>>` can be soundly transmuted to
/// `GenericType<AttachableRefFull<'data, 'u2, N, R, M, Data>>`, since
/// `<AttachableRefFull<'data, 'u1, N, R, M, Data> as Trait>::Assoc` cannot generally be soundly
/// transmuted to `<AttachableRefFull<'data, 'u2, N, R, M, Data> as Trait>::Assoc`, and
/// `GenericType` may contain an associated type dependent on the exact `'erased` lifetime.
///
/// However -- perhaps barring questionable generativity-ish patterns reliant on references instead
/// of custom guard types -- references to this struct (such as `&`, `&mut`, `&&&&`, or
/// `&mut &&&mut` references) merely enable reading and writing values of this struct. Changing
/// the `'u1` lifetime of this struct under one or more nested references to `'u2` means that
/// reads and writes effectively perform transmutes between
/// `AttachableRefFull<'data, 'u1, N, R, M, Data>` and
/// `AttachableRefFull<'data, 'u2, N, R, M, Data>`, which is sound.
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "used to implement four methods of `AttachableRefFull` in `super::map_{full, slot}`",
)]
pub struct AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    Data:   ?Sized,
{
    /// # Safety of Use
    /// This is a lifetime-erased `SelfRefSlot<'stable, 'erased, N, R, M>`.
    ///
    /// ## Dropping
    ///
    /// Until destruction, it must be initialized for some `'stable`, though within the drop
    /// glue of this type, it is dropped and therefore briefly uninitialized, I suppose.
    /// Additionally, since `self.data` is dropped later in the drop glue (which can leave some
    /// parts of `self.slot` not `dereferenceable`), this field currently needs to be wrapped in
    /// `MaybeUninit` to avoid violating the protectors of references passed as arguments to the
    /// drop glue function. (At some point, `MaybeDangling` would be nice.)
    ///
    /// Since Rust is specified to drop a struct's field in order from first to last, it is critical
    /// that `self.slot` appear before `self.data`, so that any references in `self.slot` do not
    /// dangle *while* it's being dropped. (They may only dangle in the brief window where
    /// `self.slot` is dropped and `self` is still being destructed.)
    ///
    /// ## Contained References
    ///
    /// For writing `self.slot` -- noting that exposing a `&mut` reference to the self-ref slot
    /// generally allows both reads and writes -- `'stable` data written to `self.slot` must
    /// either be self-references to `self.data` *or* be valid for at least `'data`.
    ///
    /// Since `self.slot`'s data is (semantically) covariant over the `'stable` lifetime parameter,
    /// this implies the following robust guarantee.
    ///
    /// # Robust Guarantees
    ///
    /// ## Unerasure
    /// When reading `self.slot`, the erased lifetime can be unerased to any `'stable` lifetime
    /// such that `'data: 'stable` and, at least until `'stable` ends, `self.data` is not
    /// manipulated in a way that invalidates `self.slot`.
    ///
    /// ## Changing `'upper`
    /// See the robust guarantee of [`ErasedSelfRefSlot`] about `'erased`.
    pub(super) slot:     ErasedSelfRefSlot<'upper, N, R, M>,
    /// Make this struct covariant over `'data`, and ensure invariance over `R` and `M`.
    ///
    /// (The latter should already be guaranteed anyway, but it can't hurt to be doubly-sure.)
    ///
    /// Note that this struct is also covariant over `Data`.
    pub(super) variance: PhantomData<fn(*mut R, *mut M) -> &'data ()>,
    /// # Safety Invariant
    /// TODO: revamping semantics of `stable-view` rn.
    pub(super) data:     SpeedBump<Data>,
}

impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    'upper: 'data,
{
    #[inline]
    #[must_use]
    const fn from_long_lived_slot(
        data: Data,
        slot: SelfRefSlot<'data, 'upper, N, R, M>,
    ) -> Self {
        let data = SpeedBump {
            speed_bump_inner: data,
        };

        unsafe { Self::from_slot(data, slot) }
    }

    /// # Safety
    /// Any `'stable` data in `slot` must consist only of:
    /// - references to the given `data: Data` value, obtained via [`StableView::view`], and/or
    /// - data that lives for at least `'data`.
    ///
    /// # Robust Guarantee
    /// This function only moves `data`, and does not unwind (which could cause `data` to be
    /// unexpectedly dropped). Therefore, it does not invalidate any `'stable` data in `slot`.
    #[inline]
    #[must_use]
    const unsafe fn from_slot<'stable: 'stable>(
        data: SpeedBump<Data>,
        slot: SelfRefSlot<'stable, 'upper, N, R, M>,
    ) -> Self {
        let erased = unsafe { ErasedSelfRefSlot::erase(slot) };

        Self {
            slot:     erased,
            variance: PhantomData,
            data,
        }
    }

    /// Construct an [`AttachableRefFull`] in the [`NoRef`] state, with no self-reference to
    /// the given `data: Data` backing data.
    ///
    /// [`NoRef`]: SelfRefCases::NoRef
    #[inline]
    #[must_use]
    pub const fn unattached(data: Data, no_ref: N) -> Self {
        Self::from_long_lived_slot(data, SelfRefCases::NoRef(no_ref))
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

    /// Deconstruct `self` into owned pieces, including `Data`. The provided callback is prevented
    /// from returning self-referential data (since self-references to `Data` could be invalidated
    /// after this function returns).
    #[inline]
    #[must_use]
    pub fn into_pieces<F, T>(self, f: F) -> (T, Data)
    where
        // Ranges over `'stable` such that `'stable: data`.
        //
        // **Critically**, there are no implied lower bounds on `'stable`, despite `T`
        // potentially causing some concern. See the reasoning of `ViewToLend`.
        F: for<'stable> FnOnce(
            SelfRefSlot<'stable, 'upper, N, R, M>,
            PhantomData<&'stable &'data ()>,
        ) -> T,
    {
        // Extra scope, to make sure that if `f(slot, PhantomData)` unwinds,
        // any references to `data` are necessarily dropped before `data` is dropped.
        let output = {
            let slot = unsafe { self.slot.into_unerased() };

            f(slot, PhantomData)
        };

        (output, self.data.speed_bump_inner)
    }

    /// Unsafely get the backing `Data` and the slot for self-references of `self`.
    ///
    /// After calling this function, `Data` could be mutated in a way that invalidates any
    /// self-references in the slot, possibly causing undefined behavior.
    ///
    /// You generally **should not** use this method; it is used internally to implement *safe*
    /// methods to retrieve the inner pieces. It is exposed mainly in case another library author
    /// experienced with `unsafe` wants to implement similar methods.
    ///
    /// # Safety
    /// The slot for self-references, `self.get()`, **must** currently have no `'stable`
    /// self-references to the `Data` value of `self`.
    ///
    /// This safety condition is certainly met when `Data` is a value like `()` or
    /// `Option::None`, or when `self` is in the [`NoRef`] state.
    ///
    /// [`NoRef`]: SelfRefCases::NoRef
    #[inline]
    #[must_use]
    pub unsafe fn into_raw_pieces(self) -> (SelfRefSlot<'data, 'upper, N, R, M>, Data) {
        // SAFETY: As robustly guaranteed by the `slot` field, the erased lifetime can be soundly
        // unerased into any `'stable` lifetime such that `'data: 'stable` and, at least until
        // `'stable` ends, `self.data` is not manipulated in a way that invalidates `self.slot`.
        // Our caller has `unsafe`ly asserted that `self.slot` has no `'stable` references to
        // `self.data`, so *no* manipulation of `self.data` (during at least `'data`) can invalidate
        // `self.slot`. Therefore, we can soundly choose `'stable = 'data`.
        let slot = unsafe { self.slot.into_unerased::<'data>() };

        // SAFETY INVARIANT: Our caller has `unsafe`ly asserted that `self.slot` has no `'stable`
        // references to `self.data`, so *no* manipulation of `self.data` (during at least `'data`)
        // can invalidate the `slot` value. Therefore, completely exposing `self.data` to
        // the caller's code is sound.
        let data = self.data.speed_bump_inner;

        (slot, data)
    }

    /// Get the backing `Data` by value, dropping the potentially self-referential data.
    #[inline]
    #[must_use]
    pub fn into_data(self) -> Data {
        drop(self.slot);
        // There are no more references to `data`.
        self.data.speed_bump_inner
    }

    /// Attempt to get both the backing `Data` and [`NoRef`] data by value.
    ///
    /// # Errors
    /// This method succeeds if and only if the slot for self-references is in the [`NoRef`] state.
    ///
    /// On error, the given `self` is passed back.
    ///
    /// [`NoRef`]: SelfRefCases::NoRef
    #[inline]
    pub fn try_into_owned(self) -> Result<(N, Data), Self> {
        if matches!(self.get(), SelfRefCases::NoRef(_)) {
            let (slot, data) = unsafe { self.into_raw_pieces() };

            let no_ref = match slot {
                SelfRefCases::NoRef(no_ref) => no_ref,
                SelfRefCases::Ref(_) | SelfRefCases::RefMut(_) => {
                    unsafe { unreachable_unchecked() }
                }
            };

            Ok((no_ref, data))
        } else {
            Err(self)
        }
    }
}

impl<'data, 'upper, N, R, M, Data>
    AttachableRefFull<'data, 'upper, N, R, M, Option<Data>>
where
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    'upper: 'data,
{
    /// Construct a new [`AttachableRefFull`] in the [`Ref`] state without actually having
    /// self-references.
    ///
    /// TODO: This can be paired with `wrap_data_in_option` to mix owned and borrowed data.
    ///
    /// See also [`AttachableRefFull::new_always_owned_ref`] if the possibility of borrowed data is
    /// not needed.
    ///
    /// [`Ref`]: SelfRefCases::Ref
    #[inline]
    #[must_use]
    pub const fn new_owned_ref(shared_ref: Lend<'data, &'upper (), R>) -> Self {
        Self::from_long_lived_slot(None, SelfRefCases::Ref(shared_ref))
    }

    /// Construct a new [`AttachableRefFull`] in the [`RefMut`] state without actually having
    /// self-references.
    ///
    /// TODO: This can be paired with `wrap_data_in_option` to mix owned and borrowed data.
    ///
    /// See also [`AttachableRefFull::new_always_owned_mut`] if the possibility of borrowed data is
    /// not needed.
    ///
    /// [`RefMut`]: SelfRefCases::RefMut
    #[inline]
    #[must_use]
    pub const fn new_owned_mut(exclusive_ref: Lend<'data, &'upper (), M>) -> Self {
        Self::from_long_lived_slot(None, SelfRefCases::RefMut(exclusive_ref))
    }

    /// If `Data` is [`None`], then this struct is not actually protecting any self-referential
    /// data, and the slot for self-references can be safely obtained by-value.
    ///
    /// # Errors
    /// If `Data` is `Some`, `self` is returned back.
    #[inline]
    pub fn try_into_owned_slot(self) -> Result<SelfRefSlot<'data, 'upper, N, R, M>, Self> {
        if self.data.speed_bump_inner.is_none() {
            let (slot, none) = unsafe { self.into_raw_pieces() };

            unsafe {
                assert_unchecked(none.is_none());
            };

            Ok(slot)
        } else {
            Err(self)
        }
    }
}

impl<'data, 'upper, N, R, M> AttachableRefFull<'data, 'upper, N, R, M, ()>
where
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    'upper: 'data,
{
    /// Construct a new [`AttachableRefFull`] in the [`Ref`] state which *cannot* be
    /// self-referential.
    ///
    /// This constructor is similar to [`AttachableRefFull::new_owned_ref`], but this type does
    /// not allow self-references. It may be useful in generic scenarios, though not as a concrete
    /// type.
    ///
    /// [`Ref`]: SelfRefCases::Ref
    #[inline]
    #[must_use]
    pub const fn new_always_owned_ref(shared_ref: Lend<'data, &'upper (), R>) -> Self {
        Self::from_long_lived_slot((), SelfRefCases::Ref(shared_ref))
    }

    /// Construct a new [`AttachableRefFull`] in the [`RefMut`] state which *cannot* be
    /// self-referential.
    ///
    /// This constructor is similar to [`AttachableRefFull::new_owned_mut`], but this type does
    /// not allow self-references. It may be useful in generic scenarios, though not as a concrete
    /// type.
    ///
    /// [`RefMut`]: SelfRefCases::RefMut
    #[inline]
    #[must_use]
    pub const fn new_always_owned_mut(exclusive_ref: Lend<'data, &'upper (), M>) -> Self {
        Self::from_long_lived_slot((), SelfRefCases::RefMut(exclusive_ref))
    }

    /// Get the data by-value.
    ///
    /// Since an [`AttachableRefFull<.., ()>`] does not allow self-references, no protection is
    /// actually needed for the self-reference slot.
    #[inline]
    #[must_use]
    pub fn into_owned_slot(self) -> SelfRefSlot<'data, 'upper, N, R, M> {
        let (slot, ()) = unsafe { self.into_raw_pieces() };

        slot
    }
}

impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    Data:   ?Sized,
    'upper: 'data,
{
    /// Obtain a valid immutable/shared reference to potentially self-referential data.
    #[inline]
    #[must_use]
    pub const fn get(&self) -> &SelfRefSlot<'_, 'upper, N, R, M> {
        unsafe { self.slot.unerase_ref() }
    }

    /// Obtain a valid immutable/shared reference to potentially self-referential data and, if
    /// possible, the backing data.
    ///
    /// If `self` is currently in the [`RefMut`] state (meaning that there could be a mutable
    /// self-reference to the backing data), the backing data is not accessed.
    ///
    /// [`RefMut`]: SelfRefCases::RefMut
    #[expect(clippy::type_complexity, reason = "it' a one-off type, and there's a `_full` suffix")]
    #[inline]
    #[must_use]
    pub const fn get_full(&self) -> SelfRefCases<
        (&N, &Data),
        (&Lend<'_, &'upper (), R>, &Data),
        &Lend<'_, &'upper (), M>,
    > {
        match self.get() {
            SelfRefCases::NoRef(no_ref)        => {
                SelfRefCases::NoRef((no_ref, &self.data.speed_bump_inner))
            }
            SelfRefCases::Ref(self_ref)        => {
                SelfRefCases::Ref((self_ref, &self.data.speed_bump_inner))
            }
            SelfRefCases::RefMut(self_ref_mut) => SelfRefCases::RefMut(self_ref_mut),
        }
    }

    /// Attempt to obtain a valid immutable/shared reference to the backing data, without
    /// invalidating any self-references.
    ///
    /// If `self` is currently in the [`RefMut`] state (meaning that there could be a mutable
    /// self-reference to the backing data), the backing data is not accessed and `None` is
    /// returned.
    ///
    /// [`RefMut`]: SelfRefCases::RefMut
    #[inline]
    #[must_use]
    pub const fn try_get_data(&self) -> Option<&Data> {
        match self.get() {
            SelfRefCases::NoRef(_) | SelfRefCases::Ref(_) => Some(&self.data.speed_bump_inner),
            SelfRefCases::RefMut(_) => None,
        }
    }
}

impl<'data, 'upper, N, R, Data> AttachableRefFull<'data, 'upper, N, R, Infallible, Data>
where
    R:      LendFamily<&'upper ()>,
    Data:   ?Sized,
    'upper: 'data,
{
    /// Obtain a valid immutable/shared reference to the backing data, without invalidating
    /// any self-references.
    #[inline]
    #[must_use]
    pub const fn get_data(&self) -> &Data {
        match *self.get() {
            SelfRefCases::NoRef(_) | SelfRefCases::Ref(_) => &self.data.speed_bump_inner,
            SelfRefCases::RefMut(infallible) => match infallible {},
        }
    }
}

impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    Data:   ?Sized,
    'upper: 'data,
{
    /// Mutably access the self-referential data and, if possible, the backing data.
    ///
    /// New data can be introduced into the stored self-referential data, so long as that new
    /// data lives for at least `'data`.
    #[inline]
    pub fn with_mut<'s, F, T>(&'s mut self, f: F) -> T
    where
        F: 'data + for<'a> FnOnce(
            SelfRefCases<
                (&'s mut N, &'s mut Data),
                &'a mut Lend<'s, &'upper (), R>,
                &'a mut Lend<'s, &'upper (), M>
            >,
            PhantomData<&'a &'s &'data ()>,
        ) -> T,
    {
        let unerased = unsafe { self.slot.unerase_mut() };
        let cases = match unerased {
            SelfRefCases::NoRef(no_ref)        => {
                SelfRefCases::NoRef((no_ref, &mut self.data.speed_bump_inner))
            }
            SelfRefCases::Ref(self_ref)        => SelfRefCases::Ref(self_ref),
            SelfRefCases::RefMut(self_ref_mut) => SelfRefCases::RefMut(self_ref_mut),
        };

        f(cases, PhantomData)
    }

    /// Attempt to obtain a mutable reference to the backing data and [`NoRef`] data.
    ///
    /// Returns `None` if `self` is not in the [`NoRef`] state.
    ///
    /// [`NoRef`]: SelfRefCases::NoRef
    #[inline]
    #[must_use]
    pub const fn try_get_mut(&mut self) -> Option<(&mut N, &mut Data)> {
        let unerased = unsafe { self.slot.unerase_mut() };

        if let SelfRefCases::NoRef(no_ref) = unerased {
            Some((no_ref, &mut self.data.speed_bump_inner))
        } else {
            None
        }
    }

    /// Attempt to obtain a mutable reference to the backing data, without invalidating any
    /// self-references.
    ///
    /// Returns `None` if `self` is not in the [`NoRef`] state.
    ///
    /// [`NoRef`]: SelfRefCases::NoRef
    #[inline]
    #[must_use]
    pub const fn try_get_data_mut(&mut self) -> Option<&mut Data> {
        if matches!(self.get(), SelfRefCases::NoRef(_)) {
            Some(&mut self.data.speed_bump_inner)
        } else {
            None
        }
    }

    // map self-refs and whatnot, and take/set them (switching between `NoRef` and `Ref(Mut)`)

    // safely map Data to something else if currently in state `NoRef`
    // pub fn change_data

    // change_data_return

    // If `Data` is `()`, set it.
    // pub fn set_data

    // If `Data` is currently `None`, set it. Else give back the value you tried to set.
    // pub fn try_set_data
}

impl<'data, 'upper, N, R, Data> Clone for AttachableRefFull<'data, 'upper, N, R, Infallible, Data>
where
    'upper: 'data,
    N:      Clone,
    R:      LendFamily<&'upper (), Is: Clone>,
    Data:   StableClone<'data>,
{
    #[inline]
    fn clone(&self) -> Self {
        let (slot, data) = match self.get_full() {
            SelfRefCases::NoRef((no_ref, data)) => (SelfRefCases::NoRef(no_ref.clone()), data),
            SelfRefCases::Ref((self_ref, data)) => (SelfRefCases::Ref(self_ref.clone()), data),
            SelfRefCases::RefMut(&infallible)  => match infallible {},
        };

        // Even if this panics and unwinds, the fact that `self.data` is immutably borrowed
        // for this whole function body implies that any self-references in `maybe_ref` are
        // not invalidated (before they are dropped).
        let data = SpeedBump {
            speed_bump_inner: data.clone(),
        };

        unsafe { Self::from_slot(data, slot) }
    }
}

impl<'data, 'upper, N, R, M, Data> Debug for AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    N:      Debug,
    R:      LendFamily<&'upper (), Is: Debug>,
    M:      LendFamily<&'upper (), Is: Debug>,
    Data:   ?Sized + Debug,
{
    #[expect(clippy::missing_inline_in_public_items, reason = "in formatting, size matters more")]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let data = self.try_get_data();

        let data_dbg: &dyn Debug = if let Some(data) = data.as_ref() {
            data
        } else {
            &format_args!("<exclusively borrowed>")
        };

        f.debug_struct("AttachableRefFull")
            .field("slot",     &self.get())
            .field("variance", &self.variance)
            .field("data",     data_dbg)
            .finish()
    }
}
