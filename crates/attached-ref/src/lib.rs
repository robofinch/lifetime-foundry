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

#![expect(missing_docs, clippy::missing_docs_in_private_items, reason = "under development; TODO: remove")]

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;


mod slot;
mod erased_slot;
mod error;

mod map_data_impl;

mod full_impl;
mod wrappers;

mod closure_traits;
mod const_hack;


pub use self::error::TryAttachError;
pub use self::{
    closure_traits::{Compose, MapVarying, ViewMutToVarying, ViewToVarying},
    full_impl::AttachableRefFull,
    slot::{SelfRefCases, SelfRefSlot},
    wrappers::{AttachableRef, AttachableRefMut, AttachedRef, AttachedRefMut},
};


/// Complicated machinery that enables transformations that can change *every* generic parameter
/// of `AttachableRefFull<'_, '_, _, _, _, _>`.
///
/// This code should only be needed for *very* advanced usage.
/// (This machinery is also used internally.)
#[expect(clippy::module_name_repetitions, reason = "`full::MapFull` is acceptable")]
pub mod map_full {
    pub use crate::full_impl::{FullResult, RefMutResult, RefResult};
    pub use crate::full_impl::{MapFull, MapFullAndClone, MappedRef, MappedRefMut};
}

/// Machinery for mapping the backing `Data` values without invalidating self-references.
#[expect(clippy::module_name_repetitions, reason = "`map_data::MapDataStrict(est)` is acceptable")]
pub mod map_data {
    pub use crate::map_data_impl::{
        BlanketMapData, ComposeMaps, DynDrop, MapData, MapDataStrict, MapDataStrictest, MapViaAlloc,
        MapViaAltMove, MapViaMove, MapViaMoveInto,
    };

    #[cfg(feature = "alloc")]
    pub use crate::map_data_impl::RcUninit;

    #[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
    pub use crate::map_data_impl::ArcUninit;
}
