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

// #![expect(missing_docs, clippy::missing_docs_in_private_items, clippy::missing_errors_doc, reason = "under development; TODO: remove")]

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;


mod erased;
mod error;
mod non_null_option;
mod outlives;
mod pre_1_94_closure_hack;
mod slot;

mod attachable_ref_full;
mod map_slot;

mod wrappers;


pub use self::{
    attachable_ref_full::AttachableRefFull,
    error::TryAttachError,
    pre_1_94_closure_hack::LendWrapper
};
pub use self::{
    outlives::{Outlives, OutlivesChain},
    slot::{SelfRefCases, SelfRefSlot, SelfRefSlotWrapper},
    wrappers::{AttachableRef, AttachableRefMut, AttachedRef, AttachedRefMut},
};


/// Supporting tools for changing generic parameters of `AttachableRefFull<'_, '_, _, _, _, _>`.
pub mod mapping {
    pub use crate::map_slot::{
        Branded, BrandFamily, CloneDataToken, DataToken, MapCases, MapClonedCases, MappedSlot,
        NonMutMapClonedToken, NoRefMapToken, RefMapToken, RefMutMapToken, TakeDataToken,
        VaryingMappedSlot,
    };
    pub use crate::erased::DynDestruct;

    #[cfg(feature = "alloc")]
    pub use crate::erased::{ErasedAliasableBox, ErasedBox, ErasedRc};
    #[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
    pub use crate::erased::ErasedArc;
}
