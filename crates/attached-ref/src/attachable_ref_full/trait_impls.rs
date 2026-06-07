#![expect(unsafe_code, reason = "perform unsafe lifetime erasure and extension of self-refs")]

use core::{cmp::Ordering, convert::Infallible};
use core::fmt::{Debug, Formatter, Result as FmtResult};

use stable_view::StableClone;
use variance_family::LendFamily;


use crate::slot::SelfRefCases;
use super::full_struct::{AttachableRefFull, SpeedBump};


/// Compares only based on the self-reference slot, not the backing data.
impl<'data, 'upper, N, R, M, Data> PartialEq for AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    N:      PartialEq,
    R:      LendFamily<&'upper (), Is: PartialEq>,
    M:      LendFamily<&'upper (), Is: PartialEq>,
    Data:   ?Sized,
{
    /// Compares only based on the self-reference slot, not the backing data.
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

/// Compares only based on the self-reference slot, not the backing data.
impl<'data, 'upper, N, R, M, Data> Eq for AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    N:      Eq,
    R:      LendFamily<&'upper (), Is: Eq>,
    M:      LendFamily<&'upper (), Is: Eq>,
    Data:   ?Sized,
{}

/// Compares only based on the self-reference slot, not the backing data.
impl<'data, 'upper, N, R, M, Data> PartialOrd for AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    N:      PartialOrd,
    R:      LendFamily<&'upper (), Is: PartialOrd>,
    M:      LendFamily<&'upper (), Is: PartialOrd>,
    Data:   ?Sized,
{
    /// Compares only based on the self-reference slot, not the backing data.
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get().partial_cmp(other.get())
    }
}

/// Compares only based on the self-reference slot, not the backing data.
impl<'data, 'upper, N, R, M, Data> Ord for AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    N:      Ord,
    R:      LendFamily<&'upper (), Is: Ord>,
    M:      LendFamily<&'upper (), Is: Ord>,
    Data:   ?Sized,
{
    /// Compares only based on the self-reference slot, not the backing data.
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.get().cmp(other.get())
    }
}



impl<'data, 'upper, N, R, Data> Clone for AttachableRefFull<'data, 'upper, N, R, Infallible, Data>
where
    'upper: 'data,
    N:      Clone,
    R:      LendFamily<&'upper (), Is: Clone>,
    Data:   StableClone<'data>,
{
    #[inline]
    fn clone(&self) -> Self {
        let (slot, data) = match self.get_full() {
            SelfRefCases::NoRef((no_ref, data)) => (SelfRefCases::NoRef(no_ref.clone()), data),
            SelfRefCases::Ref((self_ref, data)) => (SelfRefCases::Ref(self_ref.clone()), data),
            SelfRefCases::RefMut(&infallible)  => match infallible {},
        };

        // Even if this panics and unwinds, the fact that `self.data` is immutably borrowed
        // for this whole function body implies that any self-references in `maybe_ref` are
        // not invalidated (before they are dropped).
        let data = SpeedBump {
            speed_bump_inner: data.clone(),
        };

        unsafe { Self::from_slot(data, slot) }
    }
}

impl<'data, 'upper, N, R, M, Data> Debug for AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    N:      Debug,
    R:      LendFamily<&'upper (), Is: Debug>,
    M:      LendFamily<&'upper (), Is: Debug>,
    Data:   ?Sized + Debug,
{
    #[expect(clippy::missing_inline_in_public_items, reason = "in formatting, size matters more")]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let data = self.try_get_data();

        let data_dbg: &dyn Debug = if let Some(data) = data.as_ref() {
            data
        } else {
            &format_args!("<exclusively borrowed>")
        };

        f.debug_struct("AttachableRefFull")
            .field("slot",     &self.get())
            .field("variance", &self.variance)
            .field("data",     data_dbg)
            .finish()
    }
}
