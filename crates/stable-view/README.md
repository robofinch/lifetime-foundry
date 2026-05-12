<div align="center" class="rustdoc-hidden">
<h1> Stable View </h1>
</div>

[<img alt="github" src="https://img.shields.io/badge/github-stable--view-08f?logo=github" height="20">](https://github.com/robofinch/lifetime-foundry/tree/main/crates/stable-view)
[![Latest version](https://img.shields.io/crates/v/stable-view.svg)](https://crates.io/crates/stable-view)
[![Documentation](https://img.shields.io/docsrs/stable-view)](https://docs.rs/stable-view/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

# Overview

Get temporary, but somewhat stable, "views" of values.

This crate provides [`StableView`], [`StableViewMut`], and [`StableClone`] traits
which prohibit certain actions, such as moving a value, from invalidating the temporary views
of values implementing the traits.

The traits are intended to be useful for self-referential structs; aliasable source data can be
stored alongside a view to that data (or values derived from views to that data), and so long as
the conditions laid out in the traits are satisfied, the views can be continue to be used
(perhaps via `unsafe` lifetime extension) even when Rust's normal borrow checking and aliasing
rules would ordinarily make such a struct impossible or unsound to implement.

The traits also make use of [`variance-family`] in order to give implementations substantial freedom
over what their view types are; rather than a plain `&'stable Self::Target` reference (as with the
`Deref` trait), a view can be an arbitrary type (parameterized by several lifetimes, if needed). For
instance, the default implementation of `StableView<'_, '_, Option<T>>` sets
`View<'_, '_, '_, Option<T>>` to `Option<View<'_, '_, '_, T>>`.

# Architecture

TODO: go over how the system works

# `noalias` Types

`&mut T` and ([currently]) `Box<T>` cannot provide `&'stable T` or `&'stable mut T` references
to their direct contents; with Rust's current `noalias` semantics for those types, moving a value
of either of those types would assert exclusive access over its pointee, which could invalidate
views to its pointee. (You can think of moving a `&mut T` or `Box<T>` as being very similar to
moving a `T` itself.)

Therefore, aliasable versions of those types, [`AliasableRefMut<'_, T>`] and [`AliasableBox<T>`],
are provided. Most other common types, including `Vec<T>`, do not assert exclusive access over
data they reference when they are moved. That is [very unlikely to change], but if it ever does,
this crate will have to make a breaking change for the sake of soundness. Conversely, if `Box<T>`
loosens its aliasing requirements, this crate may eventually deprecate and remove `AliasableBox<T>`
in a bump of the major version.

# Prior Art

The [`stable_deref_trait`] crate also offers a trait intended for use with self-referential structs,
but requiring that the reference returned by `Deref::deref` refers to the same address even if the
source value is moved does not seem critical for the soundness of self-referential structs; this
crate's traits more narrowly focus on the properties needed for lifetime transmutes (or lifetime
erasure) of self-references to be sound in self-referential structs.

That does imply, for example, that a wacky implementation of [`StableViewMut`] can soundly return
a different value from [`Box::leak`] on each call, "views" of a `MyString` type may be provided by
cloning a source string on every call, and a `RecursiveView<()>` may implement [`StableView`] for
`MyVec<T>` by returning a `&T` view to an element which is randomly chosen on each call. Such
oddities are probably not very useful, but neither do they harm soundness.

(Moreover, the idea of a "stable" deref does not extend well to arbitrary lifetime-infected types.)

The [`aliasable`] crate also provides aliasable versions of `&mut T` and `Box<T>`, but the version
on crates.io at the time of writing is unsound. Therefore, I decided to implement my own
versions (complete with extensive documentation about soundness and aliasing guarantees).

# License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE][])
* MIT license ([LICENSE-MIT][])

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.

[LICENSE-APACHE]: LICENSE-APACHE
[LICENSE-MIT]: LICENSE-MIT

[`StableView`]: https://docs.rs/stable-view/0/stable_view/trait.StableView.html
[`StableViewMut`]: https://docs.rs/stable-view/0/stable_view/trait.StableViewMut.html
[`StableClone`]: https://docs.rs/stable-view/0/stable_view/trait.StableClone.html
[`AliasableRefMut<'_, T>`]: https://docs.rs/stable-view/0/stable_view/struct.AliasableRefMut.html
[`AliasableBox<T>`]: https://docs.rs/stable-view/0/stable_view/struct.AliasableBox.html
[`Box::leak`]: https://doc.rust-lang.org/std/boxed/struct.Box.html#method.leak

[`variance-family`]: https://docs.rs/variance-family/0/variance_family
[`stable_deref_trait`]: https://docs.rs/stable_deref_trait/1.2.1/stable_deref_trait/index.html
[`aliasable`]: https://docs.rs/aliasable/0.1.3/aliasable/

[currently]: https://github.com/rust-lang/rfcs/pull/3712
[very unlikely to change]: https://github.com/rust-lang/rfcs/pull/3712#issuecomment-3715013712
