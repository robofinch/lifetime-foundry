use core::marker::PhantomData;

use stable_view::{CustomView, CustomViewMut, StableView, StableViewMut};
use variance_family::{Lend, LendFamily};


/// A trait which explicitly states the bounds needed by many callbacks in this crate.
///
/// # Purpose
///
/// This allows manual implementations of "closures" to be used, in place of actual closures.
/// Actual closures *do* also implement this trait, so falling back to an explicit implementation
/// should not usually be necessary.
///
/// However, in versions of Rust before 1.94.0, the combination of higher-rank trait bounds,
/// implied bounds, and associated type projections seem to give the compiler a headache; some
/// callsites using closures in pre-1.94.0 versions might fail to compile with a somewhat
/// nonsensical lifetime error.
///
/// As the v0.x and v1.x versions of this crate are intended to support Rust 1.85, this trait is
/// included as a polyfill. (Plus, it could conceivably be useful in Rust 1.94 and above.)
///
/// # Dummy `PhantomData` Argument
/// The `FnOnce` implementation of this trait takes a dummy `PhantomData` value as a second
/// argument, in addition to the view. This bound is primarily to allow `'stable` to be
/// mentioned in its return type, and secondly to make the `'data: 'stable` and `'stable: 'a`
/// implied bounds slightly more explicit.
///
/// The input `CustomView<..>` alias might refer to a type which does not use `'stable` at all. If
/// none of the inputs to the `FnOnce` trait mention `'stable`, the return type cannot mention
/// `'stable`. (Without the `PhantomData` parameter mentioning `'stable`, the closures's
/// signature would not be valid.)
pub trait ViewToLend<'data, 'upper, Data, View, Dest>
where
    'upper: 'data,
    Data:   ?Sized,
    // Ranges over `'a` such that `'data: 'a`.
    //
    // **Critically**, there is no `Data: 'a` or `V: 'a` implied bound (or similar)
    // restricting `'a`, only a `'data: 'a` implied bound.
    View:   for<'a> StableView<'a, 'data, Data>,
    // There are internally two binders, `for<'lower> and `for<'varying>`, that ranges over
    // `'lower` and `'varying` such that `'upper: 'varying` and `'varying: 'lower`.
    //
    // **Critically**, there should be no other implied bounds. It might look risky that `Dest::Is`
    // appears as a trait input to `ChangeBounds`, but that's an associated type that should be
    // required to be WF when the trait inputs to `WithLifetime` are WF.
    // (Stepping back from abstract reasoning, I have indeed gotten compiler errors when `Dest::Is`
    // isn't well-formed.)
    Dest:   LendFamily<&'upper ()>,
{
    /// A higher-order `FnOnce` closure-like function.
    ///
    /// Intended to be simpler for the compiler than the full higher-rank `FnOnce` trait bound.
    ///
    /// # Robust Guarantee about Bounds
    /// When `Self: ViewToLend<'data, 'upper, Data, View, Dest>` and `Self`, `Data`, `View`,
    /// and `Dest` are well-formed types, we have that `'a` and `'stable` range over all lifetimes
    /// such that `'data: 'stable` and `'stable: a`.
    ///
    /// `unsafe` code can rely on this guarantee.
    ///
    /// (This is a guarantee made by the author of this trait, by reasoning based on the trait
    /// bounds, *not* a guarantee `unsafe`ly made by implementors. If you manage to break this
    /// trait, please reach out! That would mean there's a bug in either this trait or the
    /// compiler.)
    ///
    /// ## Reasoning
    /// When the input parameters of a trait (including `Self`) are well-formed (and the trait
    /// bound holds), the associated types of that trait must also be well-formed.
    ///
    /// The type of `view`, namely `CustomView<'a, 'stable, 'data, Data, View>`, expands to:
    ///
    /// ```ignore
    /// <
    ///     <
    ///         View as StableView<'a, 'data, Data, &'a &'data ()>
    ///     >::View
    ///     as
    ///     WithLifetime<'stable, 'a, &'data (), &'a &'stable &'data ()>
    /// >::Is
    /// ```
    ///
    /// We can see that the input parameters are `'a`, `'stable`, `'data`, `Data`, `View`,
    /// `&'data ()`, `&'a &'data ()`, and `&'a &'stable &'data ()`.
    ///
    /// In addition to the generic type parameters that we already assume are well-formed, the
    /// additional parameters are well-formed when `'data: 'stable` and `'stable: a`. The type of
    /// `self` is also assumed well-formed.
    ///
    /// Even the return type, `Lend<'stable, &'upper (), Dest>`, is well-formed
    /// in that scenario; its expanded input parameters are `'stable`, `'upper`, `Dest`,
    /// `&'upper ()`, and `&'stable &'stable &'upper ()`. We have a where-bound that
    /// `'upper: 'data`, so when `'data: 'stable`, the type `&'stable &'stable &'upper ()` is
    /// also well-formed.
    ///
    /// Thus, when `'data: 'stable` and `'stable: a` (in addition to where-bounds and our stated
    /// WF assumptions), we have that all input parameters, argument types, and even the return type
    /// of the function item
    /// `<Self as ViewToLend<'data, 'upper, Data, View, Dest>>::view_to_lend::<'a, 'stable>`
    /// are well-formed.
    ///
    /// The where-bounds of this function item do not further restrict `'a` and `'stable`, and
    /// implementors are not permitted to strengthen those bounds.
    ///
    /// Therefore, there should be no way for somebody to implement this trait in a way such that
    /// `'a` and `'stable` *don't* range over all lifetimes such that `'data: 'stable` and
    /// `'stable: a`.
    #[must_use]
    fn view_to_lend<'a, 'stable>(
        self,
        view: CustomView<'a, 'stable, 'data, Data, View>,
    ) -> Lend<'stable, &'upper (), Dest>
    where
        'data:   'stable,
        'stable: 'a;
}

impl<'data, 'upper, Data, View, F, Dest> ViewToLend<'data, 'upper, Data, View, Dest> for F
where
    'upper: 'data,
    Data:   ?Sized,
    View:   for<'a> StableView<'a, 'data, Data>,
    F:      for<'a, 'stable> FnOnce(
                CustomView<'a, 'stable, 'data, Data, View>,
                PhantomData<&'a &'stable &'data ()>,
            ) -> Lend<'stable, &'upper (), Dest>,
    Dest:   LendFamily<&'upper ()>,
{
    #[inline]
    fn view_to_lend<'a, 'stable>(
        self,
        view: CustomView<'a, 'stable, 'data, Data, View>,
    ) -> Lend<'stable, &'upper (), Dest>
    where
        'data:   'stable,
        'stable: 'a
    {
        self(view, PhantomData)
    }
}

/// A trait which explicitly states the bounds needed by many callbacks in this crate.
///
/// This allows manual implementations of "closures" to be used, in place of actual closures.
/// Actual closures *do* also implement this trait, so falling back to an explicit implementation
/// should not usually be necessary.
///
/// See [`ViewToLend`] for more, including about the dummy `PhantomData` second argument
/// in the implementation of this trait for closures.
pub trait ViewMutToLend<'data, 'upper, Data, ViewMut, Dest>
where
    'upper:  'data,
    Data:    ?Sized,
    // Ranges over `'a` such that `'data: 'a`.
    ViewMut: for<'a> StableViewMut<'a, 'data, Data>,
    Dest:    LendFamily<&'upper ()>,
{
    /// A higher-order `FnOnce` closure-like function.
    ///
    /// Intended to be simpler for the compiler than the full higher-rank `FnOnce` trait bound.
    ///
    /// # Robust Guarantee about Bounds
    /// The same guarantee as the one in [`ViewToLend::view_to_lend`] holds of `view_mut_to_lend`.
    #[must_use]
    fn view_mut_to_lend<'a, 'stable>(
        self,
        view_mut: CustomViewMut<'a, 'stable, 'data, Data, ViewMut>,
    ) -> Lend<'stable, &'upper (), Dest>
    where
        'data:   'stable,
        'stable: 'a;
}

impl<'data, 'upper, Data, ViewMut, F, Dest> ViewMutToLend<'data, 'upper, Data, ViewMut, Dest>
for F
where
    'upper:     'data,
    Data:       ?Sized,
    ViewMut:    for<'a> StableViewMut<'a, 'data, Data>,
    F:          for<'a, 'stable> FnOnce(
                    CustomViewMut<'a, 'stable, 'data, Data, ViewMut>,
                    PhantomData<&'a &'stable &'data ()>,
                ) -> Lend<'stable, &'upper (), Dest>,
    Dest:       LendFamily<&'upper ()>,
{
    #[inline]
    fn view_mut_to_lend<'a, 'stable>(
        self,
        view_mut: CustomViewMut<'a, 'stable, 'data, Data, ViewMut>,
    ) -> Lend<'stable, &'upper (), Dest>
    where
        'data:   'stable,
        'stable: 'a
    {
        self(view_mut, PhantomData)
    }
}
