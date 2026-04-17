//! Implementations for:
//!
//! - `[T]`,
//! - `[T; N]`,
//! - `(T1, ..., Tn)` of arities 0-12,
//! - primitives (`bool`, `char`, `f32`, `f64`, `i{N}`, `u{N}`, `str`)
//! - `cell::{Cell, Ref, RefCell, RefMut}`,
//! - `option::Option`,
//! - `pin::Pin`,
//! - `result::Result`.
//!
//! With the `more_impls` feature, also:
//!
//! - `cmp::Ordering`,
//! - `convert::Infallible`,
//! - `mem::{ManuallyDrop, MaybeUninit}`,
//! - `num::NonZero*`,
//! - `ptr::NonNull`,
//! - `slice::Iter`,
//! - `sync::atomic::*`.
