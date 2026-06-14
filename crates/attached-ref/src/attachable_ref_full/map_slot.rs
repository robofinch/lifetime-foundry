//! Temporary organization module.

use core::mem;
use core::{convert::Infallible, marker::PhantomData};

use stable_view::StableViewer;
use variance_family::{Lend, LendFamily};

use crate::pre_1_94_closure_hack::LendWrapper;
use crate::{
    map_slot::{
        BrandFamily, Branded, CloneDataToken, DataToken, MapCases, MapClonedCases,
        map_slot_cloned_impl, map_slot_impl,
    },
    outlives::{Outlives, OutlivesChain},
    slot::{ErasedSelfRefSlot, SelfRefCases, SelfRefSlotWrapper},
};
use super::full_struct::AttachableRefFull;


impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    /// TODO.
    #[inline]
    #[must_use]
    pub fn take_ref<F, T>(&mut self, f: F) -> Option<T>
    where
        N: Default,
        F: for<'stable> FnOnce(
            Lend<'stable, &'upper (), R>,
            Outlives<'data, 'stable>,
        ) -> T,
    {
        // Note that we don't touch `self.data`, so even if `f(..)` unwinds, any references to
        // `data` are necessarily dropped before `data` is dropped.
        {
            let no_ref = SelfRefCases::NoRef(N::default());
            let no_ref = unsafe { ErasedSelfRefSlot::erase(no_ref) };

            let slot = mem::replace(&mut self.slot, no_ref);
            let slot = unsafe { slot.into_unerased() };

            match slot {
                SelfRefCases::Ref(self_ref) => Some(f(self_ref, Outlives::new())),
                SelfRefCases::NoRef(_) | SelfRefCases::RefMut(_) => None,
            }
        }
    }

    /// TODO.
    #[inline]
    #[must_use]
    pub fn map_ref<Ref, F>(self, f: F) -> AttachableRefFull<'data, 'upper, N, Ref, M, Data>
    where
        Ref:    LendFamily<&'upper ()>,
        F:      for<'a, 'stable> FnOnce(
                    LendWrapper<'stable, 'upper, R>,
                    StableViewer<'a, 'stable, 'data, Data>,
                    OutlivesChain<'data, 'stable, 'a>,
                ) -> SelfRefSlotWrapper<'stable, 'upper, N, Ref, M>,
    {
        let Ok(this) = self.try_map_ref(|lend, viewer, outlives| {
            Result::<_, Infallible>::Ok(f(lend, viewer, outlives))
        });
        this
    }

    /// TODO.
    ///
    /// # Errors
    /// TODO.
    #[inline]
    pub fn try_map_ref<NewR, F, E>(
        self,
        f: F,
    ) -> Result<AttachableRefFull<'data, 'upper, N, NewR, M, Data>, E>
    where
        NewR: LendFamily<&'upper ()>,
        F:  for<'a, 'stable> FnOnce(
            LendWrapper<'stable, 'upper, R>,
            StableViewer<'a, 'stable, 'data, Data>,
            OutlivesChain<'data, 'stable, 'a>,
        ) -> Result<SelfRefSlotWrapper<'stable, 'upper, N, NewR, M>, E>,
    {
        // If something `panic`s and triggers an unwind within this block, or if we return early,
        // this extra block ensures that everything inside -- including all the self-references,
        // which are moved out of `self.slot` at the top of the block -- is dropped before anything
        // outside the block, notably including `self.data`.
        // We therefore do not need to leak anything or abort the process due to an unwind here.
        let new_slot = {
            let slot = unsafe { self.slot.into_unerased() };

            match slot {
                SelfRefCases::NoRef(no_ref) => SelfRefCases::NoRef(no_ref),
                SelfRefCases::Ref(self_ref) => {
                    let viewer = unsafe { StableViewer::new(&self.data.speed_bump) };

                    f(LendWrapper::new(self_ref), viewer, OutlivesChain::new())?.into_lend()
                }
                SelfRefCases::RefMut(self_ref_mut) => SelfRefCases::RefMut(self_ref_mut),
            }
        };

        let new_slot = unsafe { ErasedSelfRefSlot::erase(new_slot) };

        Ok(AttachableRefFull {
            slot:     new_slot,
            variance: PhantomData,
            data:     self.data,
        })
    }

    /// TODO.
    #[inline]
    #[must_use]
    pub fn take_mut<F, T>(&mut self, f: F) -> Option<T>
    where
        N: Default,
        F: for<'stable> FnOnce(
            Lend<'stable, &'upper (), M>,
            Outlives<'data, 'stable>,
        ) -> T,
    {
        // Note that we don't touch `self.data`, so even if `f(..)` unwinds, any references to
        // `data` are necessarily dropped before `data` is dropped.
        {
            let no_ref = SelfRefCases::NoRef(N::default());
            let no_ref = unsafe { ErasedSelfRefSlot::erase(no_ref) };

            let slot = mem::replace(&mut self.slot, no_ref);
            let slot = unsafe { slot.into_unerased() };

            match slot {
                SelfRefCases::RefMut(self_ref_mut) => Some(f(self_ref_mut, Outlives::new())),
                SelfRefCases::NoRef(_) | SelfRefCases::Ref(_) => None,
            }
        }
    }

    // try_map_ref (with viewer)

    // try_map_ref_with

    // map_mut (no viewer)

    // try_map_mut (no viewer)

    // insert_unattached

    // insert_ref

    // insert_ref_with

    // insert_mut (with viewer, no old value, N: default)

    // insert_mut_with (with viewer, no old value, N: Default)

    // hmm... but anyone could do that, by taking out of `&mut self` and doing `self.into_data()`.
    // Document that fact.

    // set_mut_abort (with viewer, no old value, aborts if mapping fails)

    // set_mut_abort_with (with viewer, no old value, aborts if mapping fails)

    // Example mapping:
    // pub fn split_cases<R1, M1, N2, M2, N3, R3>(self) -> SelfRefCases<
    //     AttachableRefFull<'data, 'upper, N, R1, M1, Data>,
    //     AttachableRefFull<'data, 'upper, N2, R, M2, Data>,
    //     AttachableRefFull<'data, 'upper, N3, R3, M, Data>,
    // >
    // where
    //     R1: LendFamily<&'upper ()>,
    //     M1: LendFamily<&'upper ()>,
    //     M2: LendFamily<&'upper ()>,
    //     R3: LendFamily<&'upper ()>,
    // {
    //     self.map_slot_with_tokens::<_, _,
    //         SelfRefCases<
    //             crate::map_slot::VaryingMappedSlot<'data, 'upper, N, R1, M1>,
    //             crate::map_slot::VaryingMappedSlot<'data, 'upper, N2, R, M2>,
    //             crate::map_slot::VaryingMappedSlot<'data, 'upper, N3, R3, M>,
    //         >, _>(
    //         |cases, _| {
    //             match cases {
    //                 MapCases::NoRef { no_ref, data, token } => {
    //                     let slot = SelfRefCases::NoRef(no_ref);
    //                     SelfRefCases::NoRef(token.map(data, slot))
    //                 }
    //                 MapCases::Ref { self_ref, token, .. } => {
    //                     let slot = SelfRefCases::Ref(self_ref);
    //                     SelfRefCases::Ref(token.map(slot))
    //                 }
    //                 MapCases::RefMut { self_ref_mut, token } => {
    //                     let slot = SelfRefCases::RefMut(self_ref_mut);
    //                     SelfRefCases::RefMut(token.map(slot))
    //                 }
    //             }
    //         },
    //         |slot, token| match slot {
    //             SelfRefCases::NoRef(slot) => SelfRefCases::NoRef(token.attach(slot)),
    //             SelfRefCases::Ref(slot) => SelfRefCases::Ref(token.attach(slot)),
    //             SelfRefCases::RefMut(slot) => SelfRefCases::RefMut(token.attach(slot)),
    //         }
    //     )
    // }

    // map_slot
    // try_map_slot

    /// Map the self-reference slot.
    ///
    /// This method uses a lifetime-branded token system to achieve high flexibility.
    ///
    /// TODO: more details.
    #[expect(clippy::missing_inline_in_public_items, reason = "complicated method")]
    #[must_use]
    pub fn map_slot_with_tokens<F, G, B, T>(self, f: F, g: G) -> T
    where
        F:  for<'brand, 'a, 'stable> FnOnce(
                MapCases<'brand, 'a, 'stable, 'data, 'upper, N, R, M, Data>,
                OutlivesChain<'data, 'stable, 'a>,
            ) -> Branded<'brand, B>,
        G:  for<'brand, 'a> FnOnce(Branded<'brand, B>, DataToken<'brand, 'a, Data>) -> T,
        B:  BrandFamily,
    {
        map_slot_impl(self, f, g)
    }

    /// Map the self-reference slot, possibly cloning the backing data.
    ///
    /// This method uses a lifetime-branded token system to achieve high flexibility.
    ///
    /// TODO: more details.
    #[expect(clippy::missing_inline_in_public_items, reason = "complicated method")]
    #[must_use]
    pub fn map_slot_cloned_with_tokens<F, G, B, T>(&self, f: F, g: G) -> T
    where
        F:  for<'brand, 'a, 'stable> FnOnce(
                MapClonedCases<'brand, 'a, 'stable, 'data, 'upper, N, R, M, Data>,
                OutlivesChain<'data, 'stable, 'a>,
            ) -> Branded<'brand, B>,
        G:  for<'brand, 'a> FnOnce(Branded<'brand, B>, CloneDataToken<'brand, 'a, Data>) -> T,
        B:  BrandFamily,
    {
        map_slot_cloned_impl(self, f, g)
    }
}
