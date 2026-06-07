mod shorthand_macros;
mod traits;
mod erased;


pub use self::shorthand_macros::{FullResult, RefMutResult, RefResult};
pub use self::erased::DynDrop;
pub use self::traits::{MapBorrowedNonMut, MapNonMut, MapSlot};

#[cfg(feature = "alloc")]
pub use self::erased::{ErasedAliasableBox, ErasedBox, ErasedRc};
#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
pub use self::erased::ErasedArc;
