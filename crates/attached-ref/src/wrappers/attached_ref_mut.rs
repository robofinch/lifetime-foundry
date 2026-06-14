//! The [`AttachedRefMut`] wrapper.

use core::convert::Infallible;

use variance_family::LendFamily;

use crate::attachable_ref_full::AttachableRefFull;


#[expect(missing_debug_implementations, missing_docs, reason = "TODO")]
#[repr(transparent)]
pub struct AttachedRefMut<'data, M, Data>
where
    M:    LendFamily<&'data ()>,
    Data: ?Sized,
{
    /// `AttachedRefMut` is solely a more ergonomic interface for this inner field; it does not
    /// add any invariants on top of
    /// `AttachableRefFull<'data, 'data, Infallible, Infallible, M, Data>`.
    full: AttachableRefFull<'data, 'data, Infallible, Infallible, M, Data>,
}
