#![expect(
    unsafe_code,
    reason = "Manage lifetime-branded `!Copy` token-based invariants; manipulate self-refs",
)]
use core::convert::Infallible;
use core::fmt::{Debug, Formatter, Result as FmtResult};

use stable_view::StableViewer;
use variance_family::{Lend, LendFamily};

use crate::{outlives::OutlivesChain, slot::SelfRefCases};
use crate::attachable_ref_full::{AttachableRefFull, SpeedBump};
use super::branded_tokens::{
    BrandFamily, Branded, CloneDataToken, DataToken, new_brand, NonMutMapClonedToken, NoRefMapToken,
    RefMapToken, RefMutMapToken,
};


pub enum MapCases<'brand, 'a, 'stable, 'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    NoRef {
        no_ref:       N,
        data:         Data,
        token:        NoRefMapToken<'brand, 'a, 'stable, 'data, Data>,
    },
    Ref {
        self_ref:     Lend<'stable, &'upper (), R>,
        viewer:       StableViewer<'a, 'stable, 'data, Data>,
        token:        RefMapToken<'brand, 'stable, 'data>,
    },
    RefMut {
        self_ref_mut: Lend<'stable, &'upper (), M>,
        token:        RefMutMapToken<'brand, 'stable, 'data>,
    },
}

impl<'data, 'upper, N, R, M, Data> Debug for MapCases<'_, '_, '_, 'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    N:      Debug,
    R:      LendFamily<&'upper (), Is: Debug>,
    M:      LendFamily<&'upper (), Is: Debug>,
    Data:   Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::NoRef { no_ref, data, token } => {
                f.debug_struct("NoRef")
                    .field("no_ref", no_ref)
                    .field("data",   data)
                    .field("token",  token)
                    .finish()
            }
            Self::Ref { self_ref, viewer, token } => {
                f.debug_struct("Ref")
                    .field("self_ref", self_ref)
                    .field("viewer",   viewer)
                    .field("token",    token)
                    .finish()
            }
            Self::RefMut { self_ref_mut, token } => {
                f.debug_struct("RefMut")
                    .field("self_ref_mut", self_ref_mut)
                    .field("token",        token)
                    .finish()
            }
        }
    }
}

pub(crate) fn map_slot_impl<'data, 'upper, N, R, M, Data, F, G, B, T>(
    full: AttachableRefFull<'data, 'upper, N, R, M, Data>,
    f:    F,
    g:    G,
) -> T
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    F:      for<'brand, 'a, 'stable> FnOnce(
                MapCases<'brand, 'a, 'stable, 'data, 'upper, N, R, M, Data>,
                OutlivesChain<'data, 'stable, 'a>,
            ) -> Branded<'brand, B>,
    G:      for<'brand, 'a> FnOnce(Branded<'brand, B>, DataToken<'brand, 'a, Data>) -> T,
    B:      BrandFamily,
{
    let (data, slot) = unsafe { full.into_raw_pieces() };
    let mut data: Option<SpeedBump<Data>> = Some(SpeedBump {
        speed_bump: data,
    });

    let (data_brand, slot_brand) = unsafe { new_brand() };

    // If something `panic`s and triggers an unwind within this block, or if we return early,
    // this extra block ensures that everything inside -- including all the self-references -- is
    // dropped before anything outside the block, notably including `data`.
    // We therefore do not need to leak anything or abort the process due to an unwind here.
    let intermediate = {
        let moved_slot = slot;
        let map_cases = match moved_slot {
            SelfRefCases::NoRef(no_ref) => {
                let moved_out_data = data.take();
                let moved_out_data = unsafe { moved_out_data.unwrap_unchecked() };
                let moved_out_data = moved_out_data.speed_bump;

                let token = unsafe { NoRefMapToken::new(slot_brand, &mut data) };

                MapCases::NoRef {
                    no_ref,
                    data: moved_out_data,
                    token,
                }
            }
            SelfRefCases::Ref(self_ref) => {
                let data_ref = unsafe { data.as_ref().unwrap_unchecked() };
                let data_ref = &data_ref.speed_bump;

                let viewer = unsafe { StableViewer::new(data_ref) };

                let token = unsafe { RefMapToken::new(slot_brand) };

                MapCases::Ref { self_ref, viewer, token }
            }
            SelfRefCases::RefMut(self_ref_mut) => {
                let token = unsafe { RefMutMapToken::new(slot_brand) };

                MapCases::RefMut { self_ref_mut, token }
            }
        };

        f(map_cases, OutlivesChain::new())
    };

    let data_token = unsafe { DataToken::new(data_brand, &mut data) };

    {
        let moved_intermediate = intermediate;

        g(moved_intermediate, data_token)
    }

    // should also return `Option<Data>`.
}

pub enum MapClonedCases<'brand, 'a, 'stable, 'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    NonMut {
        non_mut:      SelfRefCases<&'a N, &'a Lend<'stable, &'upper (), R>, Infallible>,
        viewer:       StableViewer<'a, 'stable, 'data, Data>,
        token:        NonMutMapClonedToken<'brand, 'stable, 'data>,
    },
    RefMut {
        self_ref_mut: &'a Lend<'stable, &'upper (), M>,
    },
}

impl<'data, 'upper, N, R, M, Data> Debug
for MapClonedCases<'_, '_, '_, 'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    N:      Debug,
    R:      LendFamily<&'upper (), Is: Debug>,
    M:      LendFamily<&'upper (), Is: Debug>,
    Data:   Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::NonMut { non_mut, viewer, token } => {
                f.debug_struct("NonMut")
                    .field("non_mut", non_mut)
                    .field("viewer",  viewer)
                    .field("token",   token)
                    .finish()
            }
            Self::RefMut { self_ref_mut } => {
                f.debug_struct("RefMut")
                    .field("self_ref_mut", self_ref_mut)
                    .finish()
            }
        }
    }
}

pub(crate) fn map_slot_cloned_impl<'data, 'upper, N, R, M, Data, F, G, B, T>(
    full: &AttachableRefFull<'data, 'upper, N, R, M, Data>,
    f:    F,
    g:    G,
) -> T
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    F:      for<'brand, 'a, 'stable> FnOnce(
                MapClonedCases<'brand, 'a, 'stable, 'data, 'upper, N, R, M, Data>,
                OutlivesChain<'data, 'stable, 'a>,
            ) -> Branded<'brand, B>,
    G:      for<'brand, 'a> FnOnce(Branded<'brand, B>, CloneDataToken<'brand, 'a, Data>) -> T,
    B:      BrandFamily,
{
    // Since `full` (and therefore its `data`) cannot be dropped while this function runs,
    // we don't have to be as careful about adding extra scopes to drop self-references before
    // `data`.
    let (data_brand, slot_brand) = unsafe { new_brand() };

    let (cases, data) = match full.get_full() {
        SelfRefCases::NoRef((no_ref, data)) => {
            let non_mut = SelfRefCases::NoRef(no_ref);

            let viewer = unsafe { StableViewer::new(data) };

            let token = unsafe { NonMutMapClonedToken::new(slot_brand) };

            (MapClonedCases::NonMut { non_mut, viewer, token }, Some(data))
        }
        SelfRefCases::Ref((self_ref, data)) => {
            let non_mut = SelfRefCases::Ref(self_ref);

            let viewer = unsafe { StableViewer::new(data) };

            let token = unsafe { NonMutMapClonedToken::new(slot_brand) };

            (MapClonedCases::NonMut { non_mut, viewer, token }, Some(data))
        }
        SelfRefCases::RefMut(self_ref_mut) => (MapClonedCases::RefMut { self_ref_mut }, None),
    };

    let intermediate = f(cases, OutlivesChain::new());

    let data_token = unsafe { CloneDataToken::new(data_brand, data) };

    {
        let moved_intermediate = intermediate;

        g(moved_intermediate, data_token)
    }
}
