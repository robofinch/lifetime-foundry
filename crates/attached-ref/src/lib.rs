// See https://linebender.org/blog/doc-include for this README inclusion strategy
// File links are not supported by rustdoc
//!
//! [LICENSE-APACHE]: https://github.com/robofinch/attached-ref/blob/main/LICENSE-APACHE
//! [LICENSE-MIT]: https://github.com/robofinch/attached-ref/blob/main/LICENSE-MIT
//!
//!
//! <style>
//! .rustdoc-hidden { display: none; }
//! </style>
#![cfg_attr(doc, doc = include_str!("../README.md"))]

#![expect(warnings, reason = "under development; TODO: remove")]

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

// mod slot;
// mod erased_slot;

// mod variance;
// mod uninhabited_ref;

// mod attached_ref_slot;


// pub use self::{
//     // erase::{DefaultErase, EraseSelfRef, LifetimeErase},
//     slot::SelfRefSlot,
//     uninhabited_ref::{NeverExclusiveRef, NeverNoRef, NeverSharedRef},
//     variance::{Covariant, DataBound, DataVariance, Invariant},
// };
