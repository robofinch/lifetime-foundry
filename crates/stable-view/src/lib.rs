// See https://linebender.org/blog/doc-include for this README inclusion strategy
// File links are not supported by rustdoc
//!
//! [LICENSE-APACHE]: https://github.com/robofinch/lifetime-foundry/blob/main/LICENSE-APACHE
//! [LICENSE-MIT]: https://github.com/robofinch/lifetime-foundry/blob/main/LICENSE-MIT
//!
//! [`StableView`]: StableView
//! [`StableViewMut`]: StableViewMut
//! [`StableClone`]: StableClone
//! [`AliasableRefMut<'_, T>`]: AliasableRefMut
#![cfg_attr(feature = "alloc", doc = " [`AliasableBox<T>`]: AliasableBox")]
#![cfg_attr(feature = "alloc", doc = " [`Box::leak`]: alloc::boxed::Box::leak")]
//! [`variance-family`]: variance_family
//!
//! <style>
//! .rustdoc-hidden { display: none; }
//! </style>
#![cfg_attr(doc, doc = include_str!("../README.md"))]

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;


mod traits;
mod view_kinds;
mod macros;
mod aliasable;

mod core_impls;

#[cfg(feature = "alloc")]
mod alloc_impls;

#[cfg(feature = "std")]
mod std_impls;

mod other_impls;


#[doc(hidden)]
pub mod __macro {
    pub use variance_family;

    pub use crate::macros::unsafe_recursive_view;
}


pub use self::{
    aliasable::{AliasableRefMut, VaryingAliasableRefMut},
    traits::{CustomView, CustomViewMut, StableClone, StableView, StableViewMut},
    view_kinds::{
        CollectionViewKind, DefaultViewKind, PointerViewKind, RecursiveViewKind, SetDefaultView,
        SetDefaultViewMut, UnstableViewKind, View, ViewMut, ZeroSizedViewKind,
    },
};
#[cfg(feature = "alloc")]
pub use self::aliasable::AliasableBox;
