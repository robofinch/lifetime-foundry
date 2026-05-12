//! Aliasable versions of `&mut T` and `Box<T>` which don't invalidate pointers to their pointee
//! when moved.
//!
//! That is, these types allow their pointees to be aliased.
//
// Note that we cannot use the `aliasable` crate, since as of January 2026, it has not been updated
// for 4-5 years and has substantial UB. No clue why the changes on its repo haven't been pushed.

mod aliasable_ref_mut;
#[cfg(feature = "alloc")]
mod aliasable_box;
mod families;

// Currently, `Vec` and friends are already aliasable. If that ever changes for whatever reason,
// this crate will make a breaking change to remove `StableView(Mut)` impls for `Vec` and friends
// and create `AliasableVec`, `AliasableString`, `AliasableCowSlice`, etc.


pub use self::{aliasable_ref_mut::AliasableRefMut, families::VaryingAliasableRefMut};
#[cfg(feature = "alloc")]
pub use self::aliasable_box::AliasableBox;
