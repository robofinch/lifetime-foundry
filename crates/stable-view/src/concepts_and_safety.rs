//! Documentation of concepts and safety concerns common across [`StableView`], [`StableViewMut`],
//! and [`StableClone`].
//!
//! Look through the crate-level overview before reading these details.
//!
//! # Parameters
//!
//! The implementor, `Self`, of [`StableView`] or [`StableViewMut`] is a view kind, such as
//! [`PointerViewKind`] or [`DefaultViewKind`].
//!
//! `'a` represents a short lifetime with no extra guarantees beyond the languages's invariants
//! enforced by the borrow checker.
//!
//! `'stable` represents the lifetime of data which can be accessed longer than usual; this lifetime
//! is the one which can be soundly (and `unsafe`ly) lifetime-extended in specific conditions.
//! The views are required to be covariant over `'stable`.
//!
//! `'data` represents a long lifetime; all views will stop being used before `'data` ends.
//! A covariant `'data` lifetime could be shortened to `'stable`, making long-lived data that
//! outlives `'data` a potential source of `'stable` data in the returned view.
//!
//! `Data` is the type of the source data of the view. Depending on `unsafe` details, it is a
//! potential source of `'stable` data in the returned view. The `Self` parameter of [`StableClone`]
//! is a `Data` type rather than a view kind.
//!
//! Note that `'data` and `Data` do not have any direct relationship; they share a name as potential
//! sources of `'stable` data.
//!
//! # Stable Data
//!
//! The `'stable` data in a returned view is split into two categories defined as follows:
//! - long-lived `'stable` data which remains valid for at least `'data`, regardless of what is
//!   done to the source `Data` value,
//! - stable `'stable` data.
//!
//! That is, the definition of "stable" `'stable` data can be taken as "all `'stable` data which is
//! not long-lived `'stable` data". (This definition is dependent on context for what `'stable` and
//! `'data` are.) Yes, this means that "stable data" -- without backticks and an apostrophe --
//! does not refer to all `'stable` data.
//!
//! `StableView`, `StableViewMut`, and `StableClone` place requirements on the validity of stable
//! data, but not long-lived data. ("Do not use this data after `'data` ends" is a sufficient
//! requirement for long-lived data.)
//!
//! `StableView` (or `StableViewMut`, respectively) requires that stable data remains valid,
//! starting from the time it is obtained (via [`StableView::view`] or [`StableViewMut::view_mut`],
//! respectively, applied to some source data value), while both:
//! - `'data` has not ended (in other words, both long-lived and stable data could be invalidated
//!   after `'data` ends), **and**
//! - the source data value is manipulated only via three kinds of operations specified by
//!   `StableView` (or `StableViewMut`, respectively).
//!
//! `StableView` (or `StableViewMut`, respectively) do not specify the impact of other kinds of
//! operations (besides the three permitted by the respective traits) on the validity of stable
//! data. That is, these requirements are a *lower* bound on when stable data is valid, so you
//! cannot generally assume that stable data is immediately invalidated after `'data` ends or after
//! a different operation is performed on the source data value. Indeed, implementing `StableClone`
//! raise that lower bound. More mechanisms like `StableClone` could be created and used,
//! whether within this crate or downstream.
//!
//! `StableClone` defines the relevant "source" of stable data obtained via `StableView` in terms
//! of "conceptual pools" of values, rather than individual source data values. Each `Data` type
//! may have a different definition of conceptual pool used for views obtained via
//! `StableView::<'_, '_, Data>::view`. Imagine a pool of `Rc` clones pointing to the same
//! allocation, a pool of `Arc`s, an entirely imaginary infinite-size pool of `()`s, or an
//! arbitrarily-defined pool of [`Infallible`]s (which cannot have views taken of them anyway). Note
//! that we'd like to allow an `Rc<T>` and an `Rc<dyn Trait>` to be in the same conceptual pool, so
//! `StableClone` permits definitions of "conceptual pool" to be filled with values of any type.
//!
// TODO: yeahhhhhhh I'm changing `StableClone`
//!
//! `StableClone` places constraints on the definitions of conceptual pools, constrains how certain
//! operations may affect the size of a certain conceptual pool, associates stable data with source
//! conceptual pools, and requires that stable data remains valid while its source pool is nonempty.
//!
//! Note that `StableClone` only interacts with `StableView`, not `StableViewMut`. The two traits
//! don't contradict each other, though. For example, an implementation of
//! `StableViewMut<'_, '_, Data: StableClone>` could act analogously to a type with a *partially*
//! reference-counted `Clone` implementation, such that the data obtained via `Deref` is pooled in
//! an `Rc` while the data obtained via `DerefMut` is individual to each value. A
//! reference-counted copy-on-write type -- including `Rc` and `Arc`, via [`Rc::make_mut`] and
//! [`Arc::make_mut`] -- could also allow `StableViewMut` and `StableClone` to be implemented with
//! the same `Data` type in a useful way.
//!
//! [`StableView`]: crate::traits::StableView
//! [`StableViewMut`]: crate::traits::StableViewMut
//! [`StableClone`]: crate::traits::StableClone
// TODO: other links
