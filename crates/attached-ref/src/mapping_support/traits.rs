use core::{convert::Infallible, marker::PhantomData};

use stable_view::Viewer;
use variance_family::{Lend, LendFamily};

use crate::{SelfRefSlot, slot::SelfRefCases};
use super::shorthand_macros::{FullResult, RefResult, RefMutResult};


pub trait MapSlot<'data, 'upper, 'new_upper, N, R, M, Data>
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

pub trait MapBorrowedNonMut<'data, 'upper, 'new_upper, N, R, M, Data>
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

    fn map_non_mut<'a, 'stable>(
        self,
        non_mut: SelfRefCases<&'a N, &'a Lend<'stable, &'upper (), R>, Infallible>,
        data:    Viewer<'a, 'stable, 'upper, Data>,
    ) -> RefResult!('stable, 'new_upper, Self)
    where
        'data:   'stable,
        'stable: 'a;
}

pub trait MapNonMut<'data, 'upper, 'new_upper, N, R, NewN, NewR, NewM, Data>
where
    'upper:     'data,
    'new_upper: 'data,
    R:          LendFamily<&'upper ()>,
    NewR:       LendFamily<&'new_upper ()>,
    NewM:       LendFamily<&'new_upper ()>,
{
    #[must_use]
    fn map_non_mut<'a, 'stable>(
        self,
        non_mut: SelfRefSlot<'stable, 'upper, N, R, Infallible>,
        data:    Viewer<'a, 'stable, 'upper, Data>,
    ) -> SelfRefSlot<'stable, 'new_upper, NewN, NewR, NewM>
    where
        'data:   'stable,
        'stable: 'a;
}

impl<'data, 'upper, 'new_upper, N, R, NewN, NewR, NewM, Data, F>
    MapNonMut<'data, 'upper, 'new_upper, N, R, NewN, NewR, NewM, Data>
for F
where
    'upper:     'data,
    'new_upper: 'data,
    R:          LendFamily<&'upper ()>,
    NewR:       LendFamily<&'new_upper ()>,
    NewM:       LendFamily<&'new_upper ()>,
    F:          for<'a, 'stable> FnOnce(
                    SelfRefSlot<'stable, 'upper, N, R, Infallible>,
                    Viewer<'a, 'stable, 'upper, Data>,
                    PhantomData<&'a &'stable &'data ()>,
                ) -> SelfRefSlot<'stable, 'new_upper, NewN, NewR, NewM>,
{
    #[inline]
    fn map_non_mut<'a, 'stable>(
        self,
        non_mut: SelfRefSlot<'stable, 'upper, N, R, Infallible>,
        data:    Viewer<'a, 'stable, 'upper, Data>,
    ) -> SelfRefSlot<'stable, 'new_upper, NewN, NewR, NewM>
    where
        'data:   'stable,
        'stable: 'a
    {
        self(non_mut, data, PhantomData)
    }
}
