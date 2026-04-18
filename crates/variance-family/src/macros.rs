//! [`covariant`], [`contravariant`], and [`unvarying`] macros that cover simple cases.
//!
//! [`generic_wrapper`] for a slightly more complicated (and slightly more `unsafe`) case.
//!
//! [`varying_ref_wrapper`] and [`varying_ref_mut_wrapper`] for specific (and yet more `unsafe`)
//! cases.
//!
//! Additionally, a [`phantom_zst_methods`] macro for phantom variance family markers.
//!
//! Note that these macros are what `variance-family` uses internally for *all* of its
//! `unsafe impl`s, so they should be quite versatile.

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
/// This macro `expect`s the `unsafe_code` lint for most of its implementation, since
/// its `unsafe impl`s are mostly encapsulated, but calls an `unsafe` function to denote the
/// remaining `unsafe` assertion.
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
///
/// This macro `expect`s the `unsafe_code` lint for most of its implementation, since
/// its `unsafe impl`s are mostly encapsulated, but calls an `unsafe` function to denote the
/// remaining `unsafe` assertion.
#[macro_export]
macro_rules! contravariant {
    {$($body:tt)*} => {
        $crate::__either_variance_family! {
            @ Contravariant
            $($body)*
        }
    };
}

/// Implement a simple lifetime family which does not use the lifetime parameter at all.
///
/// # Examples
/// ```rust
/// # use core::fmt::Debug;
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
/// This macro `expect`s the `unsafe_code` lint for most of its implementation, since
/// its `unsafe impl`s are mostly encapsulated, but calls an `unsafe` function to denote the
/// remaining `unsafe` assertion.
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
        const _: () = {
            $crate::__impl_with_lifetime! {
                impl<'varying, '_, {$($params)*}> _ <'_, _>
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

/// Note: This macro would be unsound if it leaked outside `variance-family`. (Also, it must
/// only be called on types from `core`, `alloc`, `std`, or `variance-family`. Since we have no
/// dependencies, that is trivially true.)
///
/// Since it's not marked `#[macro_export]`, it's sound.
macro_rules! atomic_types {
    ($($atomic_type:ty),+ $(,)?) => {
        #[expect(
            unsafe_code,
            reason = "safely encapsulated, thanks to being crate-private",
        )]
        const _: () = {
            $(
                $crate::unvarying! {
                    impl (Co+Contra)variantFamily<'_, _>
                    // SAFETY: `variance-family` is allowed to implement traits for this atomic type
                    // in `core`, `alloc`, `std`, or `variance-family`.
                    for #[unsafe(not_a_foreign_fundamental_type)] $atomic_type
                }
            )+
        };
    };
}

pub(crate) use atomic_types;

/// Implement variance family traits for a type which contains other lifetime families but does
/// not directly use `'varying`.
///
/// Support of const-generics isn't perfect, but should generally be sufficient. (Also, commas
/// are handled *extremely* loosely, except after const-generic parameters, which must be followed
/// by at least one comma.)
///
/// # Troubleshooting
/// The most common error (if the macro parses successfully) is the varying type needing its
/// generics to be `Sized`. You need to add an `(Is: Sized)` bound to contrain the `Is` associated
/// type (of the relevant type family) to be `Sized`.
///
/// # Examples
/// ```rust
/// # use core::fmt::Debug;
/// # use variance_family::generic_wrapper;
/// struct Foo<'c, T: ?Sized, const N: usize, U: Debug, V, W>(fn(&'c T), U, V, W);
///
/// // Could be a foreign type that we want a variance family for.
/// struct Bar<'c, T: ?Sized, const N: usize, U: Debug, V, W>(fn(&'c T), U, V, W);
///
/// // The `'varying`-parameterized version of `Foo<'c, T, N, U, V, W>` will be
/// // `Bar<'c, T<'varying>, N, U<'varying>, V, W<'varying>>`,
/// // regardless of whether `V` happens to also implement `variance-family` traits.
///
/// generic_wrapper! {
///     // `([Co] + [Contra])variantFamily<'_, _>` is simply a magic string you must include.
///     // (This is intended to mimic real Rust syntax, of course, including `[const]` meaning
///     // something like "conditionally `const`".)
///     // See below docs for the meaning of these attributes and their safety requirements.
///     //
///     // Each generic parameter of `Foo` must be listed out here. No bounds are permitted here,
///     // for simplicity of implementation.
///     impl<{
///         // No attribute. This is not a variance family. (Obviously. It's not even a type.)
///         'c,
///         // The attribute indicates that `T` is treated as a variance family.
///         // SAFETY: The varying type `Bar<'c, T, N, U, V>` is contravariant over `T`.
///         #[unsafe(contravariant)] T,
///         // No attribute. Not treated as a variance family. (Obviously. It's not even a type.)
///         const N: usize,
///         // SAFETY: The varying type `Bar<'c, T, N, U, V>` is covariant over `U`.
///         #[unsafe(covariant)] U (Is: Sized),
///         // No attribute. Not treated as a variance family.
///         V,
///         // `W` is required to implement `UnvaryingFamily`.
///         #[unvarying] W (Is: Sized),
///     }> ([Co] + [Contra])variantFamily<'_, _>
///     // SAFETY: `Foo` is defined in this crate.
///     for #[unsafe(not_a_foreign_fundamental_type)] Foo<..>
///     // The above type is the family type. The below type is the varying type.
///     // The varying type defaults to being the family type (albeit with substituted generics).
///     as Bar<..>
///     where {
///         T: ?Sized,
///         U: Debug,
///     }
/// }
/// ```
///
/// Simpler example:
/// ```rust
/// # use variance_family::generic_wrapper;
/// struct MyOption<T: ?Sized>(T);
///
/// generic_wrapper! {
///     // SAFETY: The varying type `MyOption<T>` is covariant over `T`.
///     impl<{#[unsafe(covariant)] T}> ([Co] + [Contra])variantFamily<'_, _>
///     // SAFETY: `MyOption` is defined in this crate.
///     for #[unsafe(not_a_foreign_fundamental_type)] MyOption<..>
///     where {T: ?Sized}
/// }
/// ```
///
/// # Safety
/// There are two unsafe attributes for denoting variance, which are **NOT** checked. You
/// **MUST** be correct about them. Additionally, there is an attribute used to fulfill
/// [`WithLifetime`]'s safety conditions.
///
/// This macro `expect`s the `unsafe_code` lint for most of its implementation, since
/// its `unsafe impl`s are mostly encapsulated, but calls `unsafe` functions to denote the
/// remaining `unsafe` assertions.
///
/// ## `unsafe(covariant)` and `unsafe(contravariant)`
/// The varying type **must** be covariant over parameters marked with `unsafe(covariant)` and must
/// be contravariant over parameters marked with `unsafe(contravariant)`.
///
/// When all of the `unsafe(covariant)` types implement `CovariantFamily` and all of the
/// `unsafe(contravariant)` types implement `ContravariantFamily`, the varying type will implement
/// `CovariantFamily`.
///
/// Conversely, when all of the `unsafe(covariant)` types implement `ContravariantFamily` and all
/// of the `unsafe(contravariant)` types implement `CovariantFamily`, the varying type will implement
/// `ContravariantFamily`.
///
/// ## `unsafe(not_a_foreign_fundamental_type)`
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
macro_rules! generic_wrapper {
    {
        impl<{$($params:tt)*}> ([Co] + [Contra])variantFamily<'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $($name:ident)::+<..>
        as $($varying_name:ident)::+<..>
        where {$($where_bounds:tt)*}
    } => {
        // SAFETY: Asserted by user of this macro.
        const _: () = unsafe { $crate::assert_variance() };
        // SAFETY: Asserted by user of this macro.
        const _: () = unsafe { $crate::assert_not_a_foreign_fundamental_type() };

        #[expect(
            unsafe_code,
            reason = "safely encapsulated, aside from the three unsafe attributes",
        )]
        const _: () = {
            $crate::__impl_generic_wrapper! {
                {$($params)*} {} {} {}
                $($name)::+ $($varying_name)::+
                {$($where_bounds)*} {} {} {}
            }
        };
    };

    {
        impl<{
            $($params:tt)*
        }> ([Co] + [Contra])variantFamily<'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $($name:ident)::+<..>
        $(where {$($where_bounds:tt)*})?
    } => {
        $crate::generic_wrapper! {
            impl<{$($params)*}> ([Co] + [Contra])variantFamily<'_, _>
            for #[unsafe(not_a_foreign_fundamental_type)] $($name)::+<..>
            as $($name)::+<..>
            where {$($($where_bounds)*)?}
        }
    };

    {
        impl<{
            $($params:tt)*
        }> ([Co] + [Contra])variantFamily<'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $($name:ident)::+<..>
        as $($varying_name:ident)::+<..>
    } => {
        $crate::generic_wrapper! {
            impl<{$($params)*}> ([Co] + [Contra])variantFamily<'_, _>
            for #[unsafe(not_a_foreign_fundamental_type)] $($name)::+<..>
            as $($varying_name)::+<..>
            where {}
        }
    };
}

/// Implement variance traits for something resembling `&'varying T`.
///
/// Despite how specific this macro is, it is sufficient for many types in `std`.
///
/// # Examples
/// ```rust
/// # use core::{fmt::Debug, marker::PhantomData};
/// # use variance_family::varying_ref_wrapper;
/// struct VaryingFoo<T: ?Sized + Debug>(PhantomData<fn() -> T>);
///
/// // Could be a foreign type that we want a variance family for.
/// struct Foo<'varying, T: ?Sized + Debug + 'varying>(&'varying T);
///
/// varying_ref_wrapper! {
///     // `CovariantFamily<'_, _>` is simply a magic string you must include.
///     // (This is intended to mimic real Rust syntax, of course.)
///     impl<T (Is: Debug)> CovariantFamily<'_, _>
///     // SAFETY: `VaryingFoo` is defined in this crate.
///     for #[unsafe(not_a_foreign_fundamental_type)] VaryingFoo<T>
///     // `#[unsafe(covariant)]` is a magic string you must include,
///     // to mimic `generic_wrapper`'s syntax.
///     // SAFETY: The varying type `Foo<'varying, T>` is covariant over `'varying` and `T`.
///     as Foo<#[unsafe(covariant)] '_, #[unsafe(covariant)] T>
///     where {
///         T: ?Sized,
///     }
/// }
/// ```
///
/// # Safety
/// This macro `expect`s the `unsafe_code` lint for most of its implementation, since
/// its `unsafe impl`s are mostly encapsulated, but calls `unsafe` functions to denote the
/// remaining `unsafe` assertions.
///
/// ## `unsafe(covariant)`
/// The varying type **must** be covariant over its parameter marked with `unsafe(covariant)`.
///
/// When the `unsafe(covariant)` type implements `CovariantFamily`, the varying type will implement
/// `CovariantFamily`.
///
/// ## `unsafe(not_a_foreign_fundamental_type)`
/// See [`generic_wrapper`]; a `#[unsafe(not_a_foreign_fundamental_type)]` marker is required.
#[macro_export]
macro_rules! varying_ref_wrapper {
    {
        impl<$t:ident $((Is: $($bound:tt)*))?> CovariantFamily<'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $($name:ident)::+<$fam_t:ident>
        as $($varying_name:ident)::+<
            #[unsafe(covariant)] '_,
            #[unsafe(covariant)] $varying_t:ident $(,)?
        >
        $(where {$($where_bounds:tt)*})?
    } => {
        // SAFETY: Asserted by user of this macro.
        const _: () = unsafe { $crate::assert_variance() };
        // SAFETY: Asserted by user of this macro.
        const _: () = unsafe { $crate::assert_not_a_foreign_fundamental_type() };

        #[expect(
            unsafe_code,
            reason = "safely encapsulated, aside from the two unsafe attributes",
        )]
        const _: () = {
            $crate::__impl_varying_ref_wrapper! {
                $t $((Is: $($bound)*))?
                for $($name)::+<$fam_t>
                as $($varying_name)::+<'_, $varying_t>
                where {$($($where_bounds)*)?}
            }
        };
    };
}

/// Implement variance traits for something resembling `&'varying mut T`.
///
/// Despite how specific this macro is, it is sufficient for many types in `std`.
///
/// # Examples
/// ```rust
/// # use core::{fmt::Debug, marker::PhantomData};
/// # use variance_family::varying_ref_mut_wrapper;
/// struct VaryingFoo<T: ?Sized + Debug>(PhantomData<fn(*mut T)>);
///
/// // Could be a foreign type that we want a variance family for.
/// struct Foo<'varying, T: ?Sized + Debug + 'varying>(&'varying mut T);
///
/// varying_ref_mut_wrapper! {
///     // `(Co+Contra)variantFamily<'_, _>` is simply a magic string you must include.
///     // (This is intended to mimic real Rust syntax, of course.)
///     impl<T (Is: Debug)> (Co+Contra)variantFamily<'_, _>
///     // SAFETY: `VaryingFoo` is defined in this crate.
///     for #[unsafe(not_a_foreign_fundamental_type)] VaryingFoo<T>
///     // `#[unvarying]` is a magic string you must include, to mimic `generic_wrapper`'s syntax.
///     as Foo<'_, #[unvarying] T>
///     where {
///         T: ?Sized,
///     }
/// }
/// ```
///
/// # Safety
/// See [`generic_wrapper`]; a `#[unsafe(not_a_foreign_fundamental_type)]` marker is required.
///
/// This macro `expect`s the `unsafe_code` lint for most of its implementation, since
/// its `unsafe impl`s are mostly encapsulated, but calls an `unsafe` function to denote the
/// remaining `unsafe` assertion.
#[macro_export]
macro_rules! varying_ref_mut_wrapper {
    {
        impl<$t:ident $((Is: $($bound:tt)*))?> (Co+Contra)variantFamily<'_, _>
        for #[unsafe(not_a_foreign_fundamental_type)] $($name:ident)::+<$fam_t:ident>
        as $($varying_name:ident)::+<
            '_,
            #[unvarying] $varying_t:ident $(,)?
        >
        $(where {$($where_bounds:tt)*})?
    } => {
        // SAFETY: Asserted by user of this macro.
        const _: () = unsafe { $crate::assert_variance() };
        // SAFETY: Asserted by user of this macro.
        const _: () = unsafe { $crate::assert_not_a_foreign_fundamental_type() };

        #[expect(
            unsafe_code,
            reason = "safely encapsulated, aside from the two unsafe attributes",
        )]
        const _: () = {
            $crate::__impl_varying_ref_mut_wrapper! {
                $t $((Is: $($bound)*))?
                for $($name)::+<$fam_t>
                as $($varying_name)::+<'_, $varying_t>
                where {$($($where_bounds)*)?}
            }
        };
    };
}

/// This should not be called directly (outside of this module). Do so at your own risk (including
/// risk of UB).
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
                impl<$varying, $varying, $varying, {$($params)*}> _ <'_, _>
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
                impl<$varying, $varying, $varying, {$($params)*}> _ <'_, _>
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

/// This should not be called directly (outside of this module). Do so at your own risk (including
/// risk of UB).
#[doc(hidden)]
#[macro_export]
macro_rules! __impl_with_lifetime {
    {
        impl<
            $varying:lifetime, $varying_single_use:lifetime, $($varying_single_use_param:lifetime,)?
            {$($params:tt)*
        }> _<'_, _>
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
        unsafe impl<$($varying_single_use_param,)? Upper: $crate::UpperBound, $($params)*>
        $crate::WithLifetime<$varying_single_use, '_, Upper> for $family_type
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

/// This should not be called directly (outside of this module). Do so at your own risk (including
/// risk of UB).
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

/// This should not be called directly (outside of this module). Do so at your own risk (including
/// risk of UB).
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

/// This should not be called directly (outside of this module). Do so at your own risk (including
/// risk of UB).
#[doc(hidden)]
#[macro_export]
macro_rules! __impl_generic_wrapper {
    {
        {
            ,
            $($remaining:tt)*
        }
        {$($impl_params:tt)*}
        {$($generics:tt)*}
        {$($varying_generics:tt)*}
        $($name:ident)::+ $($varying_name:ident)::+
        {$($where_bounds:tt)*}
        {$($covariant_types:ident ($($covariant_is_bound:tt)*))*}
        {$($contravariant_types:ident ($($contravariant_is_bound:tt)*))*}
        {$($unvarying_types:ident ($($unvarying_is_bound:tt)*))*}
    } => {
        $crate::__impl_generic_wrapper! {
            {
                $($remaining)*
            }
            {$($impl_params)*}
            {$($generics)*}
            {$($varying_generics)*}
            $($name)::+ $($varying_name)::+
            {$($where_bounds)*}
            {$($covariant_types ($($covariant_is_bound)*))*}
            {$($contravariant_types ($($contravariant_is_bound)*))*}
            {$($unvarying_types ($($unvarying_is_bound)*))*}
        }
    };

    {
        {
            $lt:lifetime
            $($remaining:tt)*
        }
        {$($impl_params:tt)*}
        {$($generics:tt)*}
        {$($varying_generics:tt)*}
        $($name:ident)::+ $($varying_name:ident)::+
        {$($where_bounds:tt)*}
        {$($covariant_types:ident ($($covariant_is_bound:tt)*))*}
        {$($contravariant_types:ident ($($contravariant_is_bound:tt)*))*}
        {$($unvarying_types:ident ($($unvarying_is_bound:tt)*))*}
    } => {
        $crate::__impl_generic_wrapper! {
            {
                $($remaining)*
            }
            {$($impl_params)* $lt,}
            {$($generics)* $lt,}
            {$($varying_generics)* $lt,}
            $($name)::+ $($varying_name)::+
            {$($where_bounds)*}
            {$($covariant_types ($($covariant_is_bound)*))*}
            {$($contravariant_types ($($contravariant_is_bound)*))*}
            {$($unvarying_types ($($unvarying_is_bound)*))*}
        }
    };

    {
        {
            const $const_param:ident: $const_type:ty,
            $($remaining:tt)*
        }
        {$($impl_params:tt)*}
        {$($generics:tt)*}
        {$($varying_generics:tt)*}
        $($name:ident)::+ $($varying_name:ident)::+
        {$($where_bounds:tt)*}
        {$($covariant_types:ident ($($covariant_is_bound:tt)*))*}
        {$($contravariant_types:ident ($($contravariant_is_bound:tt)*))*}
        {$($unvarying_types:ident ($($unvarying_is_bound:tt)*))*}
    } => {
        $crate::__impl_generic_wrapper! {
            {
                $($remaining)*
            }
            {$($impl_params)* const $const_param: $const_type,}
            {$($generics)* $const_param,}
            {$($varying_generics)* $const_param,}
            $($name)::+ $($varying_name)::+
            {$($where_bounds)*}
            {$($covariant_types ($($covariant_is_bound)*))*}
            {$($contravariant_types ($($contravariant_is_bound)*))*}
            {$($unvarying_types ($($unvarying_is_bound)*))*}
        }
    };

    {
        {
            $truly_unvarying_type:ident
            $($remaining:tt)*
        }
        {$($impl_params:tt)*}
        {$($generics:tt)*}
        {$($varying_generics:tt)*}
        $($name:ident)::+ $($varying_name:ident)::+
        {$($where_bounds:tt)*}
        {$($covariant_types:ident ($($covariant_is_bound:tt)*))*}
        {$($contravariant_types:ident ($($contravariant_is_bound:tt)*))*}
        {$($unvarying_types:ident ($($unvarying_is_bound:tt)*))*}
    } => {
        $crate::__impl_generic_wrapper! {
            {
                $($remaining)*
            }
            {$($impl_params)* $truly_unvarying_type,}
            {$($generics)* $truly_unvarying_type,}
            {$($varying_generics)* $truly_unvarying_type,}
            $($name)::+ $($varying_name)::+
            {$($where_bounds)*}
            {$($covariant_types ($($covariant_is_bound)*))*}
            {$($contravariant_types ($($contravariant_is_bound)*))*}
            {$($unvarying_types ($($unvarying_is_bound)*))*}
        }
    };

    {
        {
            #[unsafe(covariant)]
            $covariant:ident (Is: $($bound:tt)*)
            $($remaining:tt)*
        }
        {$($impl_params:tt)*}
        {$($generics:tt)*}
        {$($varying_generics:tt)*}
        $($name:ident)::+ $($varying_name:ident)::+
        {$($where_bounds:tt)*}
        {$($covariant_types:ident ($($covariant_is_bound:tt)*))*}
        {$($contravariant_types:ident ($($contravariant_is_bound:tt)*))*}
        {$($unvarying_types:ident ($($unvarying_is_bound:tt)*))*}
    } => {
        $crate::__impl_generic_wrapper! {
            {
                $($remaining)*
            }
            {$($impl_params)* $covariant,}
            {$($generics)* $covariant,}
            {$($varying_generics)* $covariant::Is,}
            $($name)::+ $($varying_name)::+
            {$($where_bounds)*}
            {$($covariant_types ($($covariant_is_bound)*))* $covariant ($($bound)*)}
            {$($contravariant_types ($($contravariant_is_bound)*))*}
            {$($unvarying_types ($($unvarying_is_bound)*))*}
        }
    };

    {
        {
            #[unsafe(covariant)]
            $covariant:ident
            $($remaining:tt)*
        }
        {$($impl_params:tt)*}
        {$($generics:tt)*}
        {$($varying_generics:tt)*}
        $($name:ident)::+ $($varying_name:ident)::+
        {$($where_bounds:tt)*}
        {$($covariant_types:ident ($($covariant_is_bound:tt)*))*}
        {$($contravariant_types:ident ($($contravariant_is_bound:tt)*))*}
        {$($unvarying_types:ident ($($unvarying_is_bound:tt)*))*}
    } => {
        $crate::__impl_generic_wrapper! {
            {
                $($remaining)*
            }
            {$($impl_params)* $covariant,}
            {$($generics)* $covariant,}
            {$($varying_generics)* $covariant::Is,}
            $($name)::+ $($varying_name)::+
            {$($where_bounds)*}
            {$($covariant_types ($($covariant_is_bound)*))* $covariant ()}
            {$($contravariant_types ($($contravariant_is_bound)*))*}
            {$($unvarying_types ($($unvarying_is_bound)*))*}
        }
    };

    {
        {
            #[unsafe(contravariant)]
            $contravariant:ident (Is: $($bound:tt)*)
            $($remaining:tt)*
        }
        {$($impl_params:tt)*}
        {$($generics:tt)*}
        {$($varying_generics:tt)*}
        $($name:ident)::+ $($varying_name:ident)::+
        {$($where_bounds:tt)*}
        {$($covariant_types:ident ($($covariant_is_bound:tt)*))*}
        {$($contravariant_types:ident ($($contravariant_is_bound:tt)*))*}
        {$($unvarying_types:ident ($($unvarying_is_bound:tt)*))*}
    } => {
        $crate::__impl_generic_wrapper! {
            {
                $($remaining)*
            }
            {$($impl_params)* $contravariant,}
            {$($generics)* $contravariant,}
            {$($varying_generics)* $contravariant::Is,}
            $($name)::+ $($varying_name)::+
            {$($where_bounds)*}
            {$($covariant_types ($($covariant_is_bound)*))*}
            {$($contravariant_types ($($contravariant_is_bound)*))* $contravariant ($($bound)*)}
            {$($unvarying_types ($($unvarying_is_bound)*))*}
        }
    };

    {
        {
            #[unsafe(contravariant)]
            $contravariant:ident
            $($remaining:tt)*
        }
        {$($impl_params:tt)*}
        {$($generics:tt)*}
        {$($varying_generics:tt)*}
        $($name:ident)::+ $($varying_name:ident)::+
        {$($where_bounds:tt)*}
        {$($covariant_types:ident ($($covariant_is_bound:tt)*))*}
        {$($contravariant_types:ident ($($contravariant_is_bound:tt)*))*}
        {$($unvarying_types:ident ($($unvarying_is_bound:tt)*))*}
    } => {
        $crate::__impl_generic_wrapper! {
            {
                $($remaining)*
            }
            {$($impl_params)* $contravariant,}
            {$($generics)* $contravariant,}
            {$($varying_generics)* $contravariant::Is,}
            $($name)::+ $($varying_name)::+
            {$($where_bounds)*}
            {$($covariant_types ($($covariant_is_bound)*))*}
            {$($contravariant_types ($($contravariant_is_bound)*))* $contravariant ()}
            {$($unvarying_types ($($unvarying_is_bound)*))*}
        }
    };

    {
        {
            #[unvarying]
            $unvarying:ident (Is: $($bound:tt)*)
            $($remaining:tt)*
        }
        {$($impl_params:tt)*}
        {$($generics:tt)*}
        {$($varying_generics:tt)*}
        $($name:ident)::+ $($varying_name:ident)::+
        {$($where_bounds:tt)*}
        {$($covariant_types:ident ($($covariant_is_bound:tt)*))*}
        {$($contravariant_types:ident ($($contravariant_is_bound:tt)*))*}
        {$($unvarying_types:ident ($($unvarying_is_bound:tt)*))*}
    } => {
        $crate::__impl_generic_wrapper! {
            {
                $($remaining)*
            }
            {$($impl_params)* $unvarying,}
            {$($generics)* $unvarying,}
            {$($varying_generics)* $unvarying::Is,}
            $($name)::+ $($varying_name)::+
            {$($where_bounds)*}
            {$($covariant_types ($($covariant_is_bound)*))*}
            {$($contravariant_types ($($contravariant_is_bound)*))*}
            {$($unvarying_types ($($unvarying_is_bound)*))* $unvarying ($($bound)*)}
        }
    };

    {
        {
            #[unvarying]
            $unvarying:ident
            $($remaining:tt)*
        }
        {$($impl_params:tt)*}
        {$($generics:tt)*}
        {$($varying_generics:tt)*}
        $($name:ident)::+ $($varying_name:ident)::+
        {$($where_bounds:tt)*}
        {$($covariant_types:ident ($($covariant_is_bound:tt)*))*}
        {$($contravariant_types:ident ($($contravariant_is_bound:tt)*))*}
        {$($unvarying_types:ident ($($unvarying_is_bound:tt)*))*}
    } => {
        $crate::__impl_generic_wrapper! {
            {
                $($remaining)*
            }
            {$($impl_params)* $unvarying,}
            {$($generics)* $unvarying,}
            {$($varying_generics)* $unvarying::Is,}
            $($name)::+ $($varying_name)::+
            {$($where_bounds)*}
            {$($covariant_types ($($covariant_is_bound)*))*}
            {$($contravariant_types ($($contravariant_is_bound)*))*}
            {$($unvarying_types ($($unvarying_is_bound)*))* $unvarying ()}
        }
    };

    {
        {}
        {$($impl_params:tt)*}
        {$($generics:tt)*}
        {$($varying_generics:tt)*}
        $($name:ident)::+ $($varying_name:ident)::+
        {$($where_bounds:tt)*}
        {$($covariant_types:ident ($($covariant_is_bound:tt)*))*}
        {$($contravariant_types:ident ($($contravariant_is_bound:tt)*))*}
        {$($unvarying_types:ident ($($unvarying_is_bound:tt)*))*}
    } => {
        // SAFETY:
        // - We can assume (by the safety condition of `WithLifetime`)
        //   that no `$covariant_types::Is` nor `$contravariant_types::Is` uses `'lower` or `Upper`,
        //   and since macro hygiene ensures that `'lower` and `Upper` are only exposed to those
        //   types (and not to arbitrary user code, say where-bounds), it follows that
        //   `$varying_name<$varying_generics>` does not use `'lower` or `Upper`.
        // - The caller asserted with `#[unsafe(not_a_foreign_fundamental_type)]` that the crate
        //   calling this macro is permitted to implement `WithLifetime`
        //   for the indicated `$name<$($generics)*>` type.
        unsafe impl<'varying, 'lower, $($impl_params)* Upper>
            $crate::WithLifetime<'varying, 'lower, Upper>
        for $($name)::+<$($generics)*>
        where
            Upper: $crate::UpperBound,
            $(
                $covariant_types: $crate::WithLifetime<
                    'varying, 'lower, Upper,
                    Is: $($covariant_is_bound)*
                >,
            )*
            $(
                $contravariant_types: $crate::WithLifetime<
                    'varying, 'lower, Upper,
                    Is: $($contravariant_is_bound)*
                >,
            )*
            $(
                $unvarying_types: $crate::WithLifetime<
                    'varying, 'lower, Upper,
                    Is: $($unvarying_is_bound)*
                >,
            )*
            $($where_bounds)*
        {
            type Is = $($varying_name)::+<$($varying_generics)*>;
        }

        // SAFETY: See the first bullet point in the safety comment of the `WithLifetime` impl.
        unsafe impl<'varying, 'lower, $($impl_params)* Upper>
            $crate::ChangeBounds<
                'varying, 'lower, Upper,
                $($varying_name)::+<$($varying_generics)*>,
            >
        for $($name)::+<$($generics)*>
        where
            Upper: $crate::UpperBound,
            $(
                $covariant_types: $crate::WithLifetime<
                    'varying, 'lower, Upper,
                    Is: $($covariant_is_bound)*
                >,
            )*
            $(
                $contravariant_types: $crate::WithLifetime<
                    'varying, 'lower, Upper,
                    Is: $($contravariant_is_bound)*
                >,
            )*
            $(
                $unvarying_types: $crate::WithLifetime<
                    'varying, 'lower, Upper,
                    Is: $($unvarying_is_bound)*
                >,
            )*
            $($where_bounds)*
        {
            fn prove_equal<'other_lower, OtherUpper>(
                varying: $crate::RawMutVarying<'varying, 'other_lower, OtherUpper, Self>,
            ) -> *mut *mut $($name)::+<$($varying_generics)*>
            where
                Self: $crate::WithLifetime<'varying, 'other_lower, OtherUpper>,
                OtherUpper: $crate::UpperBound,
            {
                varying.cast()
            }
        }

        // SAFETY:
        // The varying type is something like `N<T<'varying>, U<'varying>, V, W<'varying>>`,
        // where `N` is covariant over `T` and contravariant over `U`, and has any variance
        // over the other parameters (which don't change when `'varying` changes), including `W`,
        // which implements `UnvaryingFamily` and thus `W<'varying>` does not actually change
        // when `'varying` changes.
        //
        // When `T` is covariant over `'varying` and `U` is contravariant over `'varying`,
        // `N<T<'varying>, U<'varying>, V, W<'varying>>` would be covariant over `'varying`.
        //
        // By the caller's unsafe assertions, we know which parameters are acting as `T, U, V, W`.
        unsafe impl<'lower, $($impl_params)* Upper>
            $crate::CovariantFamily<'lower, Upper>
        for $($name)::+<$($generics)*>
        where
            Upper: $crate::UpperBound,
            $(
                $covariant_types: $crate::CovariantFamily<
                    'lower, Upper,
                    Is: $($covariant_is_bound)*
                >,
            )*
            $(
                $contravariant_types: $crate::ContravariantFamily<
                    'lower, Upper,
                    Is: $($contravariant_is_bound)*
                >,
            )*
            $(
                $unvarying_types: $crate::UnvaryingFamily<
                    'lower, Upper,
                    Is: $($unvarying_is_bound)*
                >,
            )*
            $($where_bounds)*
        {
            fn prove_covariance<'long, 'short>(
                long: $crate::RawVarying<'long, 'lower, Upper, Self>,
            ) -> $crate::RawVarying<'short, 'lower, Upper, Self>
            where
                Upper: 'long,
                'long: 'short,
                'short: 'lower,
            {
                long.cast()
            }
        }

        // SAFETY:
        // See the above safety comment for the `CovariantFamily` impl.
        //
        // When `T` is contravariant over `'varying` and `U` is covariant over `'varying`,
        // `N<T<'varying>, U<'varying>, V, W<'varying>>` would be contravariant over `'varying`.
        //
        // By the caller's unsafe assertions, we know which parameters are acting as `T, U, V, W`.
        unsafe impl<'lower, $($impl_params)* Upper>
            $crate::ContravariantFamily<'lower, Upper> for $($name)::+<$($generics)*>
        where
            Upper: $crate::UpperBound,
            $(
                $covariant_types: $crate::ContravariantFamily<
                    'lower, Upper,
                    Is: $($covariant_is_bound)*
                >,
            )*
            $(
                $contravariant_types: $crate::CovariantFamily<
                    'lower, Upper,
                    Is: $($contravariant_is_bound)*
                >,
            )*
            $(
                $unvarying_types: $crate::UnvaryingFamily<
                    'lower, Upper,
                    Is: $($unvarying_is_bound)*
                >,
            )*
            $($where_bounds)*
        {
            fn prove_contravariance<'short, 'long>(
                short: $crate::RawVarying<'short, 'lower, Upper, Self>,
            ) -> $crate::RawVarying<'long, 'lower, Upper, Self>
            where
                Upper: 'long,
                'long: 'short,
                'short: 'lower,
            {
                short.cast()
            }
        }

    };
}

/// This should not be called directly (outside of this module). Do so at your own risk (including
/// risk of UB).
#[doc(hidden)]
#[macro_export]
macro_rules! __impl_varying_ref_wrapper {
    {
        $t:ident $((Is: $($bound:tt)*))?
        for $($name:ident)::+<$fam_t:ident>
        as $($varying_name:ident)::+<'_, $varying_t:ident>
        where {$($where_bounds:tt)*}
    } => {
        $crate::__impl_varying_with_lifetime! {
            $t $((Is: $($bound)*))?
            for $($name)::+<$fam_t>
            as $($varying_name)::+<'_, $varying_t>
            where {$($where_bounds)*}
        }

        // SAFETY:
        // Since `N` is covariant over `'varying` and `T` (as unsafely asserted by the caller),
        // when `T<'varying>` is covariant over `'varying`, so is `N<'varying, T<'varying>>`.
        unsafe impl<'lower, 'upper, $t> $crate::CovariantFamily<'lower, &'upper ()>
        for $($name)::+<$fam_t>
        where
            $t: $crate::CovariantFamily<'lower, &'upper (), Is: 'upper + $($($bound)*)?>,
            $($where_bounds)*
        {
            fn prove_covariance<'long, 'short>(
                long: $crate::RawVarying<'long, 'lower, &'upper (), Self>,
            ) -> $crate::RawVarying<'short, 'lower, &'upper (), Self>
            where
                &'upper (): 'long,
                'long: 'short,
                'short: 'lower,
            {
                long.cast()
            }
        }

        // `N<'varying, T<'varying>>` is never contravariant over `'varying`.
        // It's always at best covariant, never bivariant.
    };
}

/// This should not be called directly (outside of this module). Do so at your own risk (including
/// risk of UB).
#[doc(hidden)]
#[macro_export]
macro_rules! __impl_varying_ref_mut_wrapper {
    {
        $t:ident $((Is: $($bound:tt)*))?
        for $($name:ident)::+<$fam_t:ident>
        as $($varying_name:ident)::+<'_, $varying_t:ident>
        where {$($where_bounds:tt)*}
    } => {
        $crate::__impl_varying_with_lifetime! {
            $t $((Is: $($bound)*))?
            for $($name)::+<$fam_t>
            as $($varying_name)::+<'_, $varying_t>
            where {$($where_bounds)*}
        }

        // SAFETY:
        // `CovariantFamily::prove_covariance` is implemented with the function body `{ long }`,
        // so this implementation is certainly sound.
        unsafe impl<'lower, 'upper, $t> $crate::CovariantFamily<'lower, &'upper ()>
        for $($name)::+<$fam_t>
        where
            $t: $crate::UnvaryingFamily<'lower, &'upper (), Is: 'upper + $($($bound)*)?>,
            $($where_bounds)*
        {
            fn prove_covariance<'long, 'short>(
                long: $crate::RawVarying<'long, 'lower, &'upper (), Self>,
            ) -> $crate::RawVarying<'short, 'lower, &'upper (), Self>
            where
                &'upper (): 'long,
                'long: 'short,
                'short: 'lower,
            {
                long
            }
        }

        // `N<'varying, T<'varying>>` is never contravariant over `'varying`.
        // It's always at best covariant, never bivariant.
    };
}

/// This should not be called directly (outside of this module). Do so at your own risk (including
/// risk of UB).
#[doc(hidden)]
#[macro_export]
macro_rules! __impl_varying_with_lifetime {
        {
        $t:ident $((Is: $($bound:tt)*))?
        for $($name:ident)::+<$fam_t:ident>
        as $($varying_name:ident)::+<'_, $varying_t:ident>
        where {$($where_bounds:tt)*}
    } => {
        // SAFETY:
        // - We can assume (by the safety condition of `WithLifetime`)
        //   that `T::Is` does not use `'lower` or `&'upper ()`,
        //   and the caller can't access those types (thanks to macro hygiene),
        //   so `N<'varying, T::Is>` does not use `'lower` or `&'upper ()`.
        // - The caller asserts with `#[unsafe(not_a_foreign_fundamental_type)]` that they are
        //   permitted to impl `WithLifetime` for the family type.
        unsafe impl<'varying, 'lower, 'upper, $t>
            $crate::WithLifetime<'varying, 'lower, &'upper ()>
        for $($name)::+<$fam_t>
        where
            $t: $crate::WithLifetime<'varying, 'lower, &'upper (), Is: 'upper + $($($bound)*)?>,
            $($where_bounds)*
        {
            type Is = $($varying_name)::+<'varying, $varying_t::Is>;
        }

        // SAFETY: See the first bullet point in the safety comment of the `WithLifetime` impl.
        unsafe impl<'varying, 'lower, Upper, T>
            $crate::ChangeBounds<
                'varying, 'lower, Upper,
                $($varying_name)::+<'varying, $varying_t::Is>,
            >
        for $($name)::+<$fam_t>
        where
            Upper: $crate::UpperBound,
            $t: $crate::WithLifetime<'varying, 'lower, Upper $(, Is: $($bound)*)?>,
            $($where_bounds)*
        {
            fn prove_equal<'other_lower, OtherUpper>(
                varying: $crate::RawMutVarying<'varying, 'other_lower, OtherUpper, Self>,
            ) -> *mut *mut $($varying_name)::+<'varying, $varying_t::Is>
            where
                Self: $crate::WithLifetime<'varying, 'other_lower, OtherUpper>,
                OtherUpper: $crate::UpperBound,
            {
                varying.cast()
            }
        }
    };
}

/// Denotes that `#[unsafe(covariant)]` or `#[unsafe(contravariant)]` may be used.
///
/// # Safety
/// No actual safety condition. Simply used to trigger the `unsafe_code` lint.
#[doc(hidden)]
pub const unsafe fn assert_variance() {}

/// Denotes that `#[unsafe(not_a_foreign_fundamental_type)]` was used.
///
/// # Safety
/// No actual safety condition. Simply used to trigger the `unsafe_code` lint.
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
