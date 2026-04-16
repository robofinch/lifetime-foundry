// See https://linebender.org/blog/doc-include for this README inclusion strategy
// File links are not supported by rustdoc
//!
//! [LICENSE-APACHE]: https://github.com/robofinch/attached-ref/blob/main/LICENSE-APACHE
//! [LICENSE-MIT]: https://github.com/robofinch/attached-ref/blob/main/LICENSE-MIT
//!
//! [`CovariantFamily`]: CovariantFamily
//! [`ContravariantFamily`]: ContravariantFamily
//! [`UnvaryingFamily`]: UnvaryingFamily
//! [`LendFamily`]: LendFamily
//!
//! <style>
//! .rustdoc-hidden { display: none; }
//! </style>
#![cfg_attr(doc, doc = include_str!("../README.md"))]

#![no_std]
#![expect(unsafe_code, reason = "allow unsafe code to rely on the marker trait impls")]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

/// The traits which are the central purpose of this crate.
mod traits;
/// An `Unvarying` type that implements `UnvaryingFamily`, greatly useful for trivial families not
/// implemented here.
mod unvarying_family;
/// `covariant`, `contravariant`, and `unvarying` macros that cover simple cases.
///
/// Additionally, an `invariant_zst` macro mainly used for their backend is included.
mod macros;
/// `shorten`, `lengthen`, `shorten_lend`, `change_bounds_from`, and `change_bounds_into` functions.
///
/// These are useful for consumers of lifetime families.
mod helper_functions;

// Note: the below implementations do NOT need to be exhaustive in order for this crate
// to be usable with arbitrary types. The implementations are solely for ergonomics, and are
// intended to reduce the number of times that someone needs to define a new lifetime family.
// In the event that a new lifetime family *is* needed, then hopefully the `macros` module
// makes it easier.

/// Implementations for `&'a T`, `&'varying T` (as `VaryingRef<T>`), and `*const T`.
mod main_const_impls;
/// Implementations for `&'a mut T`, `&'varying mut T` (as `VaryingMut<T>`), and `*mut T`.
mod main_mut_impls;
/// Implementations for `fn(..Args) -> R` for arities 0-12.
mod main_fn_impls;

/// Implementations for:
///
/// - `[T]`,
/// - `[T; N]`,
/// - `(T1, ..., Tn)`,
/// - primitives (`bool`, `char`, `f32`, `f64`, `i{N}`, `u{N}`, `str`)
/// - `cell::{Cell, Ref, RefCell, RefMut}`,
/// - `option::Option`,
/// - `pin::Pin`,
/// - `result::Result`.
///
/// With the `more_impls` feature, also:
///
/// - `cmp::Ordering`,
/// - `convert::Infallible`,
/// - `mem::{ManuallyDrop, MaybeUninit}`,
/// - `num::NonZero*`,
/// - `ptr::NonNull`,
/// - `slice::Iter`,
/// - `sync::atomic::*`.
mod core_impls;

/// Implementations for:
///
/// - `boxed::Box`,
/// - `borrow::Cow`,
/// - `rc::Rc`,
/// - `string::String`,
/// - `sync::Arc`,
/// - `vec::Vec`,
///
/// With the `more_impls` feature, also:
///
/// - `collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque}`,
/// - `rc::Weak`,
/// - `sync::Weak`.
#[cfg(feature = "alloc")]
mod alloc_impls;

/// Implementations for:
///
/// - `path::{Path, PathBuf}`,
/// - `sync::{Mutex, MutexGuard}`.
///
/// With the `more_impls` feature, also:
///
/// - `cell::{OnceCell, LazyCell}`,
/// - `collections::{HashMap, HashSet}`,
/// - `io::Cursor`,
/// - `sync::{Condvar, OnceLock, RwLock, RwLock{Read, Write}Guard, LazyLock}`.
#[cfg(feature = "std")]
mod std_impls;


pub use self::{main_const_impls::VaryingRef, main_mut_impls::VaryingRefMut};
pub use self::{
    helper_functions::{change_bounds_from, change_bounds_into, lengthen, shorten, shorten_lend},
    traits::{
        ChangeBounds, ContravariantFamily, CovariantFamily, Lend, LendFamily, LifetimeFamily,
        MaxUpperBound, RawMutVarying, RawVarying, UnvaryingFamily, UpperBound, Varying,
        WithLifetime,
    },
};

/// Module for the `Cow<'varying, T>` family, called `VaryingCow<T>`.
pub mod borrow {}
/// Module for the `cell::Ref<'varying, T>` and `cell::RefMut<'varying, T>` families,
/// called `VaryingCellRef<T>` and `VaryingCellRefMut<T>`.
///
/// The word `Cell` is added to avoid a conflict with the names of the `&'varying T` and
/// `&'varying mut T` families.
pub mod cell {}
/// Module for the `slice::Iter<'varying, T>` family, called `VaryingSliceIter<T>`.
pub mod slice {}
/// Module for the `MutexGuard<'varying, T>`, `RwLockReadGuard<'varying, T>`, and
/// `RwLockWriteGuard<'varying, T>` families, called `Varying*Guard<T>`.
pub mod sync {}
