//! Temporary organization module.

use core::convert::Infallible;

use variance_family::LendFamily;

use crate::slot::SelfRefCases;
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
            SelfRefCases::NoRef(_) | SelfRefCases::Ref(_) => &self.data.speed_bump,
            SelfRefCases::RefMut(infallible) => match infallible {},
        }
    }

    // map_non_mut
    // try_map_non_mut
    // map_non_mut_cloned
    // try_map_non_mut_cloned
}
