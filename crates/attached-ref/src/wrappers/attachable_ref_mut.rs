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
    full: AttachableRefFull<'data, 'data, N, Infallible, M, Data>,
}
