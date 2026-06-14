#![expect(
    unsafe_code,
    reason = "Manage lifetime-branded `!Copy` token-based invariants; manipulate self-refs",
)]

use core::{marker::PhantomData, mem::transmute};
use core::fmt::{Debug, Formatter, Result as FmtResult};

use variance_family::{LendFamily, family, phantom_zst_methods};

use crate::slot::{ErasedSelfRefSlot, SelfRefSlot};
use super::branded_tokens::TakeDataToken;


pub struct MappedSlot<'brand, 'data, 'upper, N, R, M>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    brand:        PhantomData<fn(*mut &'brand ())>,
    variance:     PhantomData<fn(*mut R, *mut M) -> &'data ()>,
    /// # Safety Invariant
    /// TODO.
    checked_slot: SelfRefSlot<'upper, 'upper, N, R, M>,
}

impl<'brand, 'data, 'upper, N, R, M> MappedSlot<'brand, 'data, 'upper, N, R, M>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    /// # Safety
    /// TODO.
    #[inline]
    #[must_use]
    pub(super) const unsafe fn new(checked_slot: SelfRefSlot<'_, 'upper, N, R, M>) -> Self {
        let checked_slot = unsafe {
            transmute::<
                SelfRefSlot<'_, 'upper, N, R, M>,
                SelfRefSlot<'upper, 'upper, N, R, M>,
            >(checked_slot)
        };

        Self {
            brand:        PhantomData,
            variance:     PhantomData,
            checked_slot,
        }
    }

    /// # Safety
    /// TODO.
    ///
    /// # Robust Guarantee
    /// This function does not unwind.
    #[inline]
    #[must_use]
    pub(crate) unsafe fn into_erased_slot(self) -> ErasedSelfRefSlot<'upper, N, R, M> {
        unsafe { ErasedSelfRefSlot::erase(self.checked_slot) }
    }

    #[inline]
    #[must_use]
    pub fn drop(self) -> TakeDataToken<'brand> {
        drop(self);
        unsafe { TakeDataToken::new() }
    }
}

impl<'data, 'upper, N, R, M> Debug for MappedSlot<'_, 'data, 'upper, N, R, M>
where
    'upper: 'data,
    N:      Debug,
    R:      LendFamily<&'upper (), Is: Debug>,
    M:      LendFamily<&'upper (), Is: Debug>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("MappedSlot")
            .field("checked_slot", &self.checked_slot)
            .finish_non_exhaustive()
    }
}

pub struct VaryingMappedSlot<'data, 'upper: 'data, N, R, M>(
    #[expect(clippy::type_complexity, reason = "one-off type for variance")]
    PhantomData<fn(*mut (&'upper (), N, R, M)) -> &'data ()>,
);

phantom_zst_methods!(
    impl<{'data, 'upper, N, R, M}> _ for VaryingMappedSlot<{'data, 'upper, N, R, M}>
    where {
        'upper: 'data,
        R:      LendFamily<&'upper ()>,
        M:      LendFamily<&'upper ()>,
    }
);

family! {
    impl<'brand, {'data, 'upper, N, R, M,}> LifetimeFamily<'_, _>
    // SAFETY: `VaryingMappedSlot` is defined in this crate.
    for #[unsafe(not_a_foreign_fundamental_type)] VaryingMappedSlot<'data, 'upper, N, R, M>
    // The type given here is the `'varying`-parameterized type.
    as MappedSlot<'brand, 'data, 'upper, N, R, M>
    where {
        'upper: 'data,
        R:      LendFamily<&'upper ()>,
        M:      LendFamily<&'upper ()>,
    }
}
