//! The traits which are the central purpose of this crate.

#![expect(unsafe_code, reason = "allow unsafe code to rely on impls of lifetime family traits")]

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

/// Apply a `'varying` lifetime to a family of types, and provide implied bounds that
/// bound `'varying` between `'lower` and the lifetime of an `Upper` bound (which is `&'upper ()`
/// for some `'upper`).
///
/// ## Lifetimes
///
/// The trait should be implemented for as many values of `'lower` and `Upper` as possible.
///
/// Additionally, for a given fixed `T`, `T::Is` must not use `'lower` or `Upper` (such
/// that a lifetime family `T<..>` can be described as `T<'varying>`, and the `'lower` and `Upper`
/// bounds truly just place bounds on `'varying` without doing anything else). This constraint
/// is unsafely enforced by [`ChangeBounds`].
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
/// ## Aliases
///
/// Note that `<T as WithLifetime<'varying, 'lower, Upper>>::Is` is also available as a
/// [`Varying<'varying, 'lower, Upper, T>`] alias (which is 13 characters shorter, and perhaps
/// easier to read and write).
///
/// Additionally, [`MaxUpperBound`] is provided as an alias for `&'static ()`.
///
/// # Safety
/// ## Usage of `'lower` and `Upper`
/// This `Is` associated type must not use `'lower` or `Upper`.
///
/// This holds if and only if `<Self as WithLifetime<'varying, 'lower, Upper>>::Is` and
/// `<Self as WithLifetime<'varying, 'other_lower, OtherUpper>>::Is` are the **same exact** type
/// for any `'varying`, `'lower`, `Upper`, `'other_lower`, and `OtherUpper` such that:
/// ```ignore
/// Self: WithLifetime<'varying, 'lower, Upper> + WithLifetime<'varying, 'other_lower, OtherUpper>
/// ```
///
/// This condition should be checked via the [`ChangeBounds`] supertrait. This safety condition
/// is included just in case some way of evading that supertrait bound is found.
///
/// ## External Implementations
/// This trait may only be implemented for types local to the crate containing the impl, with the
/// special  exception of `variance-family` being permitted to implement this trait for types in
/// `core`, `alloc`, and `std`.
///
/// This condition is *mostly* redundant with the usual orphan rules, with the exception of
/// `#[fundamental]` types (currently: `&T`, `&mut T`, `Box<T>`, and `Pin<T>`). Some of the
/// `unsafe` code in `variance-family` relies on reasoning about *all* implementations of
/// `WithLifetime` for those `#[fundamental]` types, which safe code could hypothetically render
/// unsound if not for this trait being `unsafe`. In the event that another crate uses
/// `#[fundamental]` types, it would likewise be permitted to assume that other crates do not
/// implement `WithLifetime` for its types, even if the orphan rules would not prevent that.
pub unsafe trait WithLifetime<
    'varying, 'lower, Upper: UpperBound,
    __ImplyBound = &'lower &'varying Upper,
>: ChangeBounds<'varying, 'lower, Upper, Self::Is> {
    type Is: ?Sized;
}

/// Enforce that `'lower` and `Upper` are solely used to constrain the `'varying` lifetime and
/// are not actually used in the `Is` associated type.
///
/// # Safety
/// `RawMutVarying<'varying, Self>` must be the same type as `*mut *mut V`.
///
/// An implementation is certainly sound if [`ChangeBounds::prove_equal`] is implemented with the
/// function body `{ varying }`. Otherwise, you will need to thoroughly confirm that
/// your lifetime family's type genuinely only uses `'varying` and not `'lower` or `Upper`, and you
/// can implement the method with `{ varying.cast() }`.
pub unsafe trait ChangeBounds<'varying, 'lower, Upper, V: ?Sized> {
    fn prove_equal<'other_lower, OtherUpper>(
        varying: RawMutVarying<'varying, 'other_lower, OtherUpper, Self>,
    ) -> *mut *mut V
    where
        Self: WithLifetime<'varying, 'other_lower, OtherUpper>,
        OtherUpper: UpperBound;
}

/// A slightly shorter and more legible alias for
/// `<T as WithLifetime<'varying, 'lower, Upper>>::Is`.
pub type Varying<'varying, 'lower, Upper, T> = <T as WithLifetime<'varying, 'lower, Upper>>::Is;

/// A type which can only coerce to a different type `U` via subtyping coercions.
///
/// Since this type is covarariant over `T<'varying>`, attempting to coerce
/// `RawVarying<'short, '_, _, T>` from or into `RawVarying<'long, '_, _, T>` where `'long: 'short`
/// can indicate whether `T<'varying>` is covariant or contravariant over `'varying`.
pub type RawVarying<'varying, 'lower, Upper, T> = *const *const Varying<'varying, 'lower, Upper, T>;

/// A type which cannot coerce to any other type and is invariant over `T<'varying>`.
pub type RawMutVarying<'varying, 'lower, Upper, T> = *mut *mut Varying<'varying, 'lower, Upper, T>;

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
/// for some `'upper`), the type `Varying<'varying, Self>` is simply equal to
/// `Self::WithAnyLifetime`.
///
/// All possible implementations of this trait are provided indirectly, based on [`WithLifetime`].
/// See the [`unvarying`] macro or [`variance_family::Unvarying`] type to get implementations of
/// this trait.
///
/// # Note on Lower Bound
/// [`MaxUpperBound`], which is an alias for `&'static ()`, is a maximally loose upper bound on
/// `'varying`. However, there is no (and *cannot* be any) special lifetime that can be substituted
/// into `'lower` to serve as a lower bound for all other lifetimes. Instead,
/// `for<'lower> UnvaryingFamily<'lower, Upper>` uses a maximally loose lower bound (and implied
/// bounds ensure that this works regardless of what `Upper` is).
///
/// [`variance_family::Unvarying`]: crate::Unvarying
/// [`unvarying`]: crate::unvarying
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
/// See the [`covariant`] and [`unvarying`] macros to implement this trait in simple cases.
/// The below documentation contains details which you might not need to know.
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
/// covariant.
///
/// # Note on Bounds
/// `'lower` and `Upper` allow for bounds on `'varying` to be expressed via implied bounds, which
/// may be necessary for implementations to satisfy well-formedness constraints. For instance,
/// the `&'varying &'a T` covariant family must have `'varying` be at most `'a`, and the
/// `&'a &'varying T` covariant family must have `'varying` be at least `'a`. However, these bounds
/// do *not* precisely constrain the range in which covariant casts are permitted; they are
/// intended solely for well-formedness constraints.
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
/// [`variance_family::shorten`] and the compiler's covariant coercion.
///
/// Additionally, performing covariant casts on the `'varying` lifetime through `unsafe` means
/// (such as [`transmute`]) is permitted.
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
/// If that fails, try to use [`covariant`] (or [`unvarying`]). If your use case involves generics
/// which are treated as variance families, that macro will not be sufficient, so you will need
/// to unsafely implement this trait (and its two unsafe supertraits).
///
/// # Implementation Safety
/// `'varying` must be sound to cast covariantly in `T<'varying>` (where `T<'varying>` is
/// shorthand for `Varying<'varying, T>`.
///
/// More precisely, for any `'short, 'long, 'lower, 'other_lower, 'upper, 'other_upper` where:
/// ```rust
/// 'long: 'short,
/// Self: WithLifetime<'short, 'lower, 'upper> + WithLifetime<'long, 'other_lower, 'other_upper>,
/// ```
/// it must be sound to cast (possibly via [`transmute`])
/// `<Self as WithLifetime<'long, 'other_lower, 'other_upper>>::Is` to
/// `<Self as WithLifetime<'short, 'other_lower, 'other_upper>>::Is`
/// in covariant positions, and vice-versa in contravariant positions.
///
/// That is, **the `'lower` and `Upper` bounds of this trait do not constrain the range in which
/// covariant coercion is permitted**.
///
/// If [`CovariantFamily::prove_covariance`] can be implemented with the function body `{ long }`,
/// then the implementation is certainly sound. Otherwise, you will need to thoroughly confirm that
/// your lifetime family is covariant over `'varying`, and you can implement the method with
/// `{ long.cast() }`.
///
/// [`transmute`]: core::mem::transmute
/// [`covariant`]: crate::covariant
/// [`unvarying`]: crate::unvarying
/// [`variance_family::shorten`]: crate::shorten
pub unsafe trait CovariantFamily<'lower, Upper: UpperBound>: LifetimeFamily<'lower, Upper> {
    /// Method to help show that `Self<'varying>` is covariant over `'varying` in simpler cases.
    ///
    /// If this method can be implemented with the body `{ long }`, then the compiler recognizes
    /// this lifetime family as covariant over `'varying`, so the implementation of this trait
    /// is sound.
    ///
    /// Otherwise, you **must** thoroughly ensure that values of type `Varying<'varying, Self>`
    /// can soundly have covariant casts of `'varying` performed on them, and this function can
    /// be implemented with `{ long.cast() }`.
    fn prove_covariance<'long, 'short>(
        long: RawVarying<'long, 'lower, Upper, Self>,
    ) -> RawVarying<'short, 'lower, Upper, Self>
    where
        Upper: 'long,
        'long: 'short,
        'short: 'lower;
}

/// A "lifetime family" of types parameterized by a `'varying` lifetime such that performing
/// contravariant casts on the `'varying` lifetime is sound.
///
/// Note that "being able to be contravariantly casted" is a slightly broader condition than
/// "being contravariant". See the Examples section. In documentation throughout this crate,
/// "contravariance" may actually refer to "the ability to soundly be contravariantly casted"
/// instead of the variance assigned by the compiler.
///
/// See the [`contravariant`] and [`unvarying`] macros to implement this trait in simple cases.
/// The below documentation contains details which you might not need to know.
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
/// [`variance_family::lengthen`] and the compiler's contravariant coercion.
///
/// Additionally, performing contravariant casts on the `'varying` lifetime through `unsafe` means
/// (such as [`transmute`]) is permitted.
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
/// If that fails, try to use [`contravariant`] (or [`unvarying`]). If your use case involves
/// generics which are treated as variance families, that macro will not be sufficient, so you will
/// need to unsafely implement this trait (and its two unsafe supertraits).
///
/// # Implementation Safety
/// `'varying` must be sound to cast contravariantly in `T<'varying>` (where `T<'varying>` is
/// shorthand for `Varying<'varying, T>`, and `'varying` is bounded by `'lower` and the lifetime
/// of `Upper`).
///
/// More precisely, for any `'short, 'long, 'lower, 'other_lower, 'upper, 'other_upper` where:
/// ```rust
/// 'long: 'short,
/// Self: WithLifetime<'short, 'lower, 'upper> + WithLifetime<'long, 'other_lower, 'other_upper>,
/// ```
/// it must be sound to cast (possibly via [`transmute`])
/// `<Self as WithLifetime<'long, 'other_lower, 'other_upper>>::Is` to
/// `<Self as WithLifetime<'short, 'other_lower, 'other_upper>>::Is`
/// in covariant positions, and vice-versa in contravariant positions.
///
/// That is, **the `'lower` and `Upper` bounds of this trait do not constrain the range in which
/// covariant coercion is permitted**.
///
/// If [`ContravariantFamily::prove_contravariance`] can be implemented with the function body
/// `{ short }`, then the implementation is certainly sound. Otherwise, you will need to thoroughly
/// confirm that your lifetime family is contravariant over `'varying`, and you can implement the
/// method with `{ short.cast() }`.
///
/// [`transmute`]: core::mem::transmute
/// [`contravariant`]: crate::contravariant
/// [`unvarying`]: crate::unvarying
/// [`variance_family::lengthen`]: crate::lengthen
pub unsafe trait ContravariantFamily<'lower, Upper: UpperBound>: LifetimeFamily<'lower, Upper> {
    /// Method to help show that `Self<'varying>` is contravariant over `'varying` in simpler cases.
    ///
    /// If this method can be implemented with the body `{ short }`, then the compiler recognizes
    /// this lifetime family as contravariant over `'varying`, so the implementation of this trait
    /// is sound.
    ///
    /// Otherwise, you **must** thoroughly ensure that values of type `Varying<'varying, Self>`
    /// can soundly have contravariant casts of `'varying` performed on them, and this function can
    /// be implemented with `{ short.cast() }`.
    fn prove_contravariance<'short, 'long>(
        short: RawVarying<'short, 'lower, Upper, Self>,
    ) -> RawVarying<'long, 'lower, Upper, Self>
    where
        Upper: 'long,
        'long: 'short,
        'short: 'lower;
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
/// `Upper` is always `&'upper ()` for some lifetime `'upper`, and it defaults to [`MaxUpperBound`]
/// (an alias for `&'static ()`), which is a maximally loose upper bound on `'varying`.
pub trait LendFamily<Upper = MaxUpperBound>
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

/// A slightly shorter and more legible alias for
/// `<T as WithLifetime<'varying, 'varying, Upper>>::Is`.
///
/// This is intended to be used with [`LendFamily`], which places no lower bound on `'varying`.
pub type Lend<'varying, Upper, T> = <T as WithLifetime<'varying, 'varying, Upper>>::Is;
