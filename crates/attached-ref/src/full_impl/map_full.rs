//! The final boss.
//!
//! Implements methods which map parts of [`AttachableRefFull`].

#![expect(unsafe_code, reason = "perform unsafe lifetime erasure and extension of self-refs")]

#![expect(missing_docs, clippy::undocumented_unsafe_blocks, clippy::missing_errors_doc, reason = "TODO")]


use core::{convert::Infallible, marker::PhantomData};

use stable_view::{StableClone, Viewer};
use variance_family::{Lend, LendFamily};

use crate::{erased_slot::ErasedSelfRefSlot, slot::SelfRefCases};
use super::{
    attachable_ref_full::{AttachableRefFull, SpeedBump},
    shorthand_macros::{FullResult, RefResult, RefMutResult},
};


pub trait MapFull<'data, 'upper, 'new_upper, N, R, M, Data>
where
    'upper:     'data,
    'new_upper: 'data,
    R:          LendFamily<&'upper ()>,
    M:          LendFamily<&'upper ()>,
{
    type NewN;
    type NewR: LendFamily<&'new_upper ()>;
    type NewM: LendFamily<&'new_upper ()>;

    type Ok;
    type Err;

    fn case_no_ref(
        self,
        no_ref: N,
        data:   Data,
    ) -> FullResult!('data, 'new_upper, Data, Self);

    fn case_ref<'a, 'stable>(
        self,
        self_ref: Lend<'stable, &'upper (), R>,
        data:     Viewer<'a, 'stable, 'upper, Data>,
    ) -> RefResult!('stable, 'new_upper, Self)
    where
        'data:   'stable,
        'stable: 'a;

    fn case_ref_mut<'a, 'stable>(
        self,
        self_ref_mut: Lend<'stable, &'upper (), M>,
    ) -> RefMutResult!('stable, 'new_upper, Self)
    where
        'data:   'stable,
        'stable: 'a;
}

pub trait MapFullClone<'data, 'upper, 'new_upper, N, R, M, Data>
where
    'upper:     'data,
    'new_upper: 'data,
    R:          LendFamily<&'upper ()>,
    M:          LendFamily<&'upper ()>,
{
    type NewN;
    type NewR: LendFamily<&'new_upper ()>;
    type NewM: LendFamily<&'new_upper ()>;

    type Ok;
    type Err;

    fn map_maybe_ref<'a, 'stable>(
        self,
        maybe_ref: SelfRefCases<&'a N, &'a Lend<'a, &'upper (), R>, Infallible>,
        data:      Viewer<'a, 'stable, 'upper, Data>,
    ) -> RefResult!('stable, 'new_upper, Self)
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
    #[must_use]
    pub fn map_full<'new_upper, Mapper>(
        self,
        mapper: Mapper,
    ) -> AttachableRefFull<'data, 'new_upper, Mapper::NewN, Mapper::NewR, Mapper::NewM, Data>
    where
        Mapper: MapFull<'data, 'upper, 'new_upper, N, R, M, Data, Ok = (), Err = Infallible>,
    {
        let Ok((this, ())) = self.try_map_full(mapper);
        this
    }

    pub fn try_map_full<'new_upper, Mapper>(
        self,
        mapper: Mapper,
    ) -> FullResult!('data, 'new_upper, Data, Mapper)
    where
        Mapper: MapFull<'data, 'upper, 'new_upper, N, R, M, Data>,
    {
        // If something `panic`s and triggers an unwind within this block, this extra block
        // ensures that everything inside -- including all the self-references, which are
        // moved out of `self.slot` at the top of the block -- is dropped before anything outside
        // the block, notably including `self.data`.
        // We therefore do not need to leak anything or abort the process due to an unwind here.
        let (new_slot, return_ok) = {

            let unerased = unsafe { self.slot.into_unerased() };

            let (new_slot, return_ok) = match unerased {
                SelfRefCases::NoRef(no_ref) => {
                    return mapper.case_no_ref(no_ref, self.data.speed_bump_inner);
                }
                SelfRefCases::Ref(self_ref) => {
                    let data = unsafe { Viewer::new(&self.data.speed_bump_inner) };
                    let (case_ref, return_ok) = mapper.case_ref(self_ref, data)?;

                    match case_ref {
                        SelfRefCases::NoRef(new_no_ref) => (
                            SelfRefCases::NoRef(new_no_ref),
                            return_ok,
                        ),
                        SelfRefCases::Ref(new_self_ref) => (
                            SelfRefCases::Ref(new_self_ref),
                            return_ok,
                        ),
                    }
                }
                SelfRefCases::RefMut(self_ref_mut) => {
                    let (case_ref_mut, return_ok) = mapper.case_ref_mut(self_ref_mut)?;

                    match case_ref_mut {
                        SelfRefCases::NoRef(new_no_ref) => (
                            SelfRefCases::NoRef(new_no_ref),
                            return_ok,
                        ),
                        SelfRefCases::RefMut(new_self_ref_mut) => (
                            SelfRefCases::RefMut(new_self_ref_mut),
                            return_ok,
                        ),
                    }
                }
            };

            let new_slot = unsafe { ErasedSelfRefSlot::erase(new_slot) };

            (new_slot, return_ok)
        };

        let new_attachable_ref = AttachableRefFull {
            slot:     new_slot,
            // NOTE: `'data` does not change, preserving the covariance over `'data`.
            // `R` and `M` can completely change, though, which is why we need to create this
            // new marker.
            variance: PhantomData,
            data:     self.data,
        };

        Ok((new_attachable_ref, return_ok))
    }
}

impl<'data, 'upper, N, R, Data> AttachableRefFull<'data, 'upper, N, R, Infallible, Data>
where
    R:      LendFamily<&'upper ()>,
    'upper: 'data,
{
    #[must_use]
    pub fn map_full_and_clone<'new_upper, Mapper>(
        &self,
        mapper: Mapper,
    ) -> AttachableRefFull<'data, 'new_upper, Mapper::NewN, Mapper::NewR, Mapper::NewM, Data>
    where
        Data:   StableClone<'data>,
        Mapper: MapFullClone<
            'data, 'upper, 'new_upper,
            N, R, Infallible, Data,
            Ok = (), Err = Infallible,
        >,
    {
        match self.try_map_full_and_clone(mapper) {
            Ok(Ok((this, ()))) => this,
            Err(&infallible) => match infallible {},
        }
    }
}

impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    'upper: 'data,
{
    #[expect(clippy::type_complexity, reason = "yeah. However, this is a `_full` method")]
    pub fn try_map_full_and_clone<'a, 'new_upper, Mapper>(
        &'a self,
        mapper: Mapper,
    ) -> Result<FullResult!('data, 'new_upper, Data, Mapper), &'a Lend<'a, &'upper (), M>>
    where
        Data:   StableClone<'data>,
        Mapper: MapFullClone<'data, 'upper, 'new_upper, N, R, M, Data>,
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
        let (new_slot, return_ok) = match mapper.map_maybe_ref(maybe_ref, viewer) {
            Ok((SelfRefCases::NoRef(new_no_ref), return_ok)) => (
                SelfRefCases::NoRef(new_no_ref),
                return_ok,
            ),
            Ok((SelfRefCases::Ref(new_self_ref), return_ok)) => (
                SelfRefCases::Ref(new_self_ref),
                return_ok,
            ),
            Err(return_err) => return Ok(Err(return_err)),
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

        Ok(Ok((new_attachable_ref, return_ok)))
    }
}

// pub trait DynDrop: 'static {}

// impl<T: ?Sized + 'static> DynDrop for T {}

// #[cfg(feature = "alloc")]
// pub type ErasedBox = alloc::boxed::Box<dyn DynDrop>;

// #[cfg(feature = "alloc")]
// pub type ErasedRc = alloc::rc::Rc<dyn DynDrop>;

// #[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
// pub type ErasedArc = alloc::sync::Arc<dyn DynDrop + Send + Sync>;
