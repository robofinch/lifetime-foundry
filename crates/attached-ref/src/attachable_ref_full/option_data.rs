#![expect(unsafe_code, reason = "perform unsafe lifetime erasure and extension of self-refs")]

use core::hint::assert_unchecked;

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
    /// TODO: This can be paired with `wrap_data_in_option` to mix owned and borrowed data.
    ///
    /// See also [`AttachableRefFull::new_always_owned_ref`] if the possibility of borrowed data is
    /// not needed.
    ///
    /// [`Ref`]: SelfRefCases::Ref
    #[inline]
    #[must_use]
    pub const fn new_owned_ref(shared_ref: Lend<'data, &'upper (), R>) -> Self {
        Self::unattached_slot(None, SelfRefCases::Ref(shared_ref))
    }

    /// Construct a new [`AttachableRefFull`] in the [`RefMut`] state without actually having
    /// self-references.
    ///
    /// TODO: This can be paired with `wrap_data_in_option` to mix owned and borrowed data.
    ///
    /// See also [`AttachableRefFull::new_always_owned_mut`] if the possibility of borrowed data is
    /// not needed.
    ///
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
        if self.data.speed_bump_inner.is_none() {
            let (slot, none) = unsafe { self.into_raw_pieces() };

            unsafe {
                assert_unchecked(none.is_none());
            };

            Ok(slot)
        } else {
            Err(self)
        }
    }

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
