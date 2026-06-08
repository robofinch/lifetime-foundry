<div align="center" class="rustdoc-hidden">
<h1> Variance Family </h1>
</div>

[<img alt="github" src="https://img.shields.io/badge/github-variance--family-08f?logo=github" height="20">](https://github.com/robofinch/lifetime-foundry/tree/main/crates/variance-family)
[![Latest version](https://img.shields.io/crates/v/variance-family.svg)](https://crates.io/crates/variance-family)
[![Documentation](https://img.shields.io/docsrs/variance-family)](https://docs.rs/variance-family/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

# Motivation
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

(I have not actually needed to use the lower bound or contravariance in non-contrived cases.
If you ever do, I'd be interested in hearing about it!)

# Overview

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

### Definition of Variance
The qualifier "effectively" on variance is added in above descriptions of `CovariantFamily` and
`ContravariantFamily` to emphasize that this crate deals with the *soundness* of changing a
`'varying` lifetime, not with the variance assigned by the compiler. One may assume that
"covariant" or "contravariant" (in the context of Rust) refers to the variance assigned by the
compiler; however, some of the documentation throughout this crate uses a slightly broader notion.
Whenever "covariant" (or "contravariant") is genuinely intended to mean "the variance assigned by
the compiler is covariance (or contravariance)", that will be explicitly stated.

Essentially, `CovariantFamily` and `ContravariantFamily` require that the *implementor* of the
trait can prove that `'varying` can be soundly cast in a covariant or contravariant way, not that
the *compiler* can prove covariance or contravariance over `'varying`. A type parameterized by
`'varying` might be considered invariant over `'varying` by the compiler but carefully ensure that
the type's interface remains sound when `'varying` is changed covariantly; such a type can soundly
implement `CovariantFamily`.

### Safe Usage

You can use [`shorten`], [`lengthen`], [`shorten_lend`], or the compiler's provided coercions to
change the `'varying` lifetime. Additionally, [`change_bounds_from`] and
[`change_bounds_into`] are provided to change the `'lower` and `Upper` bounds
of `Varying<'varying, 'lower, Upper, T>`.

### Guarantees for `unsafe` Code

If a family of types implements [`CovariantFamily`] (or [`ContravariantFamily`]), then its
`'varying` lifetime may be soundly manipulated in a covariant (or contravariant) way via
`transmute` and similar means. Additionally, a `Varying<'varying, 'lower, Upper, T>` type
must not actually use `'lower` or `Upper`, such that the lower and upper bounds can be freely
changed via `transmute` and similar means.

### `UnvaryingFamily`

[`UnvaryingFamily`] ensures with the type system that implementors do not use the `'varying`
lifetime whatsoever. Therefore, if you bound a generic `T<'varying>` lifetime family by
`UnvaryingFamily`, it is extremely likely that the compiler will let you freely coerce `T<'v1>`
into `T<'v2>` regardless of what the two lifetimes are (since it will realize that they're the same
type). If the compiler fails to allow that conversion, then `transmute` and similar means can
soundly transmute `T<'v1>` into `T<'v2>` even in an invariant position, such as `*mut T<'varying>`.

### Custom Implementations

These traits are not implemented exhaustively. In particular, third-party structs and enums
cannot be covered by `CovariantFamily` and `ContravariantFamily` implementations here, and
exhaustively implementing traits for types in `std` is not a goal of this crate. Instead,
when the lifetime families provided here are not sufficient, you can create a custom lifetime
family over whatever types you wish (including types not defined in the same crate as the custom
lifetime family).

Various `macro_rules!` macros are provided by this crate to simplify implementation.
`variance-family` internally uses those macros for *all* of its `unsafe impl`s, implying that the
macros should be sufficient for most implementations.

# Non-`'static` Data

The lifetime family traits do not require that the family be parameterized by all possible lifetimes
(including `'static` or arbitrarily short lifetimes), which would pose an issue for lifetime
families like `&'varying &'a str` and `&'a &'varying str`; in such cases, `'varying` being either
`'static` or arbitrarily short could be invalid.

Each lifetime family comes with `'lower` and `Upper` bounds on how `'varying` is allowed to vary.
Those bounds are enforced through implied bounds, causing
```text
for<'varying> Varying<'varying, 'lower, Upper, T>
```
to behave like
```text
for<'varying where Upper: 'varying, 'varying: 'lower> Varying<'varying, 'lower, Upper, T>
```

Note that a consumer of a lifetime family may sometimes wish to require that the family has *no*
upper bound or lower bound. The type `&'static ()` (available as a [`MaxUpperBound`] alias) acts as
a maximally loose upper bound, but there is no special lifetime shorter than all other lifetimes;
instead, `for<'lower> CovariantFamily<'lower, Upper>` effectively removes the lower bound.
(This, too, automagically acts like it had a `for<'lower where Upper: 'lower>` binder.)

# Caution for Implied Bounds

Most of the complexity of this crate consists of reasoning about variance in ways which are
certainly sound. However, its usage of implied bounds do edge closer to compiler bugs and similar
edge cases. For the below reasons, I consider this crate to be fairly low-risk
(with respect to compiler bugs).

Currently, in some situations involving higher-ranked function pointers, the compiler can neglect
to enforce implied bounds, resulting in soundness. This known bug is tracked at
<https://github.com/rust-lang/rust/issues/25860>. Higher-ranked `dyn` trait objects for impossible
traits can also be created, as mentioned in <https://github.com/rust-lang/rust/issues/84533>.
This crate does not implement any of its traits for higher-ranked types, and none of its
traits are `dyn`-compatible; therefore, this crate itself should not come close to triggering
the unsoundness from those bugs. Nevertheless, for the sake of caution, it is worth keeping this
potential risk in mind when working with higher-ranked types alongside this crate.

Additionally, while `WithLifetime` uses an "interesting" supertrait bound, it does not fit the form
of the example in <https://github.com/rust-lang/rust/issues/136547> which creates an unconstrained
lifetime. (Either way, it does not appear that this issue can, alone, result in unsoundness.)

[`change_bounds_from`] and [`change_bounds_into`] have complicated bounds in order to evade
<https://github.com/rust-lang/rust/issues/21974>, but the bounds are not pathological.

Many lifetime-related soundness bugs appear to result from unsound function types. (Currently,
the builtin impls of some closures and functions are not well-formed, resulting in unsoundness.)
This crate does not contain any usage of the `Fn`, `FnMut`, or `FnOnce` traits and does not even
use any closures.

# Limitations

## Ergonomics

Support is not provided for requiring invariance over `'varying`; you can always force invariance
over a generic parameter with `PhantomData`. Additionally, support is only provided for a single
`'varying` lifetime parameter, but you can chain together lifetime families to achieve arbitrarily
many varying lifetime parameters.

## Type Parameters

However, varying *type* parameters are not supported. Even if traits for providing a representation
of `T<U>` and requiring that `T<U>` is covariant (or contravariant) over `U` were created, the
result would likely not be very useful without `for<U>` type binders, or some other way to represent
bounds like `for<U: Send + Sync> T<U>: Send + Sync`. (I have tried.)

## Lower and Upper Bounds

It was mentioned above that the `'lower` and `Upper` parameters of
`Varying<'varying, 'lower, Upper, T>` are solely used to place bounds on `'varying`, and they are
not actually used in the `T<'varying>` type. The compiler is not aware of that fact, and thinks that
they *could* be used in `T<'varying>`; this constraint is, unfortunately, enforced through `unsafe`
traits rather than through safe type system tricks.

I only settled on this `unsafe` trait approach forbidding `'lower` and `Upper` from appearing in
`T<'varying>` after around four unsuccessful attempts to enforce `Upper: 'varying` and
`'varying: 'lower` as implied bounds, without allowing `T<'varying>` to use `'lower` and `Upper`.
I have come to the conclusion that achieving the `Upper: 'varying` and `'varying: 'lower` implied
bounds is only possible if `T<'varying>` actually has access to `'lower` and `Upper`, in which case
there's no way to prevent safe code from stuffing those parameters into the associated type;
an `unsafe` requirement is the only option, and the compiler does not know about our made-up
safety conditions.

This, then, results in the nightmarish bounds used by [`change_bounds_from`] and
[`change_bounds_into`], and I have come to be *strongly* annoyed by
<https://github.com/rust-lang/rust/issues/21974> in much of my code, notably including
[`attached-ref`].

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

[`CovariantFamily`]: https://docs.rs/variance-family/0/variance_family/trait.CovariantFamily.html
[`ContravariantFamily`]: https://docs.rs/variance-family/0/variance_family/trait.ContravariantFamily.html
[`UnvaryingFamily`]: https://docs.rs/variance-family/0/variance_family/trait.UnvaryingFamily.html
[`LendFamily`]: https://docs.rs/variance-family/0/variance_family/trait.LendFamily.html
[`shorten`]: https://docs.rs/variance-family/0/variance_family/fn.shorten.html
[`lengthen`]: https://docs.rs/variance-family/0/variance_family/fn.lengthen.html
[`shorten_lend`]: https://docs.rs/variance-family/0/variance_family/fn.shorten_lend.html
[`change_bounds_from`]: https://docs.rs/variance-family/0/variance_family/fn.change_bounds_from.html
[`change_bounds_into`]: https://docs.rs/variance-family/0/variance_family/fn.change_bounds_into.html
[`MaxUpperBound`]: https://docs.rs/variance-family/0/variance_family/type.MaxUpperBound.html
[variance]: https://doc.rust-lang.org/nomicon/subtyping.html#variance
