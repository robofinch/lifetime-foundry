//! The final boss.
//!
//! Implements *two* methods for [`AttachableRefFull`].

#![expect(unsafe_code, reason = "perform unsafe lifetime erasure and extension of self-refs")]

#![expect(missing_docs, clippy::undocumented_unsafe_blocks, clippy::missing_errors_doc, reason = "TODO")]


use core::{convert::Infallible, marker::PhantomData, mem::MaybeUninit};

use stable_view::{StableClone, Viewer};
use variance_family::{Lend, LendFamily};

use crate::{erased_slot::ErasedSelfRefSlot, slot::SelfRefCases};
use crate::map_data_impl::{MapDataStrict, MapDataStrictest};
use super::{
    attachable_ref_full::{AttachableRefFull, SpeedBump},
    map_full_utils::{FullResult, MappedRef, MappedRefMut, RefResult, RefMutResult},
};


pub trait MapFull<'data, 'upper, 'new_data, 'new_upper, N, R, M, Data>
where
    'new_upper: 'new_data,
    'data:      'new_data,
    R:          LendFamily<&'upper ()>,
    M:          LendFamily<&'upper ()>,
{
    type NewN;
    type NewR: LendFamily<&'new_upper ()>;
    type NewM: LendFamily<&'new_upper ()>;
    type NewData;

    type Ok;
    type Err;

    type Map:          FnOnce(Data) -> Self::NewData;
    type MapStrict:    MapDataStrict<'new_data, Data, Self::NewData>;
    type MapStrictest: MapDataStrictest<'new_data, Data, Self::NewData>;

    fn case_no_ref(
        self,
        no_ref: N,
        data:   Data,
    ) -> FullResult!(Self);

    fn case_ref<'a, 'stable>(
        self,
        self_ref: Lend<'stable, &'upper (), R>,
        data:     Viewer<'a, 'stable, 'upper, Data>,
    ) -> RefResult!(Self)
    where
        'data:   'stable,
        'stable: 'a;

    fn case_ref_mut<'a, 'stable>(
        self,
        self_ref_mut: Lend<'stable, &'upper (), M>,
    ) -> RefMutResult!(Self)
    where
        'data:   'stable,
        'stable: 'a;
}

pub trait MapFullAndClone<'data, 'upper, 'new_data, 'new_upper, N, R, M, Data>
where
    'new_upper: 'new_data,
    'data:      'new_data,
    R:          LendFamily<&'upper ()>,
    M:          LendFamily<&'upper ()>,
{
    type NewN;
    type NewR: LendFamily<&'new_upper ()>;
    type NewM: LendFamily<&'new_upper ()>;
    type NewData;

    type Ok;
    type Err;

    type Map:       FnOnce(Data) -> Self::NewData;
    type MapStrict: MapDataStrict<'new_data, Data, Self::NewData>;

    fn map_maybe_ref<'a, 'stable>(
        self,
        maybe_ref: SelfRefCases<&'a N, &'a Lend<'a, &'upper (), R>, Infallible>,
        data:      Viewer<'a, 'stable, 'upper, Data>,
    ) -> RefResult!(Self)
    where
        'data:   'stable,
        'stable: 'a;
}

impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    'upper: 'data,
{
    /// If the mapper's `MapStrict` or `MapStrictest` implementations panic and unwind, then
    /// any self-references to those fields are leaked.
    pub fn map_full<'new_data, 'new_upper, Mapper>(
        self,
        mapper: Mapper,
    ) -> FullResult!(Mapper)
    where
        'new_upper: 'new_data,
        'data:      'new_data,
        Mapper:     MapFull<'data, 'upper, 'new_data, 'new_upper, N, R, M, Data>,
    {
        enum MapData<M, MS, MST> {
            Map(M),
            MapStrict(MS),
            MapStrictest(MST),
        }

        // If something `panic`s and triggers an unwind within this block, this extra block
        // ensures that everything inside -- including all the self-references, which are
        // moved out of `self.slot` at the top of the block -- is dropped before anything outside
        // the block, notably including `self.data`.
        // We therefore do not need to leak anything or abort the process due to an unwind here.
        let (mut new_slot, map_data, return_ok) = {

            let unerased = unsafe { self.slot.into_unerased() };

            let (new_slot, map_data, return_ok) = match unerased {
                SelfRefCases::NoRef(no_ref) => {
                    return mapper.case_no_ref(no_ref, self.data.speed_bump_inner);
                }
                SelfRefCases::Ref(self_ref) => {
                    let data = unsafe { Viewer::new(&self.data.speed_bump_inner) };
                    let (case_ref, return_ok) = mapper.case_ref(self_ref, data)?;

                    match case_ref {
                        MappedRef::NoRef(new_no_ref, map) => (
                            SelfRefCases::NoRef(new_no_ref),
                            MapData::Map(map),
                            return_ok,
                        ),
                        MappedRef::Ref(new_self_ref, strict) => (
                            SelfRefCases::Ref(new_self_ref),
                            MapData::MapStrict(strict),
                            return_ok,
                        ),
                    }
                }
                SelfRefCases::RefMut(self_ref_mut) => {
                    let (case_ref_mut, return_ok) = mapper.case_ref_mut(self_ref_mut)?;

                    match case_ref_mut {
                        MappedRefMut::NoRef(new_no_ref, map) => (
                            SelfRefCases::NoRef(new_no_ref),
                            MapData::Map(map),
                            return_ok,
                        ),
                        MappedRefMut::RefMut(new_self_ref_mut, strictest) => (
                            SelfRefCases::RefMut(new_self_ref_mut),
                            MapData::MapStrictest(strictest),
                            return_ok,
                        ),
                    }
                }
            };

            let new_slot = unsafe { ErasedSelfRefSlot::erase(new_slot) };

            (new_slot, map_data, return_ok)
        };

        let new_data;

        match map_data {
            MapData::Map(map)                => {
                // In this branch, `new_slot` is necessarily in the `NoRef` state, so no
                // self-references are invalidated if `self.data` is dropped and invalidated
                // during an unwind.
                new_data = map(self.data.speed_bump_inner);
            }
            MapData::MapStrict(strict)       => {
                // In these other two cases, there *are* self-references. Note that we can't
                // detect an unwind and abort the process, since if `self.data` is dropped during
                // an unwind and self-references are invalidated... then UB has already occurred,
                // it'd be too late. Therefore, we wrap them in `MaybeUninit` in order to both
                // disable the destructors of the self-references *and* disable any
                // `dereferenceable` and `noalias` guarantees that the self-references might have.
                // If an unwind does occur... no point in aborting the process when we've already
                // taken a countermeasure, so we can just leak the data (and document as much).
                let maybe_leak = MaybeUninit::new(new_slot);
                // In this branch, there may be immutable/shared self-references, but there are
                // no mutable/exclusive self-references, so the strictest level of mapping is not
                // necessary.
                new_data = strict.map_data_strict(self.data.speed_bump_inner);
                new_slot = unsafe { maybe_leak.assume_init() };
            }
            MapData::MapStrictest(strictest) => {
                // Same as above branch.
                let maybe_leak = MaybeUninit::new(new_slot);
                // There may be immutable/shared or mutable/exclusive self-references, so the
                // strictest level of mapping *is* necessary.
                new_data = strictest.map_data_strictest(self.data.speed_bump_inner);
                new_slot = unsafe { maybe_leak.assume_init() };
            }
        }

        let new_data = SpeedBump {
            speed_bump_inner: new_data,
        };

        let new_attachable_ref = AttachableRefFull {
            slot:     new_slot,
            // NOTE: `'data` *shrinks* to `'new_data`, preserving the covariance over `'data`.
            // `R` and `M` can completely change, though, which is why we need to create this
            // new marker.
            variance: PhantomData,
            data:     new_data,
        };

        Ok((new_attachable_ref, return_ok))
    }

    #[expect(clippy::type_complexity, reason = "yeah. However, this is a `_full` method")]
    pub fn try_map_full_and_clone<'a, 'new_data, 'new_upper, Mapper>(
        &'a self,
        mapper: Mapper,
    ) -> Result<FullResult!(Mapper), &'a Lend<'a, &'upper (), M>>
    where
        'new_upper: 'new_data,
        'data:      'new_data,
        Data:       StableClone<'data>,
        Mapper:     MapFullAndClone<'data, 'upper, 'new_data, 'new_upper, N, R, M, Data>,
    {
        let (maybe_ref, data) = match self.get_full() {
            SelfRefCases::NoRef((no_ref, data)) => (SelfRefCases::NoRef(no_ref), data),
            SelfRefCases::Ref((self_ref, data)) => (SelfRefCases::Ref(self_ref), data),
            SelfRefCases::RefMut(self_ref_mut)  => return Err(self_ref_mut),
        };

        let viewer = unsafe { Viewer::new(data) };

        // Because we're viewing the `Data` within `self`, which is immutably borrowed for the whole
        // body of this function and thus cannot be invalidated, we don't need as much caution
        // around unwinds as in `map_full`.
        let (new_slot, new_data, return_ok) = match mapper.map_maybe_ref(maybe_ref, viewer) {
            Ok((case_ref, return_ok)) => {
                match case_ref {
                    MappedRef::NoRef(new_no_ref, map) => (
                        SelfRefCases::NoRef(new_no_ref),
                        map(data.clone()),
                        return_ok,
                    ),
                    MappedRef::Ref(new_self_ref, strict) => (
                        SelfRefCases::Ref(new_self_ref),
                        strict.map_data_strict(data.clone()),
                        return_ok,
                    ),
                }
            }
            Err(return_err) => return Ok(Err(return_err)),
        };

        let new_slot = unsafe { ErasedSelfRefSlot::erase(new_slot) };

        let new_data = SpeedBump {
            speed_bump_inner: new_data,
        };

        let new_attachable_ref = AttachableRefFull {
            slot:     new_slot,
            // NOTE: `'data` *shrinks* to `'new_data`, preserving the covariance over `'data`.
            // `R` and `M` can completely change, though, which is why we need to create this
            // new marker.
            variance: PhantomData,
            data:     new_data,
        };

        Ok(Ok((new_attachable_ref, return_ok)))
    }
}
