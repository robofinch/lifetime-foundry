// See https://linebender.org/blog/doc-include for this README inclusion strategy
// File links are not supported by rustdoc
//!
//! [LICENSE-APACHE]: https://github.com/robofinch/lifetime-foundry/blob/main/LICENSE-APACHE
//! [LICENSE-MIT]: https://github.com/robofinch/lifetime-foundry/blob/main/LICENSE-MIT
//!
//! [`CovariantFamily`]: CovariantFamily
//! [`ContravariantFamily`]: ContravariantFamily
//! [`UnvaryingFamily`]: UnvaryingFamily
//! [`LendFamily`]: LendFamily
//! [`shorten`]: shorten
//! [`lengthen]: lengthen
//! [`shorten_lend`]: shorten_lend
//! [`change_bounds_from`]: change_bounds_from
//! [`change_bounds_into`]: change_bounds_into
//! [`MaxUpperBound`]: MaxUpperBound
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
mod unvarying_family;
mod macros;
mod helper_functions;

// Note: the below implementations do NOT need to be exhaustive in order for this crate
// to be usable with arbitrary types. The implementations are solely for ergonomics, and are
// intended to reduce the number of times that someone needs to define a new lifetime family.
// In the event that a new lifetime family *is* needed, then hopefully the `macros` module
// makes it easier.

mod main_const_impls;
mod main_mut_impls;
mod main_fn_impls;

mod core_impls;

#[cfg(feature = "alloc")]
mod alloc_impls;

#[cfg(feature = "std")]
mod std_impls;

#[cfg(feature = "more-impls")]
mod more_core_impls;

#[cfg(feature = "more-impls")]
#[cfg(feature = "alloc")]
mod more_alloc_impls;

#[cfg(feature = "more-impls")]
#[cfg(feature = "std")]
mod more_std_impls;

#[cfg(feature = "either")]
mod either_impl;


pub use self::{
    main_const_impls::VaryingRef,
    main_mut_impls::VaryingRefMut,
    unvarying_family::Unvarying,
};
pub use self::{
    helper_functions::{change_bounds_from, change_bounds_into, lengthen, shorten, shorten_lend},
    macros::{assert_not_a_foreign_fundamental_type, assert_variance},
    traits::{
        ChangeBounds, ContravariantFamily, CovariantFamily, Lend, LendFamily, LifetimeFamily,
        MaxUpperBound, RawMutVarying, RawVarying, UnvaryingFamily, UpperBound, Varying,
        WithLifetime,
    },
};

/// Module for the `Cow<'varying, T>` family, called `VaryingCow<T>`.
pub mod borrow {
    #[cfg(feature = "alloc")]
    pub use crate::alloc_impls::VaryingCow;
}

/// Module for the `cell::Ref<'varying, T>` and `cell::RefMut<'varying, T>` families,
/// called `VaryingCellRef<T>` and `VaryingCellRefMut<T>`.
///
/// The word `Cell` is added to avoid a conflict with the names of the `&'varying T` and
/// `&'varying mut T` families.
pub mod cell {
    pub use crate::core_impls::{VaryingCellRef, VaryingCellRefMut};
}

/// Module for the `slice::Iter<'varying, T>` and `slice::IterMut<'varying, T>` families,
/// called `VaryingSlice{Iter, IterMut}<T>`.
pub mod slice {
    #[cfg(feature = "more-impls")]
    pub use crate::more_core_impls::{VaryingSliceIter, VaryingSliceIterMut};
}

/// Module for the `MutexGuard<'varying, T>`, `RwLockReadGuard<'varying, T>`, and
/// `RwLockWriteGuard<'varying, T>` families, called `Varying*Guard<T>`.
pub mod sync {
    #[cfg(feature = "std")]
    pub use crate::std_impls::VaryingMutexGuard;
    #[cfg(all(feature = "std", feature = "more-impls"))]
    pub use crate::more_std_impls::{VaryingRwLockReadGuard, VaryingRwLockWriteGuard};
}

#[cfg(feature = "either")]
pub use self::either_impl::EitherFamily;
