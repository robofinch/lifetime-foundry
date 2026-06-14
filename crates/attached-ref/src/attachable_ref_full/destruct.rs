//! Temporary organization module.

use core::hint::unreachable_unchecked;

use variance_family::LendFamily;

use crate::outlives::Outlives;
use crate::slot::{SelfRefCases, SelfRefSlot};
use super::full_struct::AttachableRefFull;


impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    /// Deconstruct `self` into owned pieces, including `Data`. The provided callback is prevented
    /// from returning self-referential data (since self-references to `Data` could be invalidated
    /// after this function returns).
    ///
    /// [`Outlives`] provides a `'data: 'stable` implied bound (and, currently, helps to guard
    /// against a compiler bug; see the documentation of [`OutlivesChain`] for more).
    ///
    /// [`OutlivesChain`]: crate::outlives::OutlivesChain
    #[inline]
    #[must_use]
    pub fn into_pieces<F, T>(self, f: F) -> (T, Data)
    where
        // Ranges over `'stable` such that `'data: 'stable`.
        //
        // **Critically**, there are no implied lower bounds on `'stable`, despite `T`
        // potentially causing some concern. See the reasoning of `ViewToLend`.
        F: for<'stable> FnOnce(
            SelfRefSlot<'stable, 'upper, N, R, M>,
            Outlives<'data, 'stable>,
        ) -> T,
    {
        // Extra scope, to make sure that if `f(slot, PhantomData)` unwinds,
        // any references to `data` are necessarily dropped before `data` is dropped.
        let output = {
            let slot = unsafe { self.slot.into_unerased() };

            f(slot, Outlives::new())
        };

        (output, self.data.speed_bump)
    }

    /// Get the backing `Data` by value, dropping the potentially self-referential data.
    #[inline]
    #[must_use]
    pub fn into_data(self) -> Data {
        drop(self.slot);
        // There are no more references to `data`.
        self.data.speed_bump
    }

    /// Attempt to get both the backing `Data` and [`NoRef`] data by value.
    ///
    /// # Errors
    /// This method succeeds if and only if the slot for self-references is in the [`NoRef`] state.
    ///
    /// On error, the given `self` is passed back.
    ///
    /// [`NoRef`]: SelfRefCases::NoRef
    #[inline]
    pub fn try_into_owned(self) -> Result<(N, Data), Self> {
        if matches!(self.get(), SelfRefCases::NoRef(_)) {
            let (data, slot) = unsafe { self.into_raw_pieces() };

            let no_ref = match slot {
                SelfRefCases::NoRef(no_ref) => no_ref,
                SelfRefCases::Ref(_) | SelfRefCases::RefMut(_) => {
                    unsafe { unreachable_unchecked() }
                }
            };

            Ok((no_ref, data))
        } else {
            Err(self)
        }
    }
}
