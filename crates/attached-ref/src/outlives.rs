//! Types for expressing implied bounds between lifetimes, with added invariance to mitigate a
//! compiler bug.
//!
//! They are essentially just sugar for the correct [`PhantomData`].

use core::marker::PhantomData;


/// Expresses that `'data: 'stable` and that `'stable: 'a`.
///
/// # Compiler bug workaround
///
/// Some callbacks used by this crate (such as the one passed to `with_mut`) genuinely need
/// [`OutlivesChain`] to provide `'data: 'stable` and `'stable: 'a` implied bounds.
///
/// Other callbacks, however, are provided a [`OutlivesChain<'data, 'stable, 'a>`] even if they
/// *already* have a [`StableViewer<'data, 'stable, 'a>`], which has the same implied bounds.
///
/// The latter callbacks take `OutlivesChain` in order to reduce the impact of unsoundness in the
/// compiler.
///
/// Those familiar with Rust may be aware that the bounds on various `F` callbacks in this crate
/// sure look a lot like the weird function types which cause **unsoundness** due to bugs in the
/// compiler; see, in particular,
/// [this comment](https://github.com/rust-lang/rust/issues/25860#issuecomment-1175760163) on the
/// "Implied bounds on nested references + variance = soundness hole" issue.
///
/// [`OutlivesChain`] is invariant over its three lifetimes, which removes the "variance"
/// part of that equation. Note that the bounds *do* use implied bounds on nested references.
/// If you build a similar higher-ranked abstraction, be careful.
///
/// Once the unsoundness is fixed in some Rust 1.X version and this crate's MSRV exceeds 1.X,
/// the `OutlivesChain` arguments in such callbacks will be removed (while it will remain in the
/// callbacks which actually need to implied bounds).
///
/// [`StableViewer<'data, 'stable, 'a>`]: stable_view::StableViewer
#[derive(Debug, Default, Clone, Copy)]
#[repr(transparent)]
pub struct OutlivesChain<'data, 'stable, 'a>(PhantomData<fn(*mut &'a &'stable &'data ())>);

impl<'data, 'stable, 'a> OutlivesChain<'data, 'stable, 'a>
where
    'data:   'stable,
    'stable: 'a,
{
    /// Expresses that `'data` outlives `'stable` and that `'stable` outlives `'a`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

/// Expresses that `'data` outlives `'stable`.
///
/// See also [`OutlivesChain`], which also details the significance of the invariance of this type's
/// two lifetime parameters.
#[derive(Debug, Default, Clone, Copy)]
#[repr(transparent)]
pub struct Outlives<'data, 'stable>(PhantomData<fn(*mut &'stable &'data ())>);

impl<'data, 'stable> Outlives<'data, 'stable>
where
    'data: 'stable,
{
    /// Expresses that `'data` outlives `'stable`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}
