//! Temporary organization module.

use variance_family::{Lend, LendFamily};

use crate::{outlives::OutlivesChain, slot::SelfRefCases};
use super::full_struct::AttachableRefFull;


impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    Data:   ?Sized,
{
    /// Mutably access the self-referential data and, if possible, the backing data.
    ///
    /// New data can be introduced into the stored self-referential data, so long as that new
    /// data lives for at least `'data`.
    ///
    /// [`OutlivesChain`] provides `'data: 's` and `'s: 'a` implied bounds (and, currently, helps
    /// to guard against a compiler bug; see its documentation for more).
    #[inline]
    pub fn with_mut<'s, F, T>(&'s mut self, f: F) -> T
    where
        F: 'data + for<'a> FnOnce(
            SelfRefCases<
                (&'s mut N, &'s mut Data),
                &'a mut Lend<'s, &'upper (), R>,
                &'a mut Lend<'s, &'upper (), M>
            >,
            OutlivesChain<'data, 's, 'a>,
        ) -> T,
    {
        let unerased = unsafe { self.slot.unerase_mut() };
        let cases = match unerased {
            SelfRefCases::NoRef(no_ref)        => {
                SelfRefCases::NoRef((no_ref, &mut self.data.speed_bump))
            }
            SelfRefCases::Ref(self_ref)        => SelfRefCases::Ref(self_ref),
            SelfRefCases::RefMut(self_ref_mut) => SelfRefCases::RefMut(self_ref_mut),
        };

        f(cases, OutlivesChain::new())
    }

    /// Attempt to obtain a mutable reference to the backing data and [`NoRef`] data.
    ///
    /// Returns `None` if `self` is not in the [`NoRef`] state.
    ///
    /// [`NoRef`]: SelfRefCases::NoRef
    #[inline]
    #[must_use]
    pub const fn try_get_mut(&mut self) -> Option<(&mut N, &mut Data)> {
        let unerased = unsafe { self.slot.unerase_mut() };

        if let SelfRefCases::NoRef(no_ref) = unerased {
            Some((no_ref, &mut self.data.speed_bump))
        } else {
            None
        }
    }

    /// Attempt to obtain a mutable reference to the backing data, without invalidating any
    /// self-references.
    ///
    /// Returns `None` if `self` is not in the [`NoRef`] state.
    ///
    /// [`NoRef`]: SelfRefCases::NoRef
    #[inline]
    #[must_use]
    pub const fn try_get_data_mut(&mut self) -> Option<&mut Data> {
        if matches!(self.get(), SelfRefCases::NoRef(_)) {
            Some(&mut self.data.speed_bump)
        } else {
            None
        }
    }
}
