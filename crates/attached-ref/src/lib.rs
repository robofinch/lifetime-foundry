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

#![expect(missing_docs, clippy::missing_docs_in_private_items, clippy::missing_errors_doc, reason = "under development; TODO: remove")]

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;


mod slot;

mod init_support;
mod mapping_support;
mod non_null_option;

mod attachable_ref_full;
mod wrappers;


pub use self::{
    attachable_ref_full::AttachableRefFull,
    init_support::{TryAttachError, ViewMutToLend, ViewToLend},
    slot::{SelfRefCases, SelfRefSlot},
    wrappers::{AttachableRef, AttachableRefMut, AttachedRef, AttachedRefMut},
};


/// Supporting tools for changing generic parameters of `AttachableRefFull<'_, '_, _, _, _, _>`.
pub mod mapping {
    pub use crate::mapping_support::{FullResult, RefMutResult, RefResult};
    pub use crate::mapping_support::{DynDrop, MapBorrowedNonMut, MapNonMut, MapSlot};

    #[cfg(feature = "alloc")]
    pub use crate::mapping_support::{ErasedAliasableBox, ErasedBox, ErasedRc};
    #[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
    pub use crate::mapping_support::ErasedArc;
}
