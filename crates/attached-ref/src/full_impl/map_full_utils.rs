use core::fmt::{Debug, Formatter, Result as FmtResult};

use variance_family::{Lend, LendFamily};


pub enum MappedRef<'stable, 'new_upper, NewN, NewR, Map, MapStrict>
where
    NewR: LendFamily<&'new_upper ()>,
{
    NoRef(NewN, Map),
    Ref(Lend<'stable, &'new_upper (), NewR>, MapStrict),
}

impl<'u, N, R, Map, MapS> Debug for MappedRef<'_, 'u, N, R, Map, MapS>
where
    N:    Debug,
    R:    LendFamily<&'u (), Is: Debug>,
    Map:  Debug,
    MapS: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::NoRef(no_ref, map) => {
                f.debug_tuple("NoRef")
                    .field(no_ref)
                    .field(map)
                    .finish()
            }
            Self::Ref(self_ref, strict) => {
                f.debug_tuple("Ref")
                    .field(self_ref)
                    .field(strict)
                    .finish()
            }
        }
    }
}

pub enum MappedRefMut<'stable, 'new_upper, NewN, NewM, Map, MapStrictest>
where
    NewM: LendFamily<&'new_upper ()>,
{
    NoRef(NewN, Map),
    RefMut(Lend<'stable, &'new_upper (), NewM>, MapStrictest),
}

impl<'u, N, M, Map, MapS> Debug for MappedRefMut<'_, 'u, N, M, Map, MapS>
where
    N:    Debug,
    M:    LendFamily<&'u (), Is: Debug>,
    Map:  Debug,
    MapS: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::NoRef(no_ref, map) => {
                f.debug_tuple("NoRef")
                    .field(no_ref)
                    .field(map)
                    .finish()
            }
            Self::RefMut(self_ref_mut, strictest) => {
                f.debug_tuple("RefMut")
                    .field(self_ref_mut)
                    .field(strictest)
                    .finish()
            }
        }
    }
}


/// An alias for `Result<(AttachableRefFull<..>, _), _>`. Brevity is the sole purpose of this macro.
#[doc(hidden)]
#[macro_export]
macro_rules! __FullResult {
    ($Mapper:ident) => {
        ::core::result::Result<
            (
                $crate::AttachableRefFull<
                    'new_data, 'new_upper,
                    $Mapper::NewN, $Mapper::NewR, $Mapper::NewM, $Mapper::NewData,
                >,
                $Mapper::Ok,
            ),
            $Mapper::Err,
        >
    };
}

#[doc(inline)]
pub use __FullResult as FullResult;

/// An alias for `Result<(MappedRef<..>, _), _>`. Brevity is the sole purpose of this macro.
#[doc(hidden)]
#[macro_export]
macro_rules! __RefResult {
    ($Self:ident) => {
        ::core::result::Result<
            (
                $crate::map_full::MappedRef<
                    'new_data, 'new_upper,
                    $Self::NewN, $Self::NewR, $Self::Map, $Self::MapStrict,
                >,
                $Self::Ok,
            ),
            $Self::Err,
        >
    };
}

#[doc(inline)]
pub use __RefResult as RefResult;

/// An alias for `Result<(MappedRefMut<..>, _), _>`. Brevity is the sole purpose of this macro.
#[doc(hidden)]
#[macro_export]
macro_rules! __RefMutResult {
    ($Self:ident) => {
        ::core::result::Result<
            (
                $crate::map_full::MappedRefMut<
                    'new_data, 'new_upper,
                    $Self::NewN, $Self::NewM, $Self::Map, $Self::MapStrictest,
                >,
                $Self::Ok,
            ),
            $Self::Err,
        >
    };
}

#[doc(inline)]
pub use __RefMutResult as RefMutResult;

