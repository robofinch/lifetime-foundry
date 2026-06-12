<div align="center" class="rustdoc-hidden">
<h1> Stable View </h1>
</div>

[<img alt="github" src="https://img.shields.io/badge/github-stable--view-08f?logo=github" height="20">](https://github.com/robofinch/lifetime-foundry/tree/main/crates/stable-view)
[![Latest version](https://img.shields.io/crates/v/stable-view.svg)](https://crates.io/crates/stable-view)
[![Documentation](https://img.shields.io/docsrs/stable-view)](https://docs.rs/stable-view/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

# Overview

Get temporary "views" whose `'stable` data is suitable for self-references to the views' source
data in self-referential structs.

The primary user-facing interface of this crate is [`StableViewer`], [`StableViewerMut`], and a
system of "view kind" types that choose how to get a view. These types can be used entirely safely.
With the help of [`attached-ref`], you can create a self-referential struct with source data stored
alongside `'stable` data, even when Rust's normal borrow checking and aliasing rules would
ordinarily make such a struct impossible or unsound to implement.

This crate also makes use of [`variance-family`] to provide substantial freedom over what the view
types are. Rather than a plain `&'stable Data::Target` reference, as with something based on the
`Deref` trait, the views of a `Data` type can be a nearly-arbitrary `'stable`-parameterized type,
such as:
- `Option<Cow<'stable, T>>`,
- `Vec<&'stable T>`,
- `&'stable &'data [u8]`, or
- [`StableViewer<'a, 'stable, 'data, Data>`] itself.

Note that [`StableViewer`] and [`StableViewerMut`] have three lifetime parameters:
- `'a`, a short-lived lifetime which cannot be used in self-references.
- `'stable`, a lifetime which might reference part of the source `Data`, but could be soundly used
  in a self-reference.
- `'data`, a long-lived lifetime. Data that lives for `'data` could be referenced, as in the
  `&'stable &'data [u8]` example, or a reference like `&'data T` could be shortened to `&'stable T`.

Technically, the views don't *need* to use `'stable` at all; for example, `&'a Data` and
`&'static str` are perfectly valid views of `Data`. These oddities might be useful when interacting
with generic library code like [`attached-ref`], even if the `'stable` lifetime is the primary
utility of this crate.

# Prior Art
The humble `Box<T>` can perhaps be considered prior art for getting a `&'stable T`.
***Do not be deceived.*** As of Rust 1.96, using `Box<T>` in a self-referential struct is very
likely to be unsound:
<https://doc.rust-lang.org/1.96.0/std/boxed/index.html#considerations-for-unsafe-code>

The [`stable_deref_trait`] crate also offers a trait which is widely used in self-referential
structs, but requiring that the reference returned by `Deref::deref` refers to the same address even
if the source value is moved does not seem necessary for the soundness of self-referential structs.
Nor is it *sufficient* for the soundness of self-referential structs; `&mut T` implements
`StableDeref`, which causes unsoundness in most `stable_deref_trait`-based self-referential struct
crates, including the popular [`yoke`] crate.

`stable-view` does not enforce that constraint, or any equivalent of it; views of a source `Data`
can be entirely different on each call.

The [`aliasable`] crate also provides aliasable versions of `&mut T` and `Box<T>`, but the version
on crates.io at the time of writing is unsound. Therefore, this crate implements its own versions
(complete with extensive documentation about soundness and aliasing guarantees).

After discussing unsoundness in prior approaches,
"now it is our turn to study statistical mechanics." (More seriously, though: I am very confident
in the soundness of this crate, though it's large enough that a typo might have slipped in
somewhere.)

# Architecture

In order to allow types to be viewed in multiple possible ways, and in order to allow custom
third-party views of other crates' types, this crate uses [`StableView`] and [`StableViewMut`]
traits which are not implemented directly for types like `Vec<T>` or `Rc<T>`. Instead, the source
data type is passed as a generic parameter, named `Data`, to the traits, and the traits are
implemented for a "view kind" type. The view kind is generally a zero-sized type whose sole purpose
is to indicate how to obtain a view.

For example:
- [`ReferenceViewKind`] implements `StableView<'_, '_, Rc<T>>` by returning a `&'stable T` to the
  contents of the `Rc<T>`.
- [`RecursiveViewKind<(ViewT, ViewE)>`] implements `StableView<'_, '_, Result<T, E>>` by viewing the
  inner `T` or `E` value with either `ViewT` or `ViewE`, respectively.
- [`UnstableViewKind`] type implements the stable view traits for all source data types in a trivial
  way, by not putting any `'stable` data in its provided views.
- [`DefaultViewKind`] implements `StableView<'_, '_, Data>` by deferring to whatever view kind
  chosen by `Data` as its default.

Note that the view kind types generally don't enforce any particular semantics on their stable view
trait implementations (beyond the traits' own requirements), whether for soundness or just
correctness; they are solely intended to be human-understandable categories. One-off dummy types
would work just as well for view kinds (aside from concerns about ergonomics or readability).

Still, the the following special cases are notable:
- [`DefaultViewKind`] receives preferential treatment (less boilerplate) in [`StableViewer::view`]
  and [`StableViewerMut::view_mut`]. Additionally, [`DefaultStableView`] and
  [`DefaultStableViewMut`] traits are implemented for `Data` types as sugar for a where-bound
  bounding `DefaultViewKind`.
- [`ReferenceViewKind`] receives preferential treatment (substantially less boilerplate) in
  `attachable-ref`. As with `DefaultViewKind`, [`StableReferenceView`] and
  [`StableReferenceViewMut`] traits are implemented for `Data` types as sugar, such that
  when `ReferenceViewKind` provides a `&'stable Pointee` view of `Data`, that `Pointee` type is
  available as `Data::Pointee` (in the presence of a `Data: StableReferenceView<'data>` bound).
- [`RecursiveViewKind`] can be implemented with a provided [`recursive_view!`] declarative macro.

# Unsafe Backbone

This crate's functionality is implemented primarily through [`StableView`], [`StableViewMut`], and
[`StableClone`] traits. These three traits prohibit certain actions, such as moving a value, from
invalidating the `'stable` data obtained via implementations of the traits. Since many people may
end up seeing these traits (and their associated types) even in entirely safe code, their
documentation is intended to have *somewhat* approachable summaries of their purpose and behavior.

However, the `unsafe` machinery enabling the safe interface is complicated. This crate even has a
`concepts_and_safety` module dedicated to hundreds of lines of documentation about unsafely
implementing `stable-view`'s traits. I have had months to gradually learn the machinery necessary to
implement `attached-ref`, `variance-family`, and `stable-view`; I consider `stable-view` to be the
most difficult. This is not entry-level `unsafe`.

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

[`StableViewer`]: https://docs.rs/stable-view/0/stable_view/struct.StableViewer.html
[`StableViewer<'a, 'stable, 'data, Data>`]: https://docs.rs/stable-view/0/stable_view/struct.StableViewer.html
[`StableViewer::view`]: https://docs.rs/stable-view/0/stable_view/struct.StableViewer.html#method.view
[`StableViewerMut`]: https://docs.rs/stable-view/0/stable_view/struct.StableViewerMut.html
[`StableViewerMut::view_mut`]: https://docs.rs/stable-view/0/stable_view/struct.StableViewerMut.html#method.view_mut

[`StableView`]: https://docs.rs/stable-view/0/stable_view/trait.StableView.html
[`StableViewMut`]: https://docs.rs/stable-view/0/stable_view/trait.StableViewMut.html
[`StableClone`]: https://docs.rs/stable-view/0/stable_view/trait.StableClone.html

[`ReferenceViewKind`]: https://docs.rs/stable-view/0/stable_view/struct.ReferenceViewKind.html
[`StableReferenceView`]: https://docs.rs/stable-view/0/stable_view/trait.StableReferenceView.html
[`StableReferenceViewMut`]: https://docs.rs/stable-view/0/stable_view/trait.StableReferenceViewMut.html

[`RecursiveViewKind`]: https://docs.rs/stable-view/0/stable_view/struct.RecursiveViewKind.html
[`RecursiveViewKind<(ViewT, ViewE)>`]: https://docs.rs/stable-view/0/stable_view/struct.RecursiveViewKind.html
[`recursive_view!`]: https://docs.rs/stable-view/0/stable_view/macro.recursive_view.html

[`UnstableViewKind`]: https://docs.rs/stable-view/0/stable_view/struct.UnstableViewKind.html

[`DefaultViewKind`]: https://docs.rs/stable-view/0/stable_view/struct.DefaultViewKind.html
[`DefaultStableView`]: https://docs.rs/stable-view/0/stable_view/trait.DefaultStableView.html
[`DefaultStableViewMut`]: https://docs.rs/stable-view/0/stable_view/trait.DefaultStableViewMut.html

[`variance-family`]: https://docs.rs/variance-family/0/variance_family
[`attached-ref`]: https://docs.rs/attached-ref/0/attached_ref

[`stable_deref_trait`]: https://docs.rs/stable_deref_trait/1.2.1/stable_deref_trait/index.html
[`aliasable`]: https://docs.rs/aliasable/0.1.3/aliasable/
[`yoke`]: https://docs.rs/yoke/0.8/yoke/

[currently]: https://github.com/rust-lang/rfcs/pull/3712
[very unlikely to change]: https://github.com/rust-lang/rfcs/pull/3712#issuecomment-3715013712
