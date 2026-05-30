use core::convert::Infallible;

use variance_family::LendFamily;

use crate::full_impl::AttachableRefFull;


#[expect(missing_debug_implementations, missing_docs, reason = "TODO")]
#[repr(transparent)]
pub struct AttachedRefMut<'data, M, Data>
where
    M:    LendFamily<&'data ()>,
    Data: ?Sized,
{
    full: AttachableRefFull<'data, 'data, Infallible, Infallible, M, Data>,
}
