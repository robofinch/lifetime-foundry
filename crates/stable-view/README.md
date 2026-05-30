<div align="center" class="rustdoc-hidden">
<h1> Stable View </h1>
</div>

[<img alt="github" src="https://img.shields.io/badge/github-stable--view-08f?logo=github" height="20">](https://github.com/robofinch/lifetime-foundry/tree/main/crates/stable-view)
[![Latest version](https://img.shields.io/crates/v/stable-view.svg)](https://crates.io/crates/stable-view)
[![Documentation](https://img.shields.io/docsrs/stable-view)](https://docs.rs/stable-view/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

# Overview

Get temporary "views" whose `'stable` data remains valid even when the views' source data is moved.

This crate provides [`StableView`], [`StableViewMut`], and [`StableClone`] traits
which prohibit certain actions, such as moving a value, from invalidating the temporary views
of values implementing the traits.

Generally, this crate's traits are low-level glue. It is expected that most transitive users will
not need to implement a `Stable*` trait at all; this crate already provides numerous `Stable*`
trait implementations (though custom implementations for non-`std` data structures may be
necessary). Neither is it expected that most transitive users will need to directly call an `unsafe`
method or perform `unsafe` lifetime extension when using the `Stable*` traits. For example, after
[`attached-ref`] deals with the `unsafe` parts of obtaining a view itself, the views can be soundly
exposed to safe user code. See also [`Viewer`], which allows safe code to obtain views
(after a `Viewer` is created in upstream `unsafe` code).

The traits are intended to be useful for self-referential structs; aliasable source data can be
soundly stored alongside values containing `'stable` references (obtained via a stable view trait)
to that source data, even when Rust's normal borrow checking and aliasing rules would ordinarily
make such a struct impossible or unsound to implement.

With this advanced `unsafe` target audience, it is quite difficult to either implement these traits
or directly use their methods and guarantees in `unsafe` code; ideally, you should not need to
directly interact with this crate, besides perhaps forwarding `V: StableView<..>` bounds (or
similar) to other libraries in generic code. (As the author of both this crate and [`attached-ref`],
it's no longer difficult for me to understand how this crate works, but I assume that it'd be
difficult for others to learn.)

# Architecture

In order to allow custom third-party views of other crates' types, the `StableView` and
`StableViewMut` traits are not implemented directly for types like `Vec<T>` or `Rc<T>`. Instead,
the source data type is passed as a generic parameter to a stable view trait, and the trait is
implemented for a "view kind" trait that indicates how the view is obtained.

For example, [`PointerViewKind`] implements `StableView<'_, '_, Rc<T>>` and
[`RecursiveViewKind<(VT, VE)>`] implements `StableView<'_, '_, Result<T, E>>`.

An [`UnstableViewKind`] type implements the stable view traits for all source data types in a trivial
way, by not putting any `'stable` data in its provided views.

Still, it is useful for a type to indicate how it is *expected* to provide views in the most common
case. For this purpose, [`DefaultViewKind`] is provided; libraries generic over view kinds may
wish to provide special-case methods for [`DefaultViewKind`]. For example, [`attached-ref`]
has `attach_ref_with` methods which need a view kind to be specified, while `attach_ref` defaults
to [`DefaultViewKind`].

The data types, rather than view types, do directly implement `StableClone`. Implementors of
`StableClone` need to make certain guarantees about *all* possible views of that type, rather than
each view kind making individual guarantees. (The latter approach was attempted first, but that
turned out to not actually be very useful.)

Note that the view kind types do not actually enforce any particular semantics on their stable view
trait implementations (beyond the traits' own requirements), whether for soundness or just
correctness; they are solely intended to be human-understandable categories. One-off dummy types
would work just as well for view kinds (aside from concerns about ergonomics or readability).

The traits also make use of [`variance-family`] in order to give implementations substantial freedom
over what their view types are; rather than a plain `&'stable Self::Target` reference (as with the
`Deref` trait), a view can be an arbitrary type (parameterized by several lifetimes, if needed). For
instance, the default implementation of `StableView<'a, 'other_data, Option<T>>` sets
`View<'a, 'stable, 'other_data, Option<T>>` to `Option<View<'a, 'stable, 'other_data, T>>`.

## Strength of `StableClone`
Glossing over details, note that `StableClone` treats `data` and `data.clone()` equivalently to a
greater extent than currently needed. In particular, [`attached-ref`] currently needs stable data
valid at the moment `data_clone = data.clone()` is called to be kept valid so long as either `data`
or `data_clone` are manipulated through permitted operations. However, it doesn't need views of
`data` obtained *after* the creation of `data_clone` to be kept valid by `data_clone`, and it
doesn't need views of `data_clone` to be kept valid by `data`. *Conceivably*, some sort of linked
list such that clones have a shared tail but different heads, or some sort of tree-based structure,
might have properties that *would* be sufficient for usage in [`attached-ref`], if not for the
excessively strong restrictions of `StableClone`.

The mental model of `StableClone` where `data` and `data.clone()` are placed in the same pool
would be insufficient for the linked structure; [...]

...I think the "conceptual pool" model is overcomplicated, actually.

Mmmm. I should cut back.


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
cloning a source string on every call, and `RecursiveView<()>` may implement [`StableView`] for
`MyVec<T>` by returning a `&T` view to an element which is randomly chosen on each call. Such
oddities are probably not very useful, but neither do they harm soundness.

(Moreover, `stable_deref_trait` requires that successive calls to `deref` return the same address;
while bytewise equality is the unambiguous definition of "same" for `usize`, extending the notion
of "same" to arbitrary lifetime-infected types in a useful way is more difficult.)

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
[`PointerViewKind`]: https://docs.rs/stable-view/0/stable_view/struct.PointerViewKind.html
[`RecursiveViewKind<(VT, VE)>`]: https://docs.rs/stable-view/0/stable_view/struct.RecursiveViewKind.html
[`UnstableViewKind`]: https://docs.rs/stable-view/0/stable_view/struct.UnstableViewKind.html
[`DefaultViewKind`]: https://docs.rs/stable-view/0/stable_view/struct.DefaultViewKind.html
[`SetDefaultView`]: https://docs.rs/stable-view/0/stable_view/trait.SetDefaultView.html
[`SetDefaultViewMut`]: https://docs.rs/stable-view/0/stable_view/trait.SetDefaultViewMut.html
[`Box::leak`]: https://doc.rust-lang.org/std/boxed/struct.Box.html#method.leak

[`variance-family`]: https://docs.rs/variance-family/0/variance_family
[`stable_deref_trait`]: https://docs.rs/stable_deref_trait/1.2.1/stable_deref_trait/index.html
[`aliasable`]: https://docs.rs/aliasable/0.1.3/aliasable/

[currently]: https://github.com/rust-lang/rfcs/pull/3712
[very unlikely to change]: https://github.com/rust-lang/rfcs/pull/3712#issuecomment-3715013712
