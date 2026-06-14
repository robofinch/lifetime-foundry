//! Temporary organization module.

use variance_family::{Lend, LendFamily};

use crate::slot::{SelfRefCases, SelfRefSlot};
use super::full_struct::AttachableRefFull;


impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Option<Data>>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    /// Construct a new [`AttachableRefFull`] in the [`Ref`] state without actually having
    /// self-references.
    ///
    /// This can be paired with [`wrap_data_in_some`] to mix owned and self-referential data.
    ///
    /// See also [`new_always_owned_ref`] if the possibility of self-referential data is not needed.
    ///
    /// [`wrap_data_in_some`]: AttachableRefFull::wrap_data_in_some
    /// [`new_always_owned_ref`]: AttachableRefFull::new_always_owned_ref
    /// [`Ref`]: SelfRefCases::Ref
    #[inline]
    #[must_use]
    pub const fn new_owned_ref(shared_ref: Lend<'data, &'upper (), R>) -> Self {
        Self::unattached_slot(None, SelfRefCases::Ref(shared_ref))
    }

    /// Construct a new [`AttachableRefFull`] in the [`RefMut`] state without actually having
    /// self-references.
    ///
    /// This can be paired with [`wrap_data_in_some`] to mix owned and self-referential data.
    ///
    /// See also [`new_always_owned_mut`] if the possibility of self-referential data is not needed.
    ///
    /// [`wrap_data_in_some`]: AttachableRefFull::wrap_data_in_some
    /// [`new_always_owned_mut`]: AttachableRefFull::new_always_owned_mut
    /// [`RefMut`]: SelfRefCases::RefMut
    #[inline]
    #[must_use]
    pub const fn new_owned_mut(exclusive_ref: Lend<'data, &'upper (), M>) -> Self {
        Self::unattached_slot(None, SelfRefCases::RefMut(exclusive_ref))
    }

    /// If `Data` is [`None`], then this struct is not actually protecting any self-referential
    /// data, and the slot for self-references can be safely obtained by-value.
    ///
    /// # Errors
    /// If `Data` is `Some`, `self` is returned back.
    #[inline]
    pub fn try_into_owned_slot(self) -> Result<SelfRefSlot<'data, 'upper, N, R, M>, Self> {
        if self.data.speed_bump.is_none() {
            let (_none, slot) = unsafe { self.into_raw_pieces() };

            Ok(slot)
        } else {
            Err(self)
        }
    }

    /// If `Data` is [`None`], then the backing data is set to the provided `data: NewData` value.
    ///
    /// Since stable self-references to `None` are not possible, the old data is soundly discarded.
    ///
    /// # Errors
    /// If `Data` is `Some`, `self` and the given `data: NewData` are returned back.
    #[inline]
    pub fn try_set_data<NewData>(
        self,
        data: NewData,
    ) -> Result<AttachableRefFull<'data, 'upper, N, R, M, NewData>, (Self, NewData)> {
        match self.try_into_owned_slot() {
            Ok(slot) => Ok(AttachableRefFull::unattached_slot(data, slot)),
            Err(old) => Err((old, data)),
        }
    }
}
