#[cfg(feature = "alloc")]
use alloc::{boxed::Box, rc::Rc};
#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
use alloc::sync::Arc;

#[cfg(feature = "alloc")]
use stable_view::AliasableBox;


pub trait DynDrop: 'static {}

impl<T: ?Sized + 'static> DynDrop for T {}

#[cfg(feature = "alloc")]
pub type ErasedAliasableBox = AliasableBox<dyn DynDrop>;

#[cfg(feature = "alloc")]
pub type ErasedBox = Box<dyn DynDrop>;

#[cfg(feature = "alloc")]
pub type ErasedRc = Rc<dyn DynDrop>;

#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
pub type ErasedArc = Arc<dyn DynDrop + Send + Sync>;
