// See https://linebender.org/blog/doc-include for this README inclusion strategy
// File links are not supported by rustdoc
//!
//! [LICENSE-APACHE]: https://github.com/robofinch/attached-ref/blob/main/LICENSE-APACHE
//! [LICENSE-MIT]: https://github.com/robofinch/attached-ref/blob/main/LICENSE-MIT
//!
//! [`AliasableView`]: AliasableView
//! [`AliasableViewMut`]: AliasableViewMut
//! [`CloneableAliasable`]: CloneableAliasable
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


// mod traits;
// mod aliasable;
// mod core_impls;

// #[cfg(feature = "alloc")]
// mod alloc_impls;

// #[cfg(feature = "std")]
// mod std_impls;

// mod other_impls;

// pub use self::{
//     aliasable::AliasableRefMut,
//     traits::{
//         AliasableClone, AliasableView, AliasableViewMut,
//         IntoAliasable, IntoAliasableMut, View, ViewMut,
//     },
// };
