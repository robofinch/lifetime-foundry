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

Sometimes, you need to require that a generic type is covariant or contravariant over one of its
parameters.

If you need to require invariance, you can make a wrapper type that uses `PhantomData` to force
invariance over a generic parameter in your own structs or enums. There is no similar easy solution
for covariance or contravariance.

This crate enables you to place requirements on a generic type's variance over a `'varying`
lifetime parameter while thoroughly supporting non-`'static` data.
(Type parameters cannot easily be supported in a useful way without `for<T>` binders.)

## Overview

Using `CovariantFamily`, `ContravariantFamily`, and `UnvaryingFamily`, you can require that a
family of types parameterized by a `'varying` lifetime be effectively covariant over `'varying`,
be effectively contravariant over `'varying`, or leave `'varying` entirely unused.

The parameterized type is called `T<'varying>`, which is an abbreviation for
`Varying<'varying, 'lower, Upper, T>`. (The `'lower` and `Upper` bounds are solely used to place
bounds on `'varying`, which allows the lifetime families to thoroughly support non-`'static` data.)

Requiring invariance is not supported, since, as mentioned, forcing invariance does not require a
fancy trait.

`LendFamily` is provided as a de-facto trait alias for a common sort of `CovariantFamily`.

See [the full README](crates/variance-family/README.md) for more.

## Stable View
[![Latest version](https://img.shields.io/crates/v/stable-view.svg)](https://crates.io/crates/stable-view)
[![Documentation](https://img.shields.io/docsrs/stable-view)](https://docs.rs/stable-view/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

## Overview

Get temporary "views" whose `'stable` data remains valid even when the views' source data is moved.

This crate provides `StableView`, `StableViewMut`, and `StableClone` traits
which prohibit certain actions, such as moving a value, from invalidating the temporary views
of values implementing the traits.

The traits are intended to be useful for self-referential structs; aliasable source data can be
soundly stored alongside values containing `'stable` references (obtained via a stable view trait)
to that source data, even when Rust's normal borrow checking and aliasing rules would ordinarily
make such a struct impossible or unsound to implement.

With this advanced `unsafe` target audience, it is quite difficult to either implement these traits
or directly use their methods and guarantees in `unsafe` code; ideally, you should not need to
directly interact with this crate, besides perhaps forwarding `V: StableView<..>` bounds (or
similar) to other libraries in generic code. (As the author, it's no longer difficult for me to
understand how this crate works, but I assume that it'd be difficult for others to learn.)

See [the full README](crates/stable-view/README.md) for more.
