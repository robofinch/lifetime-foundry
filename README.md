# Lifetime Foundry

This repository contains three crates which heavily focus on manipulating lifetimes and variance.

One major goal is the creation of a "`PowerYoke`"-like type; that is, a type similar to
`yoke::Yoke` but much more flexible. This project's `attached-ref` crate, in an approach similar to
[`yoke`](https://docs.rs/yoke/) or [`oyakodon`](https://github.com/sshockwave/oyakodon), provides
tools for creating self-referential structs via traits and generics rather than generating code
with a proc-macro as in [`ouroboros`](https://docs.rs/ouroboros/).

Additionally, `variance-family` is useful in its own right; see
[`contention-queue`](https://github.com/robofinch/contention-queue/) for an example use case.

## Attached-Ref

[![Latest version](https://img.shields.io/crates/v/attached-ref.svg)](https://crates.io/crates/attached-ref)
[![Documentation](https://img.shields.io/docsrs/attached-ref)](https://docs.rs/attached-ref/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

## Overview

This is an in-progress crate aiming to provide an extremely-configurable
`AttachableRefFull` type, in addition to simplified `AttachableRef`, `AttachableRefMut`,
`AttachedRef`, and `AttachedRefMut` transparent wrappers around configurations of
`AttachableRefFull`. (They are `repr(transparent)` wrappers rather than type aliases for the sake
of improving the readability of docs.)

`AttachedRef<'stable, S, Data>` should be a close equivalent to `yoke::Yoke<Y, C>` (with `Y`
being similar to `S` and `C` being similar to `Data`). In other words,
`AttachedRef` plays the role of the `PowerYoke<'static, Y, C>` idea [discussed several years ago](https://github.com/unicode-org/icu4x/issues/2926#issuecomment-1371393079).

Additionally, library code may wish to use this crate's types in a maximally-generic way, but
the distinction between `AttachableRefFull` and its `repr(transparent)` configurations would
greatly reduce the utility of the wrappers if the library only uses `AttachableRefFull`. That
problem would defeat the point of hiding complicated details from users via the `AttachedRef` type
and friends. As such, the development of this crate will attempt to include some sort of
`AttachableRefAlias` trait for abstracting over the configurations of `AttachableRefFull`.

See [the full README](crates/attached-ref/README.md) for more.

## Variance Family

[![Latest version](https://img.shields.io/crates/v/variance-family.svg)](https://crates.io/crates/variance-family)
[![Documentation](https://img.shields.io/docsrs/variance-family)](https://docs.rs/variance-family/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

## Motivation

Sometimes, a library needs to work with some generic lifetime-parameterized type. Historically,
that has meant supporting `&'varying T::Assoc` and `&'varying mut T::Assoc`, as with `Deref`
or `Borrow`. Generic associated types (GATs) provide another option: `T::Assoc<'varying>`.

Let's denote the `'varying`-parameterized type associated with `T` as `T<'varying>`. GATs are a
great implementation of the semantics of `T<'varying>` when:
- `'varying` can be *any* lifetime, whether arbitrarily short or `'static`, and
- you don't care *at all* what `T<'varying>` does with `'varying`. It could be `&'varying Foo`,
  `Cell<&'varying str>`, or `Box<dyn FnOnce(&'varying mut [u8])>`.
  In other words, the [variance] of `T<'varying>` over `'varying` could be anything.

Sometimes, however, a library wants to support `'varying` references to non-`'static` data, which
requires that the non-`'static` data outlives `'varying`. When working with GATs,
`for<'varying>` bounds usually end up allowing `'varying = 'static`, causing the code to fail to
compile. Additionally, a library might need to be able to shorten `T<'long>` to `T<'short>`.

I have written such libraries, and the compiler's support (and existing crate support) for these
cases is quite limited. Therefore, this crate exists to provide a custom implementation of
`T<'varying>`.

The traits here allow you to place lower and upper bounds on `'varying` (though those bounds can
also be effectively disabled), and if you wish, you can require that `T<'varying>` is covariant
(or contravariant) over `'varying`.

## Overview

Using [`CovariantFamily`], [`ContravariantFamily`], and [`UnvaryingFamily`], you can require that a
family of types parameterized by a `'varying` lifetime be effectively covariant over `'varying`,
be effectively contravariant over `'varying`, or leave `'varying` entirely unused.

[`LendFamily`] is provided as a de-facto trait alias for a common sort of [`CovariantFamily`].

The parameterized type is called `T<'varying>`, which is an abbreviation for
`Varying<'varying, 'lower, Upper, T>`. (Alternatively, you can think of `Varying<'_, 'lower, _, T>`
as an "implementation of" `T<'varying>`.)

The `'lower` and `Upper` bounds are solely used to place bounds on `'varying`, which allows the
lifetime families to thoroughly support non-`'static` data; they are not actually used in the
`T<'varying>` type.

See [the full README](crates/variance-family/README.md) for more.

## Stable View
[![Latest version](https://img.shields.io/crates/v/stable-view.svg)](https://crates.io/crates/stable-view)
[![Documentation](https://img.shields.io/docsrs/stable-view)](https://docs.rs/stable-view/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

## Overview

Get temporary "views" whose `'stable` data is suitable for self-references to the views' source
data in self-referential structs.

The primary user-facing interface of this crate is [`StableViewer`], [`StableViewerMut`], and a
system of "view kind" types that choose how to get a view. These types can be used entirely safely.
With the help of `attached-ref`, you can create a self-referential struct with source data stored
alongside `'stable` data, even when Rust's normal borrow checking and aliasing rules would
ordinarily make such a struct impossible or unsound to implement.

This crate also makes use of `variance-family` to provide substantial freedom over what the view
types are. Rather than a plain `&'stable Data::Target` reference, as with something based on the
`Deref` trait, the views of a `Data` type can be a nearly-arbitrary `'stable`-parameterized type,
such as:
- `Option<Cow<'stable, T>>`,
- `Vec<&'stable T>`,
- `&'stable &'data [u8]`, or
- [`StableViewer<'a, 'stable, 'data, Data>`] itself.

See [the full README](crates/stable-view/README.md) for more.


[variance]: https://doc.rust-lang.org/nomicon/subtyping.html#variance
[`CovariantFamily`]: https://docs.rs/variance-family/0/variance_family/trait.CovariantFamily.html
[`ContravariantFamily`]: https://docs.rs/variance-family/0/variance_family/trait.ContravariantFamily.html
[`UnvaryingFamily`]: https://docs.rs/variance-family/0/variance_family/trait.UnvaryingFamily.html
[`LendFamily`]: https://docs.rs/variance-family/0/variance_family/trait.LendFamily.html
[`StableViewer`]: https://docs.rs/stable-view/0/stable_view/struct.StableViewer.html
[`StableViewer<'a, 'stable, 'data, Data>`]: https://docs.rs/stable-view/0/stable_view/struct.StableViewer.html
[`StableViewerMut`]: https://docs.rs/stable-view/0/stable_view/struct.StableViewerMut.html
