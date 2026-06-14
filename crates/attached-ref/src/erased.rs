//! Erased `Data` types (using `dyn`) for keeping around source data solely for its destructor.

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, rc::Rc};
#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
use alloc::sync::Arc;

#[cfg(feature = "alloc")]
use stable_view::AliasableBox;


/// All `dyn Trait` vtables include a virtual destructor; this empty `DynDestruct` trait is
/// intended to be used as `dyn DynDestruct`, for the sake of that virtual destructor.
pub trait DynDestruct: 'static {}

impl<T: ?Sized + 'static> DynDestruct for T {}

/// An [`AliasableBox`] whose contents are type-erased.
#[cfg(feature = "alloc")]
pub type ErasedAliasableBox = AliasableBox<dyn DynDestruct>;

/// A [`Box`] whose contents are type-erased.
#[cfg(feature = "alloc")]
pub type ErasedBox = Box<dyn DynDestruct>;

/// An [`Rc`] whose contents are type-erased.
#[cfg(feature = "alloc")]
pub type ErasedRc = Rc<dyn DynDestruct>;

/// An [`Arc`] whose contents are type-erased *and* required to implement `Send` and `Sync`.
#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
pub type ErasedArc = Arc<dyn DynDestruct + Send + Sync>;
