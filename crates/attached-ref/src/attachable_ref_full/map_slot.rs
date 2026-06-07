#![expect(unsafe_code, reason = "perform unsafe lifetime erasure and extension of self-refs")]

use core::mem;
use core::{convert::Infallible, marker::PhantomData};

use stable_view::{StableClone, Viewer};
use variance_family::{Lend, LendFamily};

// use crate::SelfRefSlot;
use crate::mapping_support::FullResult;
use crate::{
    mapping_support::{MapBorrowedNonMut, MapSlot},
    slot::{ErasedSelfRefSlot, SelfRefCases},
};
use super::full_struct::{AttachableRefFull, SpeedBump};


impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    #[inline]
    #[must_use]
    pub fn take_ref<F, T>(&mut self, f: F) -> Option<T>
    where
        N: Default,
        F: for<'stable> FnOnce(
            Lend<'stable, &'upper (), R>,
            PhantomData<&'stable &'data ()>,
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
                SelfRefCases::Ref(self_ref) => Some(f(self_ref, PhantomData)),
                SelfRefCases::NoRef(_) | SelfRefCases::RefMut(_) => None,
            }
        }
    }

    // I just thought of a fix for implied bounds, question mark question mark???
    // #[inline]
    // #[must_use]
    // pub fn map_ref<NewR, F>(self, f: F) -> AttachableRefFull<'data, 'upper, N, NewR, M, Data>
    // where
    //     NewR:   LendFamily<&'upper ()>,
    //     F:      for<'a, 'stable> FnOnce(
    //                 Lend<'stable, &'upper (), R>,
    //                 Viewer<'a, 'stable, 'upper, Data>,
    //             ) -> SelfRefSlot<'stable, 'upper, N, NewR, M>,
    // {
    //     todo!()
    // }

    // #[inline]
    // pub fn try_map_ref<NewR, F, E>(
    //     self,
    //     f: F,
    // ) -> Result<AttachableRefFull<'data, 'upper, N, NewR, M, Data>, E>
    // where
    //     NewR:   LendFamily<&'upper ()>,
    //     F:      for<'a, 'stable> FnOnce(
    //                 Imply<'a, 'stable, 'data, 'upper>,
    //                 Lend<'stable, &'upper (), R>,
    //                 Viewer<'a, 'stable, 'upper, Data>,
    //             ) -> Result<SelfRefSlot<'stable, 'upper, N, NewR, M>, E>,
    // {
    //     // If something `panic`s and triggers an unwind within this block, or if we return early,
    //     // this extra block ensures that everything inside -- including all the self-references,
    //     // which are moved out of `self.slot` at the top of the block -- is dropped before anything
    //     // outside the block, notably including `self.data`.
    //     // We therefore do not need to leak anything or abort the process due to an unwind here.
    //     let new_slot = {
    //         let slot = unsafe { self.slot.into_unerased() };

    //         match slot {
    //             SelfRefCases::NoRef(no_ref) => SelfRefCases::NoRef(no_ref),
    //             SelfRefCases::Ref(self_ref) => {
    //                 let viewer = unsafe { Viewer::new(&self.data.speed_bump_inner) };

    //                 f(Imply(PhantomData), self_ref, viewer)?
    //             }
    //             SelfRefCases::RefMut(self_ref_mut) => SelfRefCases::RefMut(self_ref_mut),
    //         }
    //     };

    //     let new_slot = unsafe { ErasedSelfRefSlot::erase(new_slot) };

    //     Ok(AttachableRefFull {
    //         slot:     new_slot,
    //         variance: PhantomData,
    //         data:     self.data,
    //     })
    // }

    #[inline]
    #[must_use]
    pub fn take_mut<F, T>(&mut self, f: F) -> Option<T>
    where
        N: Default,
        F: for<'stable> FnOnce(
            Lend<'stable, &'upper (), M>,
            PhantomData<&'stable &'data ()>,
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
                SelfRefCases::RefMut(self_ref_mut) => Some(f(self_ref_mut, PhantomData)),
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

    #[expect(clippy::type_complexity, reason = "unavoidable. At least has good vertical align.")]
    #[inline]
    #[must_use]
    pub fn split_cases<R1, M1, N2, M2, N3, R3>(self) -> SelfRefCases<
        AttachableRefFull<'data, 'upper, N, R1, M1, Data>,
        AttachableRefFull<'data, 'upper, N2, R, M2, Data>,
        AttachableRefFull<'data, 'upper, N3, R3, M, Data>,
    >
    where
        R1: LendFamily<&'upper ()>,
        M1: LendFamily<&'upper ()>,
        M2: LendFamily<&'upper ()>,
        R3: LendFamily<&'upper ()>,
    {
        let data = self.data;
        let mixed_slot = unsafe { self.slot.into_unerased() };

        match mixed_slot {
            SelfRefCases::NoRef(no_ref) => {
                let slot = SelfRefCases::NoRef(no_ref);
                let this = unsafe { AttachableRefFull::from_slot(data, slot) };
                SelfRefCases::NoRef(this)
            }
            SelfRefCases::Ref(self_ref) => {
                let slot = SelfRefCases::Ref(self_ref);
                let this = unsafe { AttachableRefFull::from_slot(data, slot) };
                SelfRefCases::Ref(this)
            }
            SelfRefCases::RefMut(self_ref_mut) => {
                let slot = SelfRefCases::RefMut(self_ref_mut);
                let this = unsafe { AttachableRefFull::from_slot(data, slot) };
                SelfRefCases::RefMut(this)
            }
        }
    }

    #[expect(clippy::missing_inline_in_public_items, reason = "complicated method")]
    #[must_use]
    pub fn map_slot<'new_upper, Map>(
        self,
        map: Map,
    ) -> AttachableRefFull<'data, 'new_upper, Map::NewN, Map::NewR, Map::NewM, Data>
    where
        Map: MapSlot<'data, 'upper, 'new_upper, N, R, M, Data, Ok = (), Err = Infallible>,
    {
        let Ok((this, ())) = self.try_map_slot(map);
        this
    }

    #[expect(clippy::missing_inline_in_public_items, reason = "complicated method")]
    pub fn try_map_slot<'new_upper, Map>(
        self,
        map: Map,
    ) -> FullResult!('data, 'new_upper, Data, Map)
    where
        Map: MapSlot<'data, 'upper, 'new_upper, N, R, M, Data>,
    {
        // If something `panic`s and triggers an unwind within this block, this extra block
        // ensures that everything inside -- including all the self-references, which are
        // moved out of `self.slot` at the top of the block -- is dropped before anything outside
        // the block, notably including `self.data`.
        // We therefore do not need to leak anything or abort the process due to an unwind here.
        let (new_slot, ok) = {

            let unerased = unsafe { self.slot.into_unerased() };

            let (new_slot, ok) = match unerased {
                SelfRefCases::NoRef(no_ref) => {
                    return map.case_no_ref(no_ref, self.data.speed_bump_inner);
                }
                SelfRefCases::Ref(self_ref) => {
                    let data = unsafe { Viewer::new(&self.data.speed_bump_inner) };
                    map.case_ref(self_ref, data)?
                }
                SelfRefCases::RefMut(self_ref_mut) => {
                    let (case_ref_mut, ok) = map.case_ref_mut(self_ref_mut)?;

                    match case_ref_mut {
                        SelfRefCases::NoRef(new_no_ref) => (
                            SelfRefCases::NoRef(new_no_ref),
                            ok,
                        ),
                        SelfRefCases::RefMut(new_self_ref_mut) => (
                            SelfRefCases::RefMut(new_self_ref_mut),
                            ok,
                        ),
                    }
                }
            };

            let new_slot = unsafe { ErasedSelfRefSlot::erase(new_slot) };

            (new_slot, ok)
        };

        let new_attachable_ref = AttachableRefFull {
            slot:     new_slot,
            // NOTE: `'data` does not change, preserving the covariance over `'data`.
            // `R` and `M` can completely change, though, which is why we need to create this
            // new marker.
            variance: PhantomData,
            data:     self.data,
        };

        Ok((new_attachable_ref, ok))
    }

    #[expect(clippy::type_complexity, reason = "for better similarity with the non-`try` version")]
    #[expect(clippy::missing_inline_in_public_items, reason = "complicated method")]
    pub fn try_map_non_mut_cloned<'a, 'new_upper, Map>(
        &'a self,
        map: Map,
    ) -> Result<FullResult!('data, 'new_upper, Data, Map), &'a Lend<'a, &'upper (), M>>
    where
        Data: StableClone<'data>,
        Map:  MapBorrowedNonMut<'data, 'upper, 'new_upper, N, R, M, Data>,
    {
        let (non_mut, data) = match self.get_full() {
            SelfRefCases::NoRef((no_ref, data)) => (SelfRefCases::NoRef(no_ref), data),
            SelfRefCases::Ref((self_ref, data)) => (SelfRefCases::Ref(self_ref), data),
            SelfRefCases::RefMut(self_ref_mut)  => return Err(self_ref_mut),
        };

        let viewer = unsafe { Viewer::new(data) };

        // Because we're viewing the `Data` within `self`, which is immutably borrowed for the whole
        // body of this function and thus cannot be invalidated, we don't need as much caution
        // around unwinds as in `map_full`.
        let (new_slot, ok) = match map.map_non_mut(non_mut, viewer) {
            Ok(slot_and_aux) => slot_and_aux,
            Err(err)         => return Ok(Err(err)),
        };

        let new_slot = unsafe { ErasedSelfRefSlot::erase(new_slot) };

        let new_data = SpeedBump {
            speed_bump_inner: self.data.speed_bump_inner.clone(),
        };

        let new_attachable_ref = AttachableRefFull {
            slot:     new_slot,
            // NOTE: `'data` does not change, preserving the covariance over `'data`.
            // `R` and `M` can completely change, though, which is why we need to create this
            // new marker.
            variance: PhantomData,
            data:     new_data,
        };

        Ok(Ok((new_attachable_ref, ok)))
    }
}
