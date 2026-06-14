#![expect(unsafe_code, reason = "Manage lifetime-branded `!Copy` token-based invariants")]

use core::{convert::Infallible, hint::assert_unchecked, marker::PhantomData};
use core::fmt::{Debug, Formatter, Result as FmtResult};

use stable_view::StableClone;
use variance_family::{LendFamily, LifetimeFamily, MaxUpperBound, Varying};

use crate::{
    attachable_ref_full::{AttachableRefFull, SpeedBump},
    slot::{SelfRefCases, SelfRefSlot},
};
use super::mapped_slot::MappedSlot;


pub trait BrandFamily: for<'lower> LifetimeFamily<'lower, MaxUpperBound, Is: Sized> {}

impl<T: for<'lower> LifetimeFamily<'lower, MaxUpperBound, Is: Sized>> BrandFamily for T {}

pub type Branded<'brand, T> = Varying<'brand, 'brand, MaxUpperBound, T>;

/// # Safety
/// See [`stable_view::concepts_and_safety`] for jargon.
///
/// The produced brands are associated with some unique `data: Data` value, any stable
/// self-references to it, and long-lived data (where "stable" and "long-lived" are with respect to
/// `'stable` and `'data`).
///
/// Those associations **must** be fixed across all usage of `'brand`.
///
/// For at least lifetime `'brand`, any stable self-references to the `data` (whether already
/// obtained, or obtained at some point soon) **must not** be invalidated.
#[inline]
#[must_use]
pub(super) const unsafe fn new_brand<'brand, 'stable, 'data>() -> (
    DataBrand<'brand>,
    SlotBrand<'brand, 'stable, 'data>,
) {
    (DataBrand(PhantomData), SlotBrand(PhantomData))
}

pub(super) struct DataBrand<'brand>(PhantomData<fn(*mut &'brand ())>);

pub(super) struct SlotBrand<'brand, 'stable, 'data>(
    PhantomData<fn(*mut &'brand (), *mut &'stable ()) -> &'data ()>,
);

pub struct TakeDataToken<'brand>(PhantomData<fn(*mut &'brand ())>);

impl TakeDataToken<'_> {
    #[inline]
    #[must_use]
    pub(super) const unsafe fn new() -> Self {
        Self(PhantomData)
    }
}

impl Debug for TakeDataToken<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("TakeDataToken").finish_non_exhaustive()
    }
}

pub struct NoRefMapToken<'brand, 'a, 'stable, 'data, Data> {
    /// # Safety Invariant
    /// The `data` to which `'brand` is associated must have no stable self-references to it,
    /// with respect to `'stable` and `'data`.
    _brand:    SlotBrand<'brand, 'stable, 'data>,
    /// # Safety Invariant
    /// Must be `&mut None` (except after set in [`Self::map`]).
    none_data: &'a mut Option<SpeedBump<Data>>,
}

impl<'brand, 'a, 'stable, 'data, Data> NoRefMapToken<'brand, 'a, 'stable, 'data, Data> {
    /// # Safety
    /// The `data` to which `'brand` is associated (see [`new_brand`]) must have no stable
    /// self-references to it, with respect to `'stable` and `'data`.
    /// (Therefore, any actions done to `data` do not invalidate stable self-references.)
    ///
    /// Additionally, `none_data` must be `&mut None`.
    #[inline]
    #[must_use]
    pub(super) const unsafe fn new(
        brand:     SlotBrand<'brand, 'stable, 'data>,
        none_data: &'a mut Option<SpeedBump<Data>>,
    ) -> Self {
        Self {
            // Safety invariant: the caller unsafely asserted it.
            _brand: brand,
            // Safety invariant: the caller unsafely asserted it.
            none_data,
        }
    }

    #[inline]
    #[must_use]
    pub fn map<'upper, N, R, M>(
        self,
        data: Data,
        slot: SelfRefSlot<'stable, 'upper, N, R, M>,
    ) -> MappedSlot<'brand, 'data, 'upper, N, R, M>
    where
        'upper: 'data,
        R:      LendFamily<&'upper ()>,
        M:      LendFamily<&'upper ()>,
    {
        // SAFETY: As per the safety invariant, `self.none_data` is currently `None`.
        unsafe {
            assert_unchecked(self.none_data.is_none());
        };
        *self.none_data = Some(SpeedBump {
            speed_bump: data
        });

        unsafe { MappedSlot::new(slot) }
    }

    #[inline]
    #[must_use]
    pub fn return_data(self, data: Data) -> TakeDataToken<'brand> {
        // SAFETY: As per the safety invariant, `self.none_data` is currently `None`.
        unsafe {
            assert_unchecked(self.none_data.is_none());
        };
        *self.none_data = Some(SpeedBump {
            speed_bump: data
        });

        unsafe { TakeDataToken::new() }
    }
}

impl<Data> Debug for NoRefMapToken<'_, '_, '_, '_, Data> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("NoRefMapToken").finish_non_exhaustive()
    }
}

pub struct RefMapToken<'brand, 'stable, 'data>(SlotBrand<'brand, 'stable, 'data>);

impl<'brand, 'stable, 'data> RefMapToken<'brand, 'stable, 'data> {
    #[inline]
    #[must_use]
    pub(super) const unsafe fn new(brand: SlotBrand<'brand, 'stable, 'data>) -> Self {
        Self(brand)
    }

    #[inline]
    #[must_use]
    pub const fn map<'upper, N, R, M>(
        self,
        slot: SelfRefSlot<'stable, 'upper, N, R, M>,
    ) -> MappedSlot<'brand, 'data, 'upper, N, R, M>
    where
        'upper: 'data,
        R:      LendFamily<&'upper ()>,
        M:      LendFamily<&'upper ()>,
    {
        unsafe { MappedSlot::new(slot) }
    }

    #[inline]
    #[must_use]
    pub const fn data_unused(self) -> TakeDataToken<'brand> {
        unsafe { TakeDataToken::new() }
    }
}

impl Debug for RefMapToken<'_, '_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("RefMapToken").finish_non_exhaustive()
    }
}

pub struct RefMutMapToken<'brand, 'stable, 'data>(SlotBrand<'brand, 'stable, 'data>);

impl<'brand, 'stable, 'data> RefMutMapToken<'brand, 'stable, 'data> {
    #[inline]
    #[must_use]
    pub(super) const unsafe fn new(brand: SlotBrand<'brand, 'stable, 'data>) -> Self {
        Self(brand)
    }

    #[inline]
    #[must_use]
    pub fn map<'upper, N, R, M>(
        self,
        slot: SelfRefSlot<'stable, 'upper, N, Infallible, M>,
    ) -> MappedSlot<'brand, 'data, 'upper, N, R, M>
    where
        'upper: 'data,
        R:      LendFamily<&'upper ()>,
        M:      LendFamily<&'upper ()>,
    {
        let slot = match slot {
            SelfRefCases::NoRef(no_ref)        => SelfRefCases::NoRef(no_ref),
            SelfRefCases::RefMut(self_ref_mut) => SelfRefCases::RefMut(self_ref_mut),
        };

        unsafe { MappedSlot::new(slot) }
    }

    #[inline]
    #[must_use]
    pub const fn data_unused(self) -> TakeDataToken<'brand> {
        unsafe { TakeDataToken::new() }
    }
}

impl Debug for RefMutMapToken<'_, '_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("RefMutMapToken").finish_non_exhaustive()
    }
}

pub struct NonMutMapClonedToken<'brand, 'stable, 'data>(SlotBrand<'brand, 'stable, 'data>);

impl<'brand, 'stable, 'data> NonMutMapClonedToken<'brand, 'stable, 'data> {
    #[inline]
    #[must_use]
    pub(super) const unsafe fn new(brand: SlotBrand<'brand, 'stable, 'data>) -> Self {
        Self(brand)
    }

    #[inline]
    #[must_use]
    pub const fn map<'upper, N, R, M>(
        self,
        slot: SelfRefSlot<'stable, 'upper, N, R, M>,
    ) -> MappedSlot<'brand, 'data, 'upper, N, R, M>
    where
        'upper: 'data,
        R:      LendFamily<&'upper ()>,
        M:      LendFamily<&'upper ()>,
    {
        unsafe { MappedSlot::new(slot) }
    }
}

impl Debug for NonMutMapClonedToken<'_, '_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("NonMutMapClonedToken").finish_non_exhaustive()
    }
}

pub struct DataToken<'brand, 'a, Data> {
    _brand: DataBrand<'brand>,
    /// # Safety Invariant
    /// TODO.
    data:   &'a mut Option<SpeedBump<Data>>,
}

impl<'brand, 'a, Data> DataToken<'brand, 'a, Data> {
    /// # Safety
    /// TODO.
    #[inline]
    #[must_use]
    pub(super) const unsafe fn new(
        brand: DataBrand<'brand>,
        data:  &'a mut Option<SpeedBump<Data>>,
    ) -> Self {
        Self {
            _brand: brand,
            data,
        }
    }

    #[inline]
    #[must_use]
    pub fn attach<'data, 'upper, N, R, M>(
        self,
        slot: MappedSlot<'brand, 'data, 'upper, N, R, M>,
    ) -> AttachableRefFull<'data, 'upper, N, R, M, Data>
    where
        'upper: 'data,
        R:      LendFamily<&'upper ()>,
        M:      LendFamily<&'upper ()>,
    {
        let data = self.data.take();
        let data = unsafe { data.unwrap_unchecked() };

        unsafe { AttachableRefFull::from_mapped_slot(data, slot) }
    }

    #[inline]
    #[must_use]
    pub fn take_data(self, _token: TakeDataToken<'brand>) -> Data {
        let data = self.data.take();
        let data = unsafe { data.unwrap_unchecked() };
        data.speed_bump
    }
}

impl<Data> Debug for DataToken<'_, '_, Data> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("DataToken").finish_non_exhaustive()
    }
}

pub struct CloneDataToken<'brand, 'a, Data> {
    _brand: DataBrand<'brand>,
    /// # Safety Invariant
    /// TODO.
    data:   Option<&'a Data>,
}

impl<'brand, 'a, 'data, Data> CloneDataToken<'brand, 'a, Data> {
    /// # Safety
    /// TODO.
    #[inline]
    #[must_use]
    pub(super) const unsafe fn new(
        brand: DataBrand<'brand>,
        data:  Option<&'a Data>,
    ) -> Self {
        Self {
            _brand: brand,
            data,
        }
    }

    #[inline]
    #[must_use]
    pub fn clone_and_attach<'upper, N, R, M>(
        self,
        slot: MappedSlot<'brand, 'data, 'upper, N, R, M>,
    ) -> AttachableRefFull<'data, 'upper, N, R, M, Data>
    where
        'upper: 'data,
        R:      LendFamily<&'upper ()>,
        M:      LendFamily<&'upper ()>,
        Data:   StableClone<'data>,
    {
        let data = unsafe { self.data.unwrap_unchecked() };

        let data = SpeedBump {
            speed_bump: data.clone(),
        };

        unsafe { AttachableRefFull::from_mapped_slot(data, slot) }
    }
}

impl<Data> Debug for CloneDataToken<'_, '_, Data> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("CloneDataToken").finish_non_exhaustive()
    }
}
