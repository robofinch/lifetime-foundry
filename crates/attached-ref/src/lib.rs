// See https://linebender.org/blog/doc-include for this README inclusion strategy
// File links are not supported by rustdoc
//!
//! [LICENSE-APACHE]: https://github.com/robofinch/lifetime-foundry/blob/main/LICENSE-APACHE
//! [LICENSE-MIT]: https://github.com/robofinch/lifetime-foundry/blob/main/LICENSE-MIT
//!
//!
//! <style>
//! .rustdoc-hidden { display: none; }
//! </style>
#![cfg_attr(doc, doc = include_str!("../README.md"))]

#![expect(clippy::missing_docs_in_private_items, reason = "under development; TODO: remove")]

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;


mod slot;
mod erased_slot;
mod error;

mod non_null_option;

mod full_impl;
mod wrappers;

mod closure_traits;


pub use self::error::TryAttachError;
pub use self::{
    closure_traits::{ViewMutToLend, ViewToLend},
    full_impl::AttachableRefFull,
    slot::{SelfRefCases, SelfRefSlot},
    wrappers::{AttachableRef, AttachableRefMut, AttachedRef, AttachedRefMut},
};


/// Complicated machinery that enables transformations that can change every self-reference
/// parameter of `AttachableRefFull<'_, '_, _, _, _, _>`.
///
/// This code should only be needed for advanced usage.
/// (This machinery is also used internally.)
#[expect(clippy::module_name_repetitions, reason = "`full::MapFull` is acceptable")]
pub mod map_full {
    pub use crate::full_impl::{FullResult, RefMutResult, RefResult};
    pub use crate::full_impl::{MapFull, MapFullClone};
}
