#![expect(unsafe_code, reason = "perform unsafe lifetime erasure and extension of self-refs")]

use core::{hint::unreachable_unchecked, marker::PhantomData};

use variance_family::LendFamily;

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
            PhantomData<&'stable &'data ()>,
        ) -> T,
    {
        // Extra scope, to make sure that if `f(slot, PhantomData)` unwinds,
        // any references to `data` are necessarily dropped before `data` is dropped.
        let output = {
            let slot = unsafe { self.slot.into_unerased() };

            f(slot, PhantomData)
        };

        (output, self.data.speed_bump_inner)
    }

    /// Unsafely get the backing `Data` and the slot for self-references of `self`.
    ///
    /// After calling this function, `Data` could be mutated in a way that invalidates any
    /// self-references in the slot, possibly causing undefined behavior.
    ///
    /// You generally **should not** use this method; it is used internally to implement *safe*
    /// methods to retrieve the inner pieces. It is exposed mainly in case another library author
    /// experienced with `unsafe` wants to implement similar methods.
    ///
    /// # Safety
    /// The slot for self-references, `self.get()`, **must** currently have no `'stable`
    /// self-references to the `Data` value of `self`.
    ///
    /// This safety condition is certainly met when `Data` is a value like `()` or
    /// `Option::None`, or when `self` is in the [`NoRef`] state.
    ///
    /// [`NoRef`]: SelfRefCases::NoRef
    #[inline]
    #[must_use]
    pub unsafe fn into_raw_pieces(self) -> (SelfRefSlot<'data, 'upper, N, R, M>, Data) {
        // SAFETY: As robustly guaranteed by the `slot` field, the erased lifetime can be soundly
        // unerased into any `'stable` lifetime such that `'data: 'stable` and, at least until
        // `'stable` ends, `self.data` is not manipulated in a way that invalidates `self.slot`.
        // Our caller has `unsafe`ly asserted that `self.slot` has no `'stable` references to
        // `self.data`, so *no* manipulation of `self.data` (during at least `'data`) can invalidate
        // `self.slot`. Therefore, we can soundly choose `'stable = 'data`.
        let slot = unsafe { self.slot.into_unerased::<'data>() };

        // SAFETY INVARIANT: Our caller has `unsafe`ly asserted that `self.slot` has no `'stable`
        // references to `self.data`, so *no* manipulation of `self.data` (during at least `'data`)
        // can invalidate the `slot` value. Therefore, completely exposing `self.data` to
        // the caller's code is sound.
        let data = self.data.speed_bump_inner;

        (slot, data)
    }

    /// Get the backing `Data` by value, dropping the potentially self-referential data.
    #[inline]
    #[must_use]
    pub fn into_data(self) -> Data {
        drop(self.slot);
        // There are no more references to `data`.
        self.data.speed_bump_inner
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
            let (slot, data) = unsafe { self.into_raw_pieces() };

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
