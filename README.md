# Lifetime Foundry

This repository contains three crates which heavily focus on manipulating lifetimes and variance.

One major goal is the creation of a "power-yoke"-like type; that is, a type similar to
`yoke::Yoke` but much more flexible. [`yoke`](https://docs.rs/yoke/) and `attached-ref` provide
tools for creating self-referential structs via traits and generics rather than generating
code with a proc-macro as in [`ouroboros`](https://docs.rs/ouroboros/).

Additionally, `variance-family` is useful in its own right; see
[`contention-queue`](https://github.com/robofinch/contention-queue/) for an example use case.

## Attached-Ref

[![Latest version](https://img.shields.io/crates/v/attached-ref.svg)](https://crates.io/crates/attached-ref)
[![Documentation](https://img.shields.io/docsrs/attached-ref)](https://docs.rs/attached-ref/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

## Variance Family

[![Latest version](https://img.shields.io/crates/v/variance-family.svg)](https://crates.io/crates/variance-family)
[![Documentation](https://img.shields.io/docsrs/variance-family)](https://docs.rs/variance-family/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

## Stable View
[![Latest version](https://img.shields.io/crates/v/stable-view.svg)](https://crates.io/crates/stable-view)
[![Documentation](https://img.shields.io/docsrs/stable-view)](https://docs.rs/stable-view/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)
