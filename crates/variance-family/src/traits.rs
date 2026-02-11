// ================================================================
//  Support
// ================================================================

/// A private trait for sealing `UpperBound`.
trait SealedUpperBound {
    /// Ensure that `UpperBound` is not `dyn`-compatible in order to head off any concerns about
    /// interactions between higher-ranked `dyn` trait objects and implied bounds.
    #[expect(dead_code, reason = "removes `dyn`-compatibility without requiring `Sized`")]
    fn remove_dyn_compatibility() {}
}

/// A possible upper bound for a `'varying` lifetime.
/// Guaranteed to be `&'upper ()` for some `'upper`.
#[expect(private_bounds, reason = "intentionally creating a sealed trait")]
pub trait UpperBound: SealedUpperBound {}

impl SealedUpperBound for &() {}
impl UpperBound for &() {}

pub type MaxUpperBound = &'static ();

/// A private trait for sealing `ImplyBound`.
trait Sealed {
    /// Ensure that `ImplyBound` is not `dyn`-compatible in order to head off any concerns about
    /// interactions between higher-ranked `dyn` trait objects and implied bounds.
    #[expect(dead_code, reason = "removes `dyn`-compatibility without requiring `Sized`")]
    fn remove_dyn_compatibility() {}
}

/// Provide implied bounds for a `'varying` lifetime, bounding it between
/// a lower lifetime `'lower` and the lifetime of an `Upper` upper bound (which is `&'upper ()`
/// for some `'upper`).
#[expect(private_bounds, reason = "intentionally creating a sealed trait")]
pub trait ImplyBound: Sealed {}

impl<'varying, Upper: UpperBound> Sealed for (&'_ &'varying (), &'varying Upper) {}
impl<'varying, Upper: UpperBound> ImplyBound for (&'_ &'varying (), &'varying Upper) {}

/// An uninhabited type for sealing a `WithLifetime` method.
enum PrivateSeal {}

/// Apply a `'varying` lifetime to a family of types, and provide implied bounds that
/// bound `'varying` between `'lower` and the lifetime of an `Upper` bound (which is `&'upper ()`
/// for some `'upper`).
///
/// ## Lifetimes
///
/// The trait should be implemented for as many values of `'lower` and `Upper` as possible. In
/// particular, even if an implementation does not need a nontrivial upper bound on `'varying`, do
/// not solely implement the trait for the `'static` upper bound (unless `'static` is the only
/// possible lifetime valid for `'varying`).
///
/// Preserving maximum flexibility in lifetimes and upper bounds is important, as implementing
/// `for<'varying, 'any> WithLifetime<'varying, 'any, &'static ()>` does not automatically imply
/// implementations of `WithLifetime` for any other combinations of lifetimes and upper bounds,
/// even though, semantically, we can reason that
/// `for<'varying, 'any> WithLifetime<'varying, 'any, &'static ()>` applies maximally loose lower
/// and upper bounds on `'varying` and should allow for arbitrary upper bounds.
///
/// ## Why not a GAT
///
/// This trait is very similar to a generic associated type (GAT):
/// ```
/// # trait UpperBound {}
/// pub trait LifetimeFamily<'lower, Upper: UpperBound> {
///     type WithLifetime<'varying>: ?Sized
///     where
///         Upper: 'varying,
///         'varying: 'lower;
/// }
/// ```
///
/// However, `for<'varying> <T as LifetimeFamily<'lower, Upper>>::WithLifetime<'varying>: ..Bounds`
/// would not work very well; the `for<'varying>` binder may still attempt to quantify over
/// lifetimes shorter than `'lower` and which outlive `Upper`. For some reason, as of Rust 1.90.0,
/// the `for<'varying> ..: ..Bounds` bound would still compile. However, any attempts to *use*
/// whatever has that bound would fail with an opaque "higher-ranked lifetime error".
///
/// In short, `for<'varying> ..` bounds do not work even remotely well with a GAT, greatly
/// limiting any nontrivial uses of a GAT-based `LifetimeFamily`.
///
/// With this trait's use of implied bounds,
/// `for<'varying> <T as WithLifetime<'varying, 'lower, Upper>>::Is: ..Bounds` quantifies only
/// over `'varying` lifetimes between `'lower` and all lifetimes in `Upper`.
///
/// ## Alias
///
/// Note that `<T as WithLifetime<'varying, 'lower, Upper>>::Is` is also available as a
/// [`Varying<'varying, 'lower, Upper, T>`] alias (which is 13 characters shorter, and perhaps
/// easier to read and write).
///
/// Additionally, [`MaxUpperBound`] is provided as an alias for `&'static ()`.
pub trait WithLifetime<
    'varying, 'lower, Upper: UpperBound,
    __ImplyBound: ImplyBound = (&'lower &'varying (), &'varying Upper),
> {
    type Is: ?Sized;

    /// Ensure that `WithLifetime` and `LifetimeFamily` are not `dyn`-compatible in order
    /// to head off any concerns about interactions between higher-ranked `dyn` trait objects
    /// and implied bounds.
    #[doc(hidden)]
    #[expect(
        private_interfaces,
        reason = "seal this method; nobody should bother implementing it",
    )]
    fn remove_dyn_compatibility(_seal: PrivateSeal) {}
}

/// A slightly shorter and more legible alias for
/// `<T as WithLifetime<'varying, 'lower, Upper>>::Is`.
pub type Varying<'varying, 'lower, Upper, T> = <T as WithLifetime<'varying, 'lower, Upper>>::Is;

// ================================================================
//  Lifetime Family traits
// ================================================================

/// A family of types which are parameterized by a `'varying` lifetime.
///
/// In order to support non-`'static` references interacting with `'varying` in complicated ways
/// (which may require lifetime constraints for well-formedness), lower and upper bounds are placed
/// on the possible lifetimes that `'varying` may be.
///
/// Note that this trait is effectively a trait alias for
/// `for<'varying> WithLifetime<'varying, 'lower, Upper>`; all possible implementations of this
/// trait are provided, and you should implement [`WithLifetime`] for your types.
///
/// See [`WithLifetime`] for more information.
pub trait LifetimeFamily<'lower, Upper>
where
    Upper: UpperBound,
    Self: for<'varying> WithLifetime<'varying, 'lower, Upper>,
{}

impl<'lower, Upper, T> LifetimeFamily<'lower, Upper> for T
where
    Upper: UpperBound,
    T: ?Sized + for<'varying> WithLifetime<'varying, 'lower, Upper>,
{}

/// A trivial "lifetime family" of types parameterized by a `'varying` lifetime which don't
/// actually use the `'varying` parameter.
///
/// For any `'varying` lifetime between `'lower` and the lifetime of `Upper` (which is `&'upper ()`
/// for some `'upper`), the type `Varying<'varying, 'lower, Upper, Self>` is simply equal to
/// `Self::WithAnyLifetime`.
///
/// All possible implementations of this trait are already provided.
///
/// # Note on Lower Bound
/// [`MaxUpperBound`], which is an alias for `&'static ()`, is a maximally loose upper bound on
/// `'varying`. However, there is no (and *cannot* be any) special lifetime that can be substituted
/// into `'lower` to serve as a lower bound for all other lifetimes. Instead,
/// `for<'lower> UnvaryingFamily<'lower, Upper>` uses a maximally loose lower bound (and implied
/// bounds ensure that this works regardless of what `Upper` is).
pub trait UnvaryingFamily<'lower, Upper: UpperBound>:
    LifetimeFamily<'lower, Upper>
        + for<'varying> WithLifetime<'varying, 'lower, Upper, Is = Self::WithAnyLifetime>
{
    type WithAnyLifetime: ?Sized;
}

impl<'lower, Upper, T, U> UnvaryingFamily<'lower, Upper> for T
where
    Upper: UpperBound,
    T: ?Sized
        + LifetimeFamily<'lower, Upper>
        + for<'varying> WithLifetime<'varying, 'lower, Upper, Is = U>,
    U: ?Sized,
{
    type WithAnyLifetime = U;
}

/// A "lifetime family" of types parameterized by a `'varying` lifetime such that performing
/// covariant casts on the `'varying` lifetime is sound.
///
/// Note that "being able to be covariantly casted" is a slightly broader condition than
/// "being covariant (as far as the compiler is concerned)". See the Examples section. In
/// documentation throughout this crate, "covariance" may actually refer to
/// "the ability to soundly be covariantly casted" instead of the variance assigned by the compiler.
///
/// # Note on Bounds
/// `'lower` and `Upper` allow for bounds on `'varying` to be expressed via implied bounds, which
/// may be necessary for implementations to satisfy well-formedness constraints. For instance,
/// the `&'varying &'a T` covariant family must have `'varying` be at most `'a`, and the
/// `&'a &'varying T` covariant family must have `'varying` be at least `'a`.
///
/// Note that `Upper` is always `&'upper ()` for some lifetime `'upper`.
///
/// [`MaxUpperBound`], which is an alias for `&'static ()`, is a maximally loose upper bound on
/// `'varying`. However, there is no (and *cannot* be any) special lifetime that can be substituted
/// into `'lower` to serve as a lower bound for all other lifetimes. Instead,
/// `for<'lower> CovariantFamily<'lower, Upper>` uses a maximally loose lower bound (and implied
/// bounds ensure that this works regardless of what `Upper` is).
///
/// As covariant lifetimes are usually freely shrinkable (such as `&'varying mut [u8]`) with
/// only unusual exceptions (such as `&'a &'varying u8`, which requires `'varying: 'a`), common
/// use cases will likely require `for<'lower> CovariantFamily<'lower, Upper>` bounds; such a
/// bound is available more succinctly via [`LendFamily`].
///
/// # Safety of Use
/// Code can always use safe methods to change the `'varying` lifetime, including
/// [`CovariantFamily::shorten`], [`CovariantFamily::shorten_ref`], and the compiler's
/// covariant coercion.
///
/// However, before performing any covariant casts on the `'varying` lifetime through `unsafe`
/// means (such as [`transmute`]), the [`CovariantFamily::covariant_assertions`] must be called
/// and not panic. The other two methods are not guaranteed to internally call
/// `covariant_assertions`.
///
/// # Implementation
///
/// **You should probably not need to directly and unsafely implement this trait.**
///
/// The `variance-family` crate includes a large number of `unsafe` implementations of the marker
/// traits for generic types for the sake of ergonomics for users -- in particular, for the sake
/// of limiting how many times that others must unsafely implement the marker traits. When that
/// does not suffice, there are also many helper macros.
///
/// You should first try to express your desired lifetime as a composition of other lifetime
/// families, such as `(Cow<'a, str>, &'varying mut [u8], MyStruct)` becoming
/// `(Cow<'a, str>, VaryingRefMut<[u8]>, Unvarying<MyStruct>)`.
///
// TODO: describe next steps.
///
/// # Implementation Safety
///
/// The following three conditions must be met.
///
/// - If [`CovariantFamily::covariant_assertions`] does not panic, then `'varying` must be sound
///   to cast covariantly in `T<'varying>` (where `T<'varying>` is shorthand for
///   `Varying<'varying, 'lower, Upper, T>`, and `'varying` is bounded by `'lower` and the
///   lifetime of `Upper`).
///
/// - No assertions not included within `covariant_assertions` may be used.
///
/// - The implementation safety requirements of `shorten` and `shorten_ref` must be met.
///
/// ## Precise Elaboration
/// For any implementation of this type, it must be sound to cast the `'varying` lifetime of
/// `Varying<'varying, 'lower, Upper, T>` to any shorter lifetime which is at least as long as
/// `'lower`.
///
/// Compile-time assertions (possibly resulting in post-monomorphization errors) may be placed
/// in [`CovariantFamily::covariant_assertions`], which serve as additional preconditions for the
/// family of types being covariant. Runtime assertions could also be included there, though their
/// utility would seem questionable.
///
/// Provided that `covariant_assertions` does not panic, covariant casts on `'varying` may be
/// performed via [`transmute`] or similar means, not necessarily via the
/// [`CovariantFamily::shorten`] and [`CovariantFamily::shorten_ref`] methods.
/// `shorten` and `shorten_ref` are provided in part for ergonomics and in part to help confirm
/// that an implementation of this trait is sound.
///
/// # Examples
///
/// If the compiler considers the lifetime family to be covariant over `'varying`, then this trait
/// can be soundly implemented. For instance, `&'a &'varying str`, `&'varying &'a str`, and
/// `fn(&'a fn(&'varying str))` can soundly implement this trait with appropriate `'lower` and
/// `Upper` bounds.
///
/// If `'varying` is entirely unused in the lifetime family, meaning that the "family" consists of
/// a single type, this trait can be soundly implemented. Examples include `u8`, `[u8]`, and
/// `&'a [u8]`.
///
/// Additionally, the family might have some non-covariant variance over `'varying` assigned by the
/// compiler, but it may still be sound to implement this trait. A type might, for instance, gate
/// any parts of its interface that would normally rely on contravariance or invariance behind
/// `unsafe` functions with safety comments properly ensuring that a type can be treated as
/// covariant. The below is a more trivial example where the type does not actually rely on
/// contravariance or invariance whatsoever.
///
/// ```
/// # use variance_family::{CovariantFamily, LifetimeFamily, WithLifetime, UpperBound, Varying};
/// # use core::marker::PhantomData;
/// /// # Variance
/// /// Even though `'a` is invariant, covariant casts on `'a` are provided.
/// /// Users should not rely on this type's invariance over `'a`.
/// struct CouldBeCovariant<'a>(&'a str, PhantomData<fn(&'a ()) -> &'a ()>);
/// struct CouldBeCovFamily;
///
/// impl<'varying, Upper: UpperBound> WithLifetime<'varying, '_, Upper> for CouldBeCovFamily {
///     type Is = CouldBeCovariant<'varying>;
/// }
///
/// /// # Safety
/// /// `CouldBeCovariant<'varying>` can be treated as covariant over `'varying`; the invariance of
/// /// `'varying` is utterly unimportant for safety. Semantically, it varies the same as
/// /// `&'varying str`.
/// unsafe impl<'lower, Upper: UpperBound> CovariantFamily<'lower, Upper> for CouldBeCovFamily {
///     /// Performs no assertions.
///     #[inline]
///     fn covariant_assertions() {}
///
///     #[inline]
///     fn shorten<'l, 's>(
///         long: Varying<'l, 'lower, Upper, Self>,
///     ) -> Varying<'s, 'lower, Upper, Self>
///     where
///         Upper: 'l,
///         'l: 's,
///         's: 'lower,
///         for<'varying> Varying<'varying, 'lower, Upper, Self>: Sized,
///     {
///         CouldBeCovariant(long.0, PhantomData)
///     }
///
///     #[inline]
///     fn shorten_ref<'l, 's, 'r>(
///         long: &'r Varying<'l, 'lower, Upper, Self>,
///     ) -> &'r Varying<'s, 'lower, Upper, Self>
///     where
///         Upper: 'l,
///         'l: 's,
///         's: 'lower,
///         Varying<'l, 'lower, Upper, Self>: 'r,
///         Varying<'s, 'lower, Upper, Self>: 'r,
///     {
///         let long: &'r CouldBeCovariant<'l> = long;
///         // SAFETY: this shortens the lifetime of the pointee. Shortening `&'l str` to
///         // `&'s str` is sound, since that's covariant; meanwhile, `PhantomData` is a ZST that
///         // attaches no semantic meaning to its type parameter. Additionally, `CouldBeCovariant`
///         // doesn't *actually* use the invariance of its lifetime for anything important.
///         // Moreover, to avoid the hypothetical situation where someone may use
///         // `CouldBeCovariant` to cause invariance of `'l` and rely on that invariance for
///         // correct semantics (or maybe even soundness) in some way that would be broken by
///         // this cast, `CouldBeCovariant` documents that its lifetime parameter is
///         // treated as covariant. It's on authors of other `unsafe` code to read its docs.
///         let transmuted: &'r CouldBeCovariant<'s> = unsafe { core::mem::transmute(long) };
///         transmuted
///     }
/// }
/// ```
///
/// [`transmute`]: core::mem::transmute
pub unsafe trait CovariantFamily<'lower, Upper: UpperBound>: LifetimeFamily<'lower, Upper> {
    /// Perform compile-time assertions, which may cause post-monomorphization errors.
    ///
    /// (The function could, hypothetically, also include runtime assertions.)
    // TODO: `const` block example
    #[inline]
    fn covariant_assertions() {}

    /// Soundly shorten the `'varying` lifetime of an owned `Self::WithLifetime<'varying>`.
    ///
    /// # Implementation Safety
    /// It is always sound to implement this function with the body `{ long }`,
    /// relying on implicit covariant coercion (if possible).
    ///
    /// The function's body **MUST** be equivalent to:
    /// ```
    /// # struct Foo;
    /// # impl Foo {
    /// #     fn subset_of_assertions_in_covariant_assertions() {}
    /// #     fn shorten(long: u8) -> u8
    /// {
    ///     // Usually just `Self::covariant_assertions();`
    ///     Self::subset_of_assertions_in_covariant_assertions();
    ///     // SAFETY: ..
    ///     unsafe { ::core::mem::transmute(long) }
    /// }
    /// # }
    /// ```
    ///
    /// Any assertions (or other possible causes of panics) in `Self::shorten` must be included in
    /// `Self::covariant_assertions()`.
    #[expect(clippy::unnecessary_safety_doc, reason = "False positive; it's only for implementors")]
    #[must_use]
    fn shorten<'l, 's>(
        long: Varying<'l, 'lower, Upper, Self>,
    ) -> Varying<'s, 'lower, Upper, Self>
    where
        Upper: 'l,
        'l: 's,
        's: 'lower,
        for<'varying> Varying<'varying, 'lower, Upper, Self>: Sized;

    /// Soundly shorten the `'varying` lifetime of `&Self::WithLifetime<'varying>`.
    ///
    /// # Implementation Safety
    /// It is always sound to implement this function with the body `{ long }`,
    /// relying on implicit covariant coercion (if possible).
    ///
    /// The function's body **MUST** be equivalent to:
    /// ```
    /// # struct Foo;
    /// # impl Foo {
    /// #     fn subset_of_assertions_in_covariant_assertions() {}
    /// #     fn shorten(long: u8) -> u8
    /// {
    ///     // Usually just `Self::covariant_assertions();`
    ///     Self::subset_of_assertions_in_covariant_assertions();
    ///     // SAFETY: ..
    ///     unsafe { ::core::mem::transmute(long) }
    /// }
    /// # }
    /// ```
    ///
    /// Any assertions (or other possible causes of panics) in `Self::shorten` must be included in
    /// `Self::covariant_assertions()`.
    #[expect(clippy::unnecessary_safety_doc, reason = "False positive; it's only for implementors")]
    #[must_use]
    fn shorten_ref<'l, 's, 'r>(
        long: &'r Varying<'l, 'lower, Upper, Self>,
    ) -> &'r Varying<'s, 'lower, Upper, Self>
    where
        Upper: 'l,
        'l: 's,
        's: 'lower,
        Varying<'l, 'lower, Upper, Self>: 'r,
        Varying<'s, 'lower, Upper, Self>: 'r;
}

/// A "lifetime family" of types parameterized by a `'varying` lifetime such that performing
/// contravariant casts on the `'varying` lifetime is sound.
///
/// Note that "being able to be contravariantly casted" is a slightly broader condition than
/// "being contravariant". See the Examples section. In documentation throughout this crate,
/// "contravariance" may actually refer to "the ability to soundly be contravariantly casted"
/// instead of the variance assigned by the compiler.
///
/// # Note on Bounds
/// `'lower` and `Upper` allow for bounds on `'varying` to be expressed via implied bounds, which
/// may be necessary for implementations to satisfy well-formedness constraints.
///
/// Note that `Upper` is always `&'upper ()` for some lifetime `'upper`.
///
/// [`MaxUpperBound`], which is an alias for `&'static ()`, is a maximally loose upper bound on
/// `'varying`. However, there is no (and *cannot* be any) special lifetime that can be substituted
/// into `'lower` to serve as a lower bound for all other lifetimes. Instead,
/// `for<'lower> ContravariantFamily<'lower, Upper>` uses a maximally loose lower bound (and implied
/// bounds ensure that this works regardless of what `Upper` is).
///
/// # Safety of Use
/// Code can always use safe methods to change the `'varying` lifetime, including
/// [`ContravariantFamily::lengthen`], [`ContravariantFamily::lengthen_ref`], and the compiler's
/// contravariant coercion.
///
/// However, before performing any contravariant casts on the `'varying` lifetime through `unsafe`
/// means (such as [`transmute`]), the [`ContravariantFamily::contravariant_assertions`] method
/// must be called and not panic. The other two methods are not guaranteed to internally call
/// `contravariant_assertions`.
///
/// # Implementation
///
/// **You should probably not need to directly and unsafely implement this trait.**
///
/// The `variance-family` crate includes a large number of `unsafe` implementations of the marker
/// traits for the sake of ergonomics for users -- in particular, for the sake
/// of limiting how many times that others must unsafely implement the marker traits. When that
/// does not suffice, there are also many helper macros.
///
/// You should first try to express your desired lifetime as a composition of other lifetime
/// families, such as `(Cow<'a, str>, fn(&'varying mut [u8]) -> MyStruct)` becoming
/// `(Cow<'a, str>, fn(VaryingRefMut<[u8]>) -> Unvarying<MyStruct>)`.
///
// TODO: describe next steps.
///
/// # Implementation Safety
/// The following three conditions must be met.
///
/// - If [`ContravariantFamily::contravariant_assertions`] does not panic, then `'varying` must be
///   sound to cast contravariantly in `T<'varying>` (where `T<'varying>` is shorthand for
///   `Varying<'varying, 'lower, Upper, T>`, and `'varying` is bounded by `'lower` and `Upper`).
///
/// - No assertions not included within `contravariant_assertions` may be used.
///
/// - The implementation safety requirements of `lengthen` and `lengthen_ref` must be met.
///
/// ## Precise Elaboration
/// For any implementation of this type, it must be sound to cast the `'varying` lifetime of
/// `Varying<'varying, 'lower, Upper, T>` to any longer lifetime which is at most as long as
/// the lifetime of `Upper`.
///
/// Compile-time assertions (possibly resulting in post-monomorphization errors) may be placed
/// in [`ContravariantFamily::contravariant_assertions`], which serve as additional preconditions
/// for the family of types being contravariant. Runtime assertions could also be included there,
/// though their utility would seem questionable.
///
/// Provided that `contravariant_assertions` does not panic, contravariant casts on `'varying` may
/// be performed via [`transmute`] or similar means, not necessarily via the
/// [`ContravariantFamily::lengthen`], [`ContravariantFamily::lengthen_ref`] methods.
/// `lengthen` and `lengthen_ref` are provided in part for ergonomics and in part to help confirm
/// that an implementation of this trait is sound.
///
/// ## Examples
///
/// If the compiler considers the lifetime family to be contravarint over `'varying`, then this
/// trait can be soundly implemented. For instance, `fn(&'a &'varying str)` and
/// `fn(&'varying &'a str)` can soundly implement this trait, with appropriate `'lower` and
/// `Upper` bounds.
///
/// If `'varying` is entirely unused in the lifetime family, meaning that the "family" consists of
/// a single type, this trait can be soundly implemented. Examples include `u8`, `[u8]`, and
/// `&'a [u8]`.
///
/// Additionally, the family might have some non-contravariant variance over `'varying` assigned by
/// the compiler, but it may still be sound to implement this trait. A type might, for instance,
/// gate any parts of its interface that would normally rely on covariance or invariance behind
/// `unsafe` functions with safety comments properly ensuring that a type can be treated as
/// contravariant.
///
/// [`transmute`]: core::mem::transmute
pub unsafe trait ContravariantFamily<'lower, Upper: UpperBound>: LifetimeFamily<'lower, Upper> {
    /// Perform compile-time assertions, which may cause post-monomorphization errors.
    ///
    /// (The function could, hypothetically, also include runtime assertions.)
    // TODO: `const` block example
    #[inline]
    fn contravariant_assertions() {}

    /// Soundly lengthen the `'varying` lifetime of an owned `Self::WithLifetime<'varying>`.
    ///
    /// # Implementation Safety
    /// It is always sound to implement this function with the body `{ short }`,
    /// relying on implicit contravariant coercion (if possible).
    ///
    /// The function's body **MUST** be equivalent to:
    /// ```
    /// # struct Foo;
    /// # impl Foo {
    /// #     fn subset_of_assertions_in_contravariant_assertions() {}
    /// #     fn lengthen(short: u8) -> u8
    /// {
    ///     // Usually just `Self::contravariant_assertions();`
    ///     Self::subset_of_assertions_in_contravariant_assertions();
    ///     // SAFETY: ..
    ///     unsafe { ::core::mem::transmute(short) }
    /// }
    /// # }
    /// ```
    ///
    /// Any assertions (or other possible causes of panics) in `Self::lengthen` must be included in
    /// `Self::contravariant_assertions()`.
    #[expect(clippy::unnecessary_safety_doc, reason = "False positive; it's only for implementors")]
    #[must_use]
    fn lengthen<'s, 'l>(
        short: Varying<'s, 'lower, Upper, Self>,
    ) -> Varying<'l, 'lower, Upper, Self>
    where
        Upper: 'l,
        'l: 's,
        's: 'lower,
        for<'varying> Varying<'varying, 'lower, Upper, Self>: Sized;

    /// Soundly lengthen the `'varying` lifetime of `&Self::WithLifetime<'varying>`.
    ///
    /// # Implementation Safety
    /// It is always sound to implement this function with the body `{ short }`,
    /// relying on implicit contravariant coercion (if possible).
    ///
    /// The function's body **MUST** be equivalent to:
    /// ```
    /// # struct Foo;
    /// # impl Foo {
    /// #     fn subset_of_assertions_in_contravariant_assertions() {}
    /// #     fn lengthen(short: u8) -> u8
    /// {
    ///     // Usually just `Self::contravariant_assertions();`
    ///     Self::subset_of_assertions_in_contravariant_assertions();
    ///     // SAFETY: ..
    ///     unsafe { ::core::mem::transmute(short) }
    /// }
    /// # }
    /// ```
    ///
    /// Any assertions (or other possible causes of panics) in `Self::lengthen` must be included in
    /// `Self::contravariant_assertions()`.
    #[expect(clippy::unnecessary_safety_doc, reason = "False positive; it's only for implementors")]
    #[must_use]
    fn lengthen_ref<'s, 'l, 'r>(
        short: &'r Varying<'s, 'lower, Upper, Self>,
    ) -> &'r Varying<'l, 'lower, Upper, Self>
    where
        Upper: 'l,
        'l: 's,
        's: 'lower,
        Varying<'s, 'lower, Upper, Self>: 'r,
        Varying<'l, 'lower, Upper, Self>: 'r;
}

/// A `LendFamily` is a family of `Sized` types which are parameterized by a `'varying` lifetime
/// parameter which can be arbitrarily shortened via covariant casts.
///
/// All possible implementations of this trait are already provided.
///
/// # Note on Bounds
/// `Upper` allows for an upper bound on `'varying` to be expressed via implied bounds, which
/// may be necessary for implementations to satisfy well-formedness constraints. For instance,
/// a `&'varying &'a T` lend family must have `'varying` be at most `'a`.
///
/// `Upper` is always `&'upper ()` for some lifetime `'upper`, and [`MaxUpperBound`], which is an
/// alias for `&'static ()`, is a maximally loose upper bound on `'varying`.
pub trait LendFamily<Upper>
where
    Upper: UpperBound,
    Self: for<'lower> CovariantFamily<'lower, Upper>
        + for<'varying, 'lower> WithLifetime<'varying, 'lower, Upper, Is: Sized>,
{}

impl<Upper, T> LendFamily<Upper> for T
where
    Upper: UpperBound,
    T: for<'lower> CovariantFamily<'lower, Upper>
        + for<'varying, 'lower> WithLifetime<'varying, 'lower, Upper, Is: Sized>,
{}
