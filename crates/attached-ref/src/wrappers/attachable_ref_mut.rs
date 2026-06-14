//! The [`AttachableRefMut`] wrapper.

use core::convert::Infallible;

use variance_family::LendFamily;

use crate::attachable_ref_full::AttachableRefFull;


#[expect(missing_debug_implementations, missing_docs, reason = "TODO")]
#[repr(transparent)]
pub struct AttachableRefMut<'data, N, M, Data>
where
    M:    LendFamily<&'data ()>,
    Data: ?Sized,
{
    /// `AttachableRefMut` is solely a more ergonomic interface for this inner field; it does not
    /// add any invariants on top of `AttachableRefFull<'data, 'data, N, Infallible, M, Data>`.
    full: AttachableRefFull<'data, 'data, N, Infallible, M, Data>,
}
