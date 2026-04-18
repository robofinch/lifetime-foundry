<div align="center" class="rustdoc-hidden">
<h1> Attached-Ref </h1>
</div>

[<img alt="github" src="https://img.shields.io/badge/github-attached--ref-08f?logo=github" height="20">](https://github.com/robofinch/lifetime-foundry/tree/main/crates/attached-ref)
[![Latest version](https://img.shields.io/crates/v/attached-ref.svg)](https://crates.io/crates/attached-ref)
[![Documentation](https://img.shields.io/docsrs/attached-ref)](https://docs.rs/attached-ref/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

# Overview

TODO: this is very out-of-date

The central `SelfRefBox` type is based on the following idea (setting aside finer details related to
lifetimes, traits, and soundness):

```ignore
struct SelfRefBox<T, S, E> {
    self_ref_slot: SelfRefSlot<S, E>,
    backing_data:  AliasableBox<T>,
}

enum SelfRefSlot<S, E> {
    None,
    /// Lifetime-erased self-reference produced from a `&T`
    Shared(S),
    /// Lifetime-erased self-reference produced from a `&mut T`
    Exclusive(E),
}
```

The type is designed for the self-reference to be frequently changed out and/or mutated, with
access to the backing data provided to the greatest extent that is sound.

If you need a self-referential struct where only shared/immutable references to the backing data
is required, I recommend considering [`yoke`].

# Limitations

To soundly allow self-references as described, some limitations are necessary.
- The self-references produced from a `&'a (mut) T` are required to be covariant in `'a`. In
  particular, they must not be able to mutate a `&'a (mut) T` reference or contain functions with
  `&'a (mut) T` arguments. Examples of permissible types include any `T` not using `'a` at all
  (a trivial self-reference not actually referencing the backing data), `&'a Foo`, `Cow<'a, Bar>`,
  `&'a &'a mut Baz`, `&'a Cell<Qux>`, and `[&'a Quux; N]`. Among types prohibited from being
  self-references are `Cell<&'a Foo>`, `&'a mut Cow<'a, Bar>`, and `fn(&'a Baz)`. See
  [the Rustonomicon's page on variance] for full details.

  This crate needs to ensure that a self-reference read multiple times with different lifetimes is
  always exposed to your code with a lifetime valid for that value. Upholding this requirement is
  not feasible for types that are invariant or contravariant in the relevant lifetime, except by
  using `unsafe` functions whose safety invariants place the burden on you to not misuse the
  returned values. This crate should reduce the scope and amount of `unsafe` you need to write,
  so invariant and contravariant types are simply prohibited.

- `SelfRefBox` is invariant over `T`, `S`, and `E`. Given that its self references can access
  `&mut T` rather than simply `&T`, invariance over `T` is inevitable with this design.
  Additionally, `SelfRefBox` uses fairly complicated unsafe trait bounds on `S` and `E`, so they
  are kept invariant out of an abundance of caution. (TODO: is them being covariant even an option?)
- Information about the layouts of `S` and `E` is necessary to lifetime-erase them, and the
  non-lifetime-erased self-references need to be proven to be convariant. Both requirements
  necessitate custom trait bounds.

Additionally, the backing data is stored in a box allocation, rather than an arbitrary type with
a [stable deref] impl that permits mutable aliasing; this is mostly for simplicity. Please reach
out on the Rust users forum or open an issue if this is inconvenient. For my part, I'm willing to
accept the slight cost of using the heap with the global allocator.

## Trade-offs compared to `yoke`
TODO (should probably wait until I've written the code)

## Trade-offs compared to `ouroboros`
TODO (should probably wait until I've written the code)

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

[`yoke`]: https://lib.rs/crates/yoke
[the Rustonomicon's page on variance]: https://doc.rust-lang.org/nomicon/subtyping.html
[stable deref]: https://docs.rs/stable_deref_trait/1/stable_deref_trait/trait.StableDeref.html
