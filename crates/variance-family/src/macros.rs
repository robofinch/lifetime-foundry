//! [`covariant`], [`contravariant`], and [`unvarying`] macros that cover simple cases.
//!
//! Additionally, a [`phantom_zst_methods`] macro for phantom variance family markers.

#![expect(
    unsafe_code,
    reason = "create marker function for triggering `unsafe_code` lints for users",
)]

/// Implement a simple covariant family.
///
/// The family must be proven covariant by the compiler, and it must be well-formed for *any*
/// `'varying` lifetime (that is, no custom `'lower` or `Upper` bounds are permitted).
///
/// # Examples
/// ```rust
/// # use core::marker::PhantomData;
/// # use variance_family::covariant;
/// struct Foo<'a, 'b, 'c, T: ?Sized>(&'a str, &'b usize, &'c T);
///
/// // This usage of `fn` makes `VaryingFoo` be `Send + Sync` while having the same variance
/// // over `'c` and `T` as `Foo`.
/// struct VaryingFoo<'c, T: ?Sized>(PhantomData<fn() -> &'c T>);
///
/// covariant! {
///     // The varying lifetime must come first (and is mandatory).
///     // (You can name it something else, `'varying` is just a convention.)
///     // `CovariantFamily<'_, _>` is simply a magic string you must include.
///     // (This is intended to mimic real Rust syntax, of course.)
///     impl<'varying, {'other, Generics}> CovariantFamily<'_, _>
///     // (See below docs for why this `unsafe` is necessary.)
///     // SAFETY: `VaryingFoo` is defined in this crate.
///     for #[unsafe(not_a_foreign_fundamental_type)] VaryingFoo<'other, Generics>
///     // The type given here is the `'varying`-parameterized type.
///     as Foo<'varying, 'varying, 'other, Generics>
///     // If `where` is included, then at least one of the following two types of where-bounds
///     // must be included. If both are included, they must be in this order.
///     where {
///         // We could have done `impl<'varying, {'other, Generics: ?Sized}>` instead.
///         Generics: ?Sized,
///     } and for<'varying> {
///         // A silly example, of course. The point being that you can have where-bounds
///         // which mention `'varying`, though they need to be listed separately.
///         // Each bound should be in its own set of braces, so that `for<'varying>` can be
///         // applied to each.
///         {&'varying (): Copy},
///     }
/// }
/// ```
///
/// Simpler example:
/// ```rust
/// # use variance_family::covariant;
/// struct Bar<'a>(&'a u8);
///
/// struct VaryingBar;
///
/// covariant! {
///     // The `, {..}` is optional. (Note that for simplicity of implementation, a trailing
///     // comma is not permitted after the varying lifetime in this case.)
///     impl<'varying> CovariantFamily<'_, _>
///     // SAFETY: `VaryingBar` is defined in this crate.
///     for #[unsafe(not_a_foreign_fundamental_type)] VaryingBar
///     as Bar<'varying>
/// }
/// ```
///
/// # Safety
/// One safety condition for implementing [`WithLifetime`] requires that it *not* be implemented
/// for family types not defined in your crate (with the exception of `variance-family`
/// implementing its traits for types in `core`, `alloc`, and `std`).
///
/// Thanks to the orphan rules, this rule is mostly irrelevant; it only matters for
/// `#[fundamental]` types.
///
/// If the family type used with this macro (the type following `for`, **not** the varying type
/// following `as`) is a `#[fundamental]` type which already implements a variance family trait,
/// then you *should* get a compilation error.
/// (This includes all current stable `#[fundamental]` types.)
///
/// However, for a `#[fundamental]` type were to not already implement a variance family trait,
/// you could violate the safety condition of [`WithLifetime`] with this macro. *Hypothetically*,
/// someone (possibly including the crate which defined the `#[fundamental]` type) could rely on
/// "this type cannot soundly implement [`WithLifetime`]" and find a way to escalate that to
/// Undefined Behavior.
///
/// Therefore, to be thoroughly sound, this macro requires a
/// `#[unsafe(not_a_foreign_fundamental_type)]` annotation on the family type.
///
/// (Though, that annotation is slightly inaccurate for internal usage of this macro in
/// `variance-family`.)
///
/// [`WithLifetime`]: crate::traits::WithLifetime
#[macro_export]
macro_rules! covariant {
    {$($body:tt)*} => {
        $crate::__either_variance_family! {
            @ Covariant
            $($body)*
        }
    };
}

/// Implement a simple contravariant family.
///
/// The family must be proven contravariant by the compiler, and it must be well-formed
/// for *any* `'varying` lifetime (that is, no custom `'lower` or `Upper` bounds are permitted).
///
/// See [`covariant`] for examples; the sole difference in syntax is that
/// `ContravariantFamily` is used instead of `CovariantFamily`.
///
/// # Safety
/// Same as [`covariant`]; a `#[unsafe(not_a_foreign_fundamental_type)]` marker is required.
#[macro_export]
macro_rules! contravariant {
    {$($body:tt)*} => {
        $crate::__either_variance_family! {
            @ Contravariant
            $($body)*
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __either_variance_family {
    {
        @ Covariant
        impl<$varying:lifetime, {$($params:tt)*}> CovariantFamily <'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $family_type:ty
        as $varying_type:ty
        where {$($non_varying_wb:tt)*}
        and for<$wb_varying:lifetime> {
            $({$($varying_wb:tt)*}),*$(,)?
        }
    } => {
        // SAFETY: Asserted by user of this macro.
        const _: () = unsafe { $crate::assert_not_a_foreign_fundamental_type() };

        #[expect(
            unsafe_code,
            reason = "safely encapsulated, aside from `#[unsafe(not_a_foreign_fundamental_type)]`",
        )]
        const _: () = {
            $crate::__impl_with_lifetime! {
                impl<$varying, {$($params)*}> _ <'_, _>
                for #[unsafe(not_a_foreign_fundamental_type)] $family_type
                as $varying_type
                where {$($non_varying_wb)*}
                and for<$wb_varying> {
                    $({$($varying_wb)*},)*
                }
            }

            $crate::__impl_covariant_family! {
                impl<$varying, {$($params)*}> CovariantFamily<'_, _>
                for #[unsafe(not_a_foreign_fundamental_type)] $family_type
                as $varying_type
                where {$($non_varying_wb)*}
                and for<$wb_varying> {
                    $({$($varying_wb)*},)*
                }
            }
        };
    };

    {
        @ Contravariant
        impl<$varying:lifetime, {$($params:tt)*}> ContravariantFamily <'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $family_type:ty
        as $varying_type:ty
        where {$($non_varying_wb:tt)*}
        and for<$wb_varying:lifetime> {
            $({$($varying_wb:tt)*}),*$(,)?
        }
    } => {
        // SAFETY: Asserted by user of this macro.
        const _: () = unsafe { $crate::assert_not_a_foreign_fundamental_type() };

        #[expect(
            unsafe_code,
            reason = "safely encapsulated, aside from `#[unsafe(not_a_foreign_fundamental_type)]`",
        )]
        const _: () = {
            $crate::__impl_with_lifetime! {
                impl<$varying, {$($params)*}> _ <'_, _>
                for #[unsafe(not_a_foreign_fundamental_type)] $family_type
                as $varying_type
                where {$($non_varying_wb)*}
                and for<$wb_varying> {
                    $({$($varying_wb)*},)*
                }
            }

            $crate::__impl_contravariant_family! {
                impl<$varying, {$($params)*}> ContravariantFamily<'_, _>
                for #[unsafe(not_a_foreign_fundamental_type)] $family_type
                as $varying_type
                where {$($non_varying_wb)*}
                and for<$wb_varying> {
                    $({$($varying_wb)*},)*
                }
            }
        };
    };

    {
        @ $variance:tt
        impl<$varying:lifetime> $variance_family:tt <'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $family_type:ty
        as $varying_type:ty
        where {$($non_varying_wb:tt)*}
        and for<$wb_varying:lifetime> {
            $({$($varying_wb:tt)*}),*$(,)?
        }
    } => {
        $crate::__either_variance_family! {
            @ $variance
            impl<$varying, {}> $variance_family <'_, _>
            for #[unsafe(not_a_foreign_fundamental_type)] $family_type
            as $varying_type
            where {$($non_varying_wb)*}
            and for<$wb_varying> {
                $({$($varying_wb)*},)*
            }
        }
    };

    {
        @ $variance:tt
        impl<$varying:lifetime$(, {$($params:tt)*})?> $variance_family:tt <'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $family_type:ty
        as $varying_type:ty
        where {$($non_varying_wb:tt)*}
    } => {
        $crate::__either_variance_family! {
            @ $variance
            impl<$varying$(, {$($params)*})?> $variance_family <'_, _>
            for #[unsafe(not_a_foreign_fundamental_type)] $family_type
            as $varying_type
            where {$($non_varying_wb)*} and for<$varying> {}
        }
    };

    {
        @ $variance:tt
        impl<$varying:lifetime$(, {$($params:tt)*})?> $variance_family:tt <'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $family_type:ty
        as $varying_type:ty
        where for<$wb_varying:lifetime> {
            $({$($varying_wb:tt)*}),*$(,)?
        }
    } => {
        $crate::__either_variance_family! {
            @ $variance
            impl<$varying$(, {$($params)*})?> $variance_family <'_, _>
            for #[unsafe(not_a_foreign_fundamental_type)] $family_type
            as $varying_type
            where {} and for<$varying> {
                $({$($varying_wb)*},)*
            }
        }
    };

    {
        @ $variance:tt
        impl<$varying:lifetime$(, {$($params:tt)*})?> $variance_family:tt <'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $family_type:ty
        as $varying_type:ty
    } => {
        $crate::__either_variance_family! {
            @ $variance
            impl<$varying$(, {$($params)*})?> $variance_family <'_, _>
            for #[unsafe(not_a_foreign_fundamental_type)] $family_type
            as $varying_type
            where {} and for<$varying> {}
        }
    };
}

/// Implement a simple lifetime family which does not use the lifetime parameter at all.
///
/// # Examples
/// ```rust
/// # use core::{fmt::Debug, marker::PhantomData};
/// # use variance_family::unvarying;
/// struct Foo<'c, T: ?Sized + Debug>(&'c T);
///
/// // If you need to make a family for a foreign type, you need a separate type (which can just
/// // be a newtype). If making a family for a local type, it's preferred to call
/// // `unvarying!` directly on that local type.
/// struct FooFamily<'c, T: ?Sized + Debug>(Foo<'c, T>>);
///
/// unvarying! {
///     // `(Co+Contra)variantFamily<'_, _>` is simply a magic string you must include.
///     // (This is intended to mimic real Rust syntax.)
///     impl<{'c, T}> (Co+Contra)variantFamily<'_, _>
///     // (See below docs for why this `unsafe` is necessary.)
///     // SAFETY: `FooFamily` is defined in this crate.
///     for #[unsafe(not_a_foreign_fundamental_type)] FooFamily<'c, T>
///     // The type given here is the could-be-`'varying`-parameterized type which does not
///     // actually have a varying lifetime.
///     as Foo<'c, T>
///     where {
///         // We could have done `impl<{'c, T: ?Sized + Debug}>` instead.
///         T: ?Sized + Debug,
///     }
/// }
/// ```
///
/// Simpler example:
/// ```rust
/// # use variance_family::unvarying;
/// struct Bar(Vec<u8>);
///
/// unvarying! {
///     // SAFETY: `Bar` is defined in this crate.
///     impl (Co+Contra)variantFamily<'_, _> for #[unsafe(not_a_foreign_fundamental_type)] Bar
/// }
/// ```
///
/// # Safety
/// Same as [`covariant`]; a `#[unsafe(not_a_foreign_fundamental_type)]` marker is required.
///
/// (This macro intentionally does not `expect` the `unsafe_code` lint or similar.)
#[macro_export]
macro_rules! unvarying {
    {
        impl<{$($params:tt)*}> (Co+Contra)variantFamily <'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $family_type:ty
        as $varying_type:ty
        where {$($non_varying_wb:tt)*}
    } => {
        // SAFETY: Asserted by user of this macro.
        const _: () = unsafe { $crate::assert_not_a_foreign_fundamental_type() };

        #[expect(
            unsafe_code,
            reason = "safely encapsulated, aside from `#[unsafe(not_a_foreign_fundamental_type)]`",
        )]
        #[expect(single_use_lifetimes, reason = "`'varying` is unused")]
        const _: () = {
                $crate::__impl_with_lifetime! {
                impl<'varying, {$($params)*}> _ <'_, _>
                for #[unsafe(not_a_foreign_fundamental_type)] $family_type
                as $varying_type
                where {$($non_varying_wb)*}
                and for<'varying> {}
            }

            $crate::__impl_covariant_family! {
                impl<'varying, {$($params)*}> CovariantFamily <'_, _>
                for #[unsafe(not_a_foreign_fundamental_type)] $family_type
                as $varying_type
                where {$($non_varying_wb)*}
                and for<'varying> {}
            }

            $crate::__impl_contravariant_family! {
                impl<'varying, {$($params)*}> ContravariantFamily <'_, _>
                for #[unsafe(not_a_foreign_fundamental_type)] $family_type
                as $varying_type
                where {$($non_varying_wb)*}
                and for<'varying> {}
            }
        };
    };

    {
        impl (Co+Contra)variantFamily <'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $family_type:ty
        $(as $varying_type:ty)?
        $(where {$($non_varying_wb:tt)*})?
    } => {
        $crate::unvarying! {
            impl<{}> (Co+Contra)variantFamily <'_, _>
            for #[unsafe(not_a_foreign_fundamental_type)] $family_type
            $(as $varying_type)?
            $(where {$($non_varying_wb)*})?
        }
    };

    {
        impl<{$($params:tt)*}> (Co+Contra)variantFamily <'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $family_type:ty
        $(where {$($non_varying_wb:tt)*})?
    } => {
        $crate::unvarying! {
            impl<{$($params)*}> (Co+Contra)variantFamily <'_, _>
            for #[unsafe(not_a_foreign_fundamental_type)] $family_type
            as $family_type
            $(where {$($non_varying_wb)*})?
        }
    };

    {
        impl<{$($params:tt)*}> (Co+Contra)variantFamily <'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $family_type:ty
        as $varying_type:ty
    } => {
        $crate::unvarying! {
            impl<{$($params)*}> (Co+Contra)variantFamily <'_, _>
            for #[unsafe(not_a_foreign_fundamental_type)] $family_type
            as $varying_type
            where {}
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_with_lifetime {
    {
        impl<$varying:lifetime, {$($params:tt)*}> _<'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $family_type:ty
        as $varying_type:ty
        where {$($non_varying_wb:tt)*}
        and for<$wb_varying:lifetime> {
            $({$($varying_wb:tt)*}),*$(,)?
        }
    } => {
        // NOTE: theoretically, due to pathological macro types, `$family_type` and `$varying_type`
        // could be different each time we use them. Such a scenario cannot cause unsoundness,
        // though, since we do not rely on delicate reasoning about the types;
        // we entirely leave it to the compiler to prove the necessary information.
        // SAFETY: Thanks to mixed macro hygiene, the `Upper` generic we introduce cannot possibly
        // be used in `$varying_type`. Additionally, a combination of orphan rules and the
        // `#[unsafe(not_a_foreign_fundamental_type)]` annotation ensure that the other safety
        // condition is met.
        unsafe impl<$varying, Upper: $crate::UpperBound, $($params)*>
        $crate::WithLifetime<$varying, '_, Upper> for $family_type
        where
            $(
                $($varying_wb)*,
            )*
            $($non_varying_wb)*
        {
            type Is = $varying_type;
        }

        // SAFETY: `ChangeBounds::prove_equal` is implemented with the function body `{ varying }`,
        // so this implementation is certainly sound.
        unsafe impl<$varying, Upper: $crate::UpperBound, $($params)*>
        $crate::ChangeBounds<$varying, '_, Upper, $varying_type> for $family_type
        where
            $(
                $($varying_wb)*,
            )*
            $($non_varying_wb)*
        {
            fn prove_equal<'other_lower, OtherUpper>(
                varying: $crate::RawMutVarying<'varying, 'other_lower, OtherUpper, Self>,
            ) -> *mut *mut $varying_type
            where
                // Make the lifetime early-bound.
                'other_lower: 'other_lower,
                OtherUpper: $crate::UpperBound,
            {
                varying
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_covariant_family {
    {
        impl<$varying:lifetime, {$($params:tt)*}> CovariantFamily<'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $family_type:ty
        as $varying_type:ty
        where {$($non_varying_wb:tt)*}
        and for<$wb_varying:lifetime> {
            $({$($varying_wb:tt)*}),*$(,)?
        }
    } => {
        // NOTE: theoretically, due to pathological macro types, `$family_type` and `$varying_type`
        // could be different each time we use them. Such a scenario cannot cause unsoundness,
        // though, since we do not rely on delicate reasoning about the types;
        // we entirely leave it to the compiler to prove the necessary information.
        // SAFETY: `CovariantFamily::prove_covariance` is implemented with the function body
        // `{ long }`, so the implementation is certainly sound.
        unsafe impl<'lower, Upper: $crate::UpperBound, $($params)*>
        $crate::CovariantFamily<'lower, Upper>
        for $family_type
        where
            $(
                for<$varying> $($varying_wb)*,
            )*
            $($non_varying_wb)*
        {
            fn prove_covariance<'long, 'short>(
                long: $crate::RawVarying<'long, 'lower, Upper, Self>,
            ) -> $crate::RawVarying<'short, 'lower, Upper, Self>
            where
                Upper: 'long,
                'long: 'short,
                'short: 'lower,
            {
                long
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_contravariant_family {
    {
        impl<$varying:lifetime, {$($params:tt)*}> ContravariantFamily<'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $family_type:ty
        as $varying_type:ty
        where {$($non_varying_wb:tt)*}
        and for<$wb_varying:lifetime> {
            $({$($varying_wb:tt)*}),*$(,)?
        }
    } => {
        // NOTE: theoretically, due to pathological macro types, `$family_type` and `$varying_type`
        // could be different each time we use them. Such a scenario cannot cause unsoundness,
        // though, since we do not rely on delicate reasoning about the types;
        // we entirely leave it to the compiler to prove the necessary information.
        // SAFETY: `ContravariantFamily::prove_contravariance` is implemented with the function body
        // `{ short }`, so the implementation is certainly sound.
        unsafe impl<'lower, Upper: $crate::UpperBound, $($params)*>
        $crate::ContravariantFamily<'lower, Upper>
        for $family_type
        where
            $(
                for<$varying> $($varying_wb)*,
            )*
            $($non_varying_wb)*
        {
            fn prove_contravariance<'short, 'long>(
                short: $crate::RawVarying<'short, 'lower, Upper, Self>,
            ) -> $crate::RawVarying<'long, 'lower, Upper, Self>
            where
                Upper: 'long,
                'long: 'short,
                'short: 'lower,
            {
                short
            }
        }
    };
}

/// Denotes that `#[unsafe(not_a_foreign_fundamental_type)]` was used.
#[doc(hidden)]
pub const unsafe fn assert_not_a_foreign_fundamental_type() {}

/// Implement a variety of `core` traits for a tuple ZST struct whose sole field is
/// [`PhantomData`](::core::marker::PhantomData).
///
/// This macro implements `Clone`, `Copy`, `Debug`, `Default`, `Eq`, `Hash`, `Ord`, `PartialEq`,
/// and `PartialOrd`.
///
/// # Example
/// ```
/// use core::marker::PhantomData;
/// use variance_family::phantom_zst_methods;
///
///
/// /// `Foo` is invariant over `T` and `U`.
/// ///
/// /// It unconditionally implements `Clone`, `Copy`, `Debug`, `Default`, `Eq`, `Hash`,
/// /// `Ord`, `PartialEq`, `PartialOrd`, `Send`, `Sync`, `Unpin`, etc.
/// ///
/// /// ("unconditionally" is more precisely stated "when the struct is well-formed",
/// /// which requires `U: Sized` in this case.)
/// // Note that `fn` is used to make it `Send + Sync` despite the usage of `*mut`.
/// struct Foo<T: ?Sized, U>(PhantomData<fn(*mut T, *mut U)>);
///
/// phantom_zst_methods!(impl<{T: ?Sized, U}> _ for Foo<{T, U}>);
///
/// // Or:
/// // phantom_zst_methods! {
/// //     impl<{T, U}> _ for Foo<{T, U}> where {T: ?Sized};
/// // }
/// ```
#[macro_export]
macro_rules! phantom_zst_methods {
    (
        impl$(<{$($impl_params:tt)*}>)? _
        for $name:ident $(<{$($gen_params:tt)*}>)?
        $(where {$($where_bounds:tt)*})? $(;)?
    ) => {
        impl $(<$($impl_params)*>)? ::core::clone::Clone for $name $(<$($gen_params)*>)?
        where
            $($($where_bounds)*)?
        {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl $(<$($impl_params)*>)? ::core::marker::Copy for $name $(<$($gen_params)*>)?
        where
            $($($where_bounds)*)?
        {}

        impl $(<$($impl_params)*>)? ::core::fmt::Debug for $name $(<$($gen_params)*>)?
        where
            $($($where_bounds)*)?
        {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_tuple(::core::stringify!($name))
                    .field(&self.0)
                    .finish()
            }
        }

        impl $(<$($impl_params)*>)? ::core::default::Default for $name $(<$($gen_params)*>)?
        where
            $($($where_bounds)*)?
        {
            fn default() -> Self {
                Self(::core::marker::PhantomData)
            }
        }

        impl $(<$($impl_params)*>)? ::core::cmp::Eq for $name $(<$($gen_params)*>)?
        where
            $($($where_bounds)*)?
        {}

        impl $(<$($impl_params)*>)? ::core::hash::Hash for $name $(<$($gen_params)*>)?
        where
            $($($where_bounds)*)?
        {
            fn hash<H: ::core::hash::Hasher>(&self, _state: &mut H) {}
        }

        impl $(<$($impl_params)*>)? ::core::cmp::Ord for $name $(<$($gen_params)*>)?
        where
            $($($where_bounds)*)?
        {
            fn cmp(&self, _other: &Self) -> ::core::cmp::Ordering {
                ::core::cmp::Ordering::Equal
            }
        }

        impl $(<$($impl_params)*>)? ::core::cmp::PartialEq for $name $(<$($gen_params)*>)?
        where
            $($($where_bounds)*)?
        {
            fn eq(&self, _other: &Self) -> ::core::primitive::bool {
                true
            }
        }

        impl $(<$($impl_params)*>)? ::core::cmp::PartialOrd for $name $(<$($gen_params)*>)?
        where
            $($($where_bounds)*)?
        {
            fn partial_cmp(&self, other: &Self) -> ::core::option::Option<::core::cmp::Ordering> {
                ::core::option::Option::Some(<Self as ::core::cmp::Ord>::cmp(self, other))
            }
        }
    };
}
