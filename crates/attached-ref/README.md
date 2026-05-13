<div align="center" class="rustdoc-hidden">
<h1> Attached-Ref </h1>
</div>

[<img alt="github" src="https://img.shields.io/badge/github-attached--ref-08f?logo=github" height="20">](https://github.com/robofinch/lifetime-foundry/tree/main/crates/attached-ref)
[![Latest version](https://img.shields.io/crates/v/attached-ref.svg)](https://crates.io/crates/attached-ref)
[![Documentation](https://img.shields.io/docsrs/attached-ref)](https://docs.rs/attached-ref/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

# Overview

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

# Prior Art

## `yoke`

## `ouroboros`

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
