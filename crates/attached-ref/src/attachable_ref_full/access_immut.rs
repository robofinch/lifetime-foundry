//! Temporary organization module.

use variance_family::{Lend, LendFamily};

use crate::slot::{SelfRefCases, SelfRefSlot};
use super::full_struct::AttachableRefFull;


impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    Data:   ?Sized,
{
    /// Obtain a valid immutable/shared reference to potentially self-referential data.
    #[inline]
    #[must_use]
    pub const fn get(&self) -> &SelfRefSlot<'_, 'upper, N, R, M> {
        unsafe { self.slot.unerase_ref() }
    }

    // get_ref_or_insert_with

    // get_ref_or_try_insert_with

    // with_mut_ref_or_insert_with

    // with_mut_ref_or_try_insert_with

    // get_mut_or_insert_with

    // get_mut_or_try_insert_with

    // with_mut_mut_or_insert_with

    // with_mut_mut_or_try_insert_with

    /// Obtain a valid immutable/shared reference to potentially self-referential data and, if
    /// possible, the backing data.
    ///
    /// If `self` is currently in the [`RefMut`] state (meaning that there could be a mutable
    /// self-reference to the backing data), the backing data is not accessed.
    ///
    /// [`RefMut`]: SelfRefCases::RefMut
    #[expect(clippy::type_complexity, reason = "it's a one-off type, and there's a `_full` suffix")]
    #[inline]
    #[must_use]
    pub const fn get_full(&self) -> SelfRefCases<
        (&N, &Data),
        (&Lend<'_, &'upper (), R>, &Data),
        &Lend<'_, &'upper (), M>,
    > {
        match self.get() {
            SelfRefCases::NoRef(no_ref)        => {
                SelfRefCases::NoRef((no_ref, &self.data.speed_bump))
            }
            SelfRefCases::Ref(self_ref)        => {
                SelfRefCases::Ref((self_ref, &self.data.speed_bump))
            }
            SelfRefCases::RefMut(self_ref_mut) => SelfRefCases::RefMut(self_ref_mut),
        }
    }

    /// Attempt to obtain a valid immutable/shared reference to the backing data, without
    /// invalidating any self-references.
    ///
    /// If `self` is currently in the [`RefMut`] state (meaning that there could be a mutable
    /// self-reference to the backing data), the backing data is not accessed and `None` is
    /// returned.
    ///
    /// [`RefMut`]: SelfRefCases::RefMut
    #[inline]
    #[must_use]
    pub const fn try_get_data(&self) -> Option<&Data> {
        match self.get() {
            SelfRefCases::NoRef(_) | SelfRefCases::Ref(_) => Some(&self.data.speed_bump),
            SelfRefCases::RefMut(_) => None,
        }
    }
}
