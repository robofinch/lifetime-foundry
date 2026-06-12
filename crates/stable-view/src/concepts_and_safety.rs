//! Documentation of concepts and safety concerns common across [`StableView`], [`StableViewMut`],
//! and [`StableClone`].
//!
//! Look through the crate-level overview before reading these details.
//!
//! # General Usage
//!
//! ## Parameters
//!
//! The implementor (`Self`) of [`StableView`] or [`StableViewMut`] is a view kind, such as
//! [`DefaultViewKind`], [`ReferenceViewKind`], [`RecursiveViewKind`], or [`UnstableViewKind`].
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
//! potential source of `'stable` data in the returned view. The `Self` parameter of some
//! traits, such as [`StableClone`], [`DefaultStableView`], and [`StableReferenceView`], is a `Data`
//! type rather than a view kind.
//!
//! Note that `'data` and `Data` do not have any direct relationship; they share a name as potential
//! sources of `'stable` data.
//!
//! ## Stable Data
//!
//! The `'stable` data in a returned view is split into two categories defined as follows:
//! - "long-lived" `'stable` data which remains valid for at least `'data`, regardless of what is
//!   done to the source `Data` value,
//! - "stable" `'stable` data.
//!
//! That is, the definition of "stable" `'stable` data can be taken as "all `'stable` data which is
//! not long-lived `'stable` data". (This definition is dependent on context for what `'stable` and
//! `'data` are.) Yes, this means that "stable data" -- without backticks and an apostrophe --
//! does not refer to all `'stable` data.
//!
//! However, any `'stable` data, whether stable or long-lived, is suitable for self-references to a
//! `Data` value in a self-referential struct (though only stable data is *actually*
//! self-referential in such cases). As a result, most user-facing documentation focuses on
//! `'stable` data and never mentions stable data.
//!
//! ## Bounds on Stable Data
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
//! - the source data value is manipulated only via the three (or two) kinds of operations
//!   specified by `StableView` (or `StableViewMut`, respectively).
//!
//! `StableView` (or `StableViewMut`, respectively) do not specify the impact of other kinds of
//! operations (besides the two or three permitted by the respective traits) on the validity of
//! stable data. That is, these requirements are a *lower* bound on when stable data is valid, so
//! you cannot generally assume that stable data is immediately invalidated after `'data` ends or
//! after a different operation is performed on the source data value. Indeed, implementing
//! `StableClone` raise that lower bound. More mechanisms like `StableClone` could be created and
//! used, whether within this crate or downstream.
//!
//! ## `StableClone` Pools
//!
//! `StableClone` defines the relevant "source" of stable data obtained via `StableView` in terms
//! of "conceptual pools" of values, rather than individual source data values. These pools are
//! analogous to pools of reference-counted pointers, though implementations of `StableClone` need
//! not *actually* be reference-counted; they merely need to behave as though they're
//! reference-counted, at least until `'data` ends.
//!
//! Imagine a pool of `Rc` clones pointing to the same allocation, a pool of `Arc`s, an entirely
//! imaginary infinite-size pool of `()`s, or an arbitrarily-defined pool of [`Infallible`]s that
//! can never actually exist anyway. Moreover, at least until `'data` ends, `&'data T` references
//! behave as though they're in an arbitrarily-large pool.
//!
//! Of course, `StableClone`'s constraints are more precise than
//! "clones of your type have to behave somewhat similarly to a pool of reference-counted pointers".
//! In short, `StableClone` places constraints on the definitions of conceptual pools, constrains
//! how certain operations may affect the size of a certain conceptual pool, associates stable data
//! with source conceptual pools, and requires that stable data remains valid while its source pool
//! is nonempty.
//!
//! Each `Data` type may have a different definition of conceptual pool used for views obtained via
//! `StableView::<'_, '_, Data>::view`. Note that we'd like to allow an `Rc<T>` and an
//! `Rc<dyn Trait>` pointing to the same allocation to be in the same conceptual pool, so
//! `StableClone` permits each conceptual pool to be filled with values of any type.
//!
//! Note that `StableClone` only interacts with `StableView`, not `StableViewMut`. The two traits
//! don't contradict each other, though. For example, an implementation of
//! `StableViewMut<'_, '_, Data: StableClone<'_>>` could act analogously to a type with a
//! *partially* reference-counted `Clone` implementation, such that the data obtained via `Deref` is
//! pooled in an `Rc` while the data obtained via `DerefMut` is individual to each value. A
//! reference-counted copy-on-write type -- including `Rc` and `Arc`, via [`Rc::make_mut`] and
//! [`Arc::make_mut`] -- could also allow `StableViewMut` and `StableClone` to be implemented with
//! the same `Data` type in a useful way.
//!
//! On the side of *using* `StableClone`, if `data_1` and `data_2` are in the same conceptual pool,
//! then (at your choice) you can soundly pretend (for the purposes of the rules of `StableView`)
//! that `'stable` data obtained from views of `data_1` had actually been obtained from views of
//! `data_2` just after `data_2` entered the conceptual pool.
//!
//! # `noalias` Types
//!
//! ## Status Quo
//!
//! `&mut T` and ([currently]) `Box<T>` cannot provide `&'stable T` or `&'stable mut T` references
//! to their direct contents; with Rust's current `noalias` semantics for those types, moving a
//! value of either of those types would assert exclusive access over its pointee, which could
//! invalidate views to its pointee.
//!
//! Since they assert the right to read or write the inline data of the `T`, but -- at least in
//! current versions of Rust -- do not invalidate doubly-indirected data, a `Box<Box<T>>` or
//! `&mut &mut T` can actually provide `&'stable T` or `&'stable mut T` views. However, I am not
//! sure whether that possibility is a stable guarantee or not. Until the aliasing model is more
//! settled, I suggest operating under the assumption that when a `&mut U` or `Box<U>` asserts its
//! `noalias` rights over a `&mut T` or `Box<T>`, it causes those pointers to *recursively* assert
//! their own `noalias` rights as well.
//!
//! Under that assumption, moving a `&mut T` or `Box<T>` has more-or-less the same impact as moving
//! a `T` itself. Thinking of a `Box<T>` as *basically* being a directly-owned `T`, but on the heap
//! instead of the stack, these semantics make sense.
//!
//! In any case, wrapping a `T` in a `&mut` or `Box` is *no worse* for its ability to provide
//! stable views than just storing the `T` directly inline.
//!
//! ## Changes
//!
//! Note that most other types, notably including `Vec<T>`, do not assert `noalias` over the data
//! they reference. That is [extremely unlikely to change], but if it ever does, this crate would
//! have to make a breaking change for the sake of soundness.
//!
//! Conversely, it seems like the compiler devs are leaning towards stripping `Box<T>` of its
//! `noalias` properties. If that happens, this crate may eventually deprecate and remove
//! its [`AliasableBox<T>`] type in a bump of the major version, since the standard library's `Box`
//! would at that point be a strictly better version of `AliasableBox`.
//!
//!
//! [`StableView`]: crate::traits::StableView
//! [`StableViewMut`]: crate::traits::StableViewMut
//! [`StableClone`]: crate::traits::StableClone
//! [`DefaultViewKind`]: crate::view_kinds::DefaultViewKind
//! [`ReferenceViewKind`]: crate::view_kinds::ReferenceViewKind
//! [`RecursiveViewKind`]: crate::view_kinds::RecursiveViewKind
//! [`UnstableViewKind`]: crate::provided_view_kinds::UnstableViewKind
//! [`DefaultStableView`]: crate::view_kinds::DefaultStableView
//! [`StableReferenceView`]: crate::view_kinds::StableReferenceView
//! [`StableView::view`]: crate::traits::StableView::view
//! [`StableViewMut::view_mut`]: crate::traits::StableViewMut::view_mut
//! [`Infallible`]: core::convert::Infallible
//! [`Arc::make_mut`]: https://doc.rust-lang.org/std/sync/struct.Arc.html#method.make_mut
//! [`Rc::make_mut`]: https://doc.rust-lang.org/std/rc/struct.Rc.html#method.make_mut
#![cfg_attr(feature = "alloc", doc = "[`AliasableBox<T>`]: crate::aliasable::AliasableBox")]
#![cfg_attr(not(feature = "alloc"), doc = "[`AliasableBox<T>`]: https://docs.rs/stable-view/0/stable_view/struct.AliasableBox.html")]
//!
//! [currently]: https://github.com/rust-lang/rfcs/pull/3712
//! [extremely unlikely to change]: https://github.com/rust-lang/rfcs/pull/3712#issuecomment-3715013712
