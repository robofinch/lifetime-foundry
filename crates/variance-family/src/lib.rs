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
mod unvarying;
/// `covariant`, `contravariant`, and `unvarying` macros that cover common cases, in addition to
/// `recursive_covariant`, `recursive_contravariant`, `recursive_unvarying`, and
/// `covariant_for_recursive_unvarying` macros that require some `unsafe` to use.
///
/// Additionally, an `invariant_zst` macro mainly used for their backend is included.
mod macros;

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
/// `[T]`, `[T; N]`, `(T1, ..., Tn)`, `bool`, `char`, floats, ints, uints, `str`,
/// `cell::{Cell, Ref, RefCell, RefMut}`, `option::Option`, `pin::Pin`, `result::Result`,
///
/// and with the `more_impls` feature:
/// `cmp::Ordering`, `convert::Infallible`, `mem::{ManuallyDrop, MaybeUninit}`, `num::NonZero*`,
/// `ptr::NonNull`, `slice::Iter`, `sync::atomic::*`.
mod core_impls;

/// Implementations for:
/// `boxed::Box`, `borrow::Cow`, `rc::Rc`, `string::String`, `sync::Arc`, `vec::Vec`,
///
/// and with the `more_impls` feature:
/// `collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque}`, `rc::Weak`, `sync::Weak`.
#[cfg(feature = "alloc")]
mod alloc_impls;

/// Implementations for:
/// `path::{Path, PathBuf}`, `sync::{Mutex, MutexGuard}`,
///
/// and with the `more_impls` feature:
/// `cell::{OnceCell, LazyCell}`, `collections::{HashMap, HashSet}`, `io::Cursor`,
/// `sync::{Condvar, OnceLock, RwLock, RwLock{Read, Write}Guard, LazyLock}`.
#[cfg(feature = "std")]
mod std_impls;


pub use self::traits::{
    ContravariantFamily, CovariantFamily, ImplyBound, LendFamily, LifetimeFamily,
    MaxUpperBound, UnvaryingFamily, UpperBound, Varying, WithLifetime,
};
pub use self::main_const_impls::VaryingRef;
pub use self::main_mut_impls::VaryingRefMut;

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
