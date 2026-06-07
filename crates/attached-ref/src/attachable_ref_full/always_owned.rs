#![expect(unsafe_code, reason = "perform unsafe lifetime erasure and extension of self-refs")]

use variance_family::{Lend, LendFamily};

use crate::slot::{SelfRefCases, SelfRefSlot};
use super::full_struct::AttachableRefFull;


impl<'data, 'upper, N, R, M> AttachableRefFull<'data, 'upper, N, R, M, ()>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    /// Construct a new [`AttachableRefFull`] in the [`Ref`] state which *cannot* be
    /// self-referential.
    ///
    /// This constructor is similar to [`AttachableRefFull::new_owned_ref`], but this type does
    /// not allow self-references. It may be useful in generic scenarios, though not as a concrete
    /// type.
    ///
    /// [`Ref`]: SelfRefCases::Ref
    #[inline]
    #[must_use]
    pub const fn new_always_owned_ref(shared_ref: Lend<'data, &'upper (), R>) -> Self {
        Self::unattached_slot((), SelfRefCases::Ref(shared_ref))
    }

    /// Construct a new [`AttachableRefFull`] in the [`RefMut`] state which *cannot* be
    /// self-referential.
    ///
    /// This constructor is similar to [`AttachableRefFull::new_owned_mut`], but this type does
    /// not allow self-references. It may be useful in generic scenarios, though not as a concrete
    /// type.
    ///
    /// [`RefMut`]: SelfRefCases::RefMut
    #[inline]
    #[must_use]
    pub const fn new_always_owned_mut(exclusive_ref: Lend<'data, &'upper (), M>) -> Self {
        Self::unattached_slot((), SelfRefCases::RefMut(exclusive_ref))
    }

    /// Get the data by-value.
    ///
    /// Since an [`AttachableRefFull<.., ()>`] does not allow self-references, no protection is
    /// actually needed for the self-reference slot.
    #[inline]
    #[must_use]
    pub fn into_owned_slot(self) -> SelfRefSlot<'data, 'upper, N, R, M> {
        let (slot, ()) = unsafe { self.into_raw_pieces() };

        slot
    }

    #[inline]
    #[must_use]
    pub fn set_data<Data>(self, data: Data) -> AttachableRefFull<'data, 'upper, N, R, M, Data> {
        AttachableRefFull::unattached_slot(data, self.into_owned_slot())
    }
}
