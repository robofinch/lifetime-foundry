//! The [`AttachedRef`] wrapper.

use core::convert::Infallible;

use variance_family::LendFamily;

use crate::attachable_ref_full::AttachableRefFull;


#[expect(missing_debug_implementations, missing_docs, reason = "TODO")]
#[repr(transparent)]
pub struct AttachedRef<'data, R, Data>
where
    R:    LendFamily<&'data ()>,
    Data: ?Sized,
{
    /// `AttachedRef` is solely a more ergonomic interface for this inner field; it does not
    /// add any invariants on top of
    /// `AttachableRefFull<'data, 'data, Infallible, R, Infallible, Data>`.
    inner: AttachableRefFull<'data, 'data, Infallible, R, Infallible, Data>,
}
