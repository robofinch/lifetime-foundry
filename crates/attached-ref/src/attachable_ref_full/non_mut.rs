#![expect(unsafe_code, reason = "perform unsafe lifetime erasure and extension of self-refs")]

use core::{convert::Infallible, marker::PhantomData};

use stable_view::{StableClone, Viewer};
use variance_family::LendFamily;

use crate::{
    mapping_support::{MapBorrowedNonMut, MapNonMut},
    slot::{ErasedSelfRefSlot, SelfRefCases},
};
use super::full_struct::AttachableRefFull;


impl<'data, 'upper, N, R, Data> AttachableRefFull<'data, 'upper, N, R, Infallible, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    Data:   ?Sized,
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

    #[expect(clippy::missing_inline_in_public_items, reason = "complicated method")]
    #[must_use]
    pub fn map_non_mut<'new_upper, NewN, NewR, NewM, Map>(
        self,
        map: Map,
    ) -> AttachableRefFull<'data, 'new_upper, NewN, NewR, NewM, Data>
    where
        Data: Sized,
        NewR: LendFamily<&'new_upper ()>,
        NewM: LendFamily<&'new_upper ()>,
        Map:  MapNonMut<'data, 'upper, 'new_upper, N, R, NewN, NewR, NewM, Data>,
    {
        // If something `panic`s and triggers an unwind within this block, this extra block
        // ensures that everything inside -- including all the self-references, which are
        // moved out of `self.slot` at the top of the block -- is dropped before anything outside
        // the block, notably including `self.data`.
        // We therefore do not need to leak anything or abort the process due to an unwind here.
        let new_slot = {
            let non_mut = unsafe { self.slot.into_unerased() };
            let data = &self.data.speed_bump_inner;

            let viewer = unsafe { Viewer::new(data) };

            let new_slot = map.map_non_mut(non_mut, viewer);

            unsafe { ErasedSelfRefSlot::erase(new_slot) }
        };

        AttachableRefFull {
            slot:     new_slot,
            // NOTE: `'data` does not change, preserving the covariance over `'data`.
            // `R` and `M` can completely change, though, which is why we need to create this
            // new marker.
            variance: PhantomData,
            data:     self.data,
        }
    }

    #[expect(clippy::missing_inline_in_public_items, reason = "complicated method")]
    #[must_use]
    pub fn map_non_mut_cloned<'new_upper, Map>(
        &self,
        map: Map,
    ) -> AttachableRefFull<'data, 'new_upper, Map::NewN, Map::NewR, Map::NewM, Data>
    where
        Data: StableClone<'data>,
        Map:  MapBorrowedNonMut<
            'data, 'upper, 'new_upper,
            N, R, Infallible, Data,
            Ok = (), Err = Infallible,
        >,
    {
        match self.try_map_non_mut_cloned(map) {
            Ok(Ok((this, ()))) => this,
            Err(&infallible) => match infallible {},
        }
    }
}
