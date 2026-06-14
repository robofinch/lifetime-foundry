//! Temporary organization module.

use core::{marker::PhantomData, mem::transmute};

use variance_family::LendFamily;

use crate::map_slot::MappedSlot;
use crate::slot::{ErasedSelfRefSlot, SelfRefSlot};


/// Out of *extra* paranoia, disable any accidental `Debug`ing, `Clone`ing, or other immutable
/// access to `Data` (which would invalidate mutable self-references).
#[repr(transparent)]
pub(crate) struct SpeedBump<Data: ?Sized> {
    /// A `Data` value that needs to be handled carefully.
    pub speed_bump: Data,
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
    reason = "For now, `AttachableRefFull` is implemented across the `super` module",
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
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    /// Internal version of [`Self::from_slot_unchecked`], using [`SpeedBump`] for better
    /// visibility of `Data` manipulations.
    ///
    /// # Safety
    /// See [`stable_view::concepts_and_safety`] for the "stable" jargon used below.
    ///
    /// The exact safety requirements of this function depend on the state of `slot`.
    ///
    /// If `slot` is in the [`NoRef`] state, then it ***must*** not contain stable (non-long-lived)
    /// data.
    ///
    /// If `slot` is in the [`Ref`] state, then it any stable data it contains must not be
    /// invalidated by performing the three kinds of operations permitted by [`StableView::view`]
    /// on `data`. (Technically, then, that stable data need not have been obtained from
    /// [`stable_view::StableView`] views of `data`, but it needs to behave as though it were.)
    ///
    /// If `slot` is in the [`RefMut`] state, then it any stable data it contains must not be
    /// invalidated by performing the three kinds of operations permitted by
    /// [`StableViewMut::view_mut`] on `data`. (That stable data need not have been obtained from
    /// [`stable_view::StableViewMut`] views of `data`, but it needs to behave as though it were.)
    ///
    /// # Robust Guarantee
    /// This function only moves `data`, and does not unwind (which could cause `data` to be
    /// unexpectedly dropped). Therefore, it does not invalidate any `'stable` data in `slot`.
    ///
    /// [`NoRef`]: crate::slot::SelfRefCases::NoRef
    /// [`Ref`]: crate::slot::SelfRefCases::Ref
    /// [`RefMut`]: crate::slot::SelfRefCases::RefMut
    /// [`StableView::view`]: stable_view::StableView::view
    /// [`StableViewMut::view_mut`]: stable_view::StableViewMut::view_mut
    #[inline]
    #[must_use]
    pub(super) const unsafe fn from_slot<'stable: 'stable>(
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

    /// Internal version of [`Self::from_slot_unchecked`], using [`SpeedBump`] for better
    /// visibility of `Data` manipulations, and avoiding unnecessary safety comments for temporarily
    /// converting the [`MappedSlot`] into a [`SelfRefSlot`].
    ///
    /// # Safety
    /// TODO.
    ///
    /// # Robust Guarantee
    /// This function only moves `data`, and does not unwind (which could cause `data` to be
    /// unexpectedly dropped). Therefore, it does not invalidate any `'stable` data in `slot`.
    #[inline]
    #[must_use]
    pub(crate) unsafe fn from_mapped_slot(
        data: SpeedBump<Data>,
        slot: MappedSlot<'_, 'data, 'upper, N, R, M>,
    ) -> Self {
        let erased = unsafe { slot.into_erased_slot() };

        Self {
            slot:     erased,
            variance: PhantomData,
            data,
        }
    }
}

impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      for<'u> LendFamily<&'u ()>,
    M:      for<'u> LendFamily<&'u ()>,
{
    /// Freely change the `'upper` parameter (provided that the `'upper: 'data` bound is still met).
    ///
    /// This is sound because the types to which [`Lend<'stable, &'upper (), R>`] and
    /// [`Lend<'stable, &'upper (), M>`] normalize never actually use `'upper`. As long as those
    /// types are well-formed for any `'stable` such that `'data: 'stable`, the exact `'upper`
    /// lifetime does not matter, and `AttachableRefFull` does not place any additional invariants
    /// on `'upper`. (However, some `InterestingType<AttachableRefFull<'_, 'upper, ..>>`
    /// types **could** trigger unsoundness if `'upper` is changed.)
    ///
    /// As documented by [`Lend`], the compiler is unaware of that fact (and development of
    /// [`variance-family`] included four failed attempts to work around that problem).
    ///
    /// [`variance-family`]: variance_family
    /// [`Lend<'stable, &'upper (), R>`]: variance_family::Lend
    /// [`Lend<'stable, &'upper (), M>`]: variance_family::Lend
    /// [`Lend`]: variance_family::Lend
    #[inline]
    #[must_use]
    pub fn change_upper<'new_upper>(self) -> AttachableRefFull<'data, 'new_upper, N, R, M, Data>
    where
        'new_upper: 'data,
    {
        unsafe {
            transmute::<
                AttachableRefFull<'data, 'upper, N, R, M, Data>,
                AttachableRefFull<'data, 'new_upper, N, R, M, Data>,
            >(self)
        }
    }

    /// Freely change the `'upper` parameter (provided that the `'upper: 'data` bound is still met).
    ///
    /// See [`Self::change_upper`] for details. Note that `&Self` cares about the ability to read
    /// values of type `Self`, but does not place any additional invariants on `'upper`.
    ///
    /// (Some `InterestingType<AttachableRefFull<'_, 'upper, ..>>` types **could** trigger
    /// unsoundness if `'upper` is changed.)
    #[inline]
    #[must_use]
    pub fn change_upper_by_ref<'a, 'new_upper>(
        &'a self,
    ) -> &'a AttachableRefFull<'data, 'new_upper, N, R, M, Data>
    where
        'new_upper: 'data,
    {
        unsafe {
            transmute::<
                &'a AttachableRefFull<'data, 'upper, N, R, M, Data>,
                &'a AttachableRefFull<'data, 'new_upper, N, R, M, Data>,
            >(self)
        }
    }

    /// Freely change the `'upper` parameter (provided that the `'upper: 'data` bound is still met).
    ///
    /// See [`Self::change_upper`] for details. Note that `&mut Self` cares about the ability to
    /// read and write values of type `Self`, but does not place any additional invariants on
    /// `'upper`.
    ///
    /// (Some `InterestingType<AttachableRefFull<'_, 'upper, ..>>` types **could** trigger
    /// unsoundness if `'upper` is changed.)
    #[inline]
    #[must_use]
    pub fn change_upper_by_mut<'a, 'new_upper>(
        &'a mut self,
    ) -> &'a mut AttachableRefFull<'data, 'new_upper, N, R, M, Data>
    where
        'new_upper: 'data,
    {
        unsafe {
            transmute::<
                &'a mut AttachableRefFull<'data, 'upper, N, R, M, Data>,
                &'a mut AttachableRefFull<'data, 'new_upper, N, R, M, Data>,
            >(self)
        }
    }
}
