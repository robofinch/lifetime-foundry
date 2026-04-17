//! Implementations for:
//!
//! - `[T]`,
//! - `[T; N]`,
//! - `(T1, ..., Tn)` of arities 0-12,
//! - various primitives (`bool`, `char`, `f32`, `f64`, `i{N}`, `u{N}`, `str`)
//! - `cell::{Cell, Ref, RefCell, RefMut}`,
//! - `option::Option`,
//! - `pin::Pin`,
//! - `result::Result`.

#![expect(unsafe_code, reason = "assert variance and permission to impl traits for `core` types")]

use core::marker::PhantomData;

use crate::{
    generic_wrapper, macros::atomic_types, phantom_zst_methods, varying_ref_mut_wrapper,
    varying_ref_wrapper,
};


// ================================================================
//  [T]
// ================================================================

/// Get `[T]` into the standard shape expected by `generic_wrapper`.
type Slice<T> = [T];

generic_wrapper! {
    // SAFETY: `[T]` is covariant over `T`.
    impl<{#[unsafe(covariant)] T (Is: Sized)}> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] Slice<..>
}


// ================================================================
//  [T; N]
// ================================================================

/// Get `[T; N]` into the standard shape expected by `generic_wrapper`.
type Array<T, const N: usize> = [T; N];

generic_wrapper! {
    impl<{
        // SAFETY: `[T; N]` is covariant over `T`.
        #[unsafe(covariant)] T (Is: Sized),
        const N: usize,
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] Array<..>
}


// ================================================================
//  (T1, .., Tn)    (for argument arities 1..=12)
// ================================================================

macro_rules! tuple_family {
    ($Tlast:ident $(,)? $($Ti:ident),*) => {
        const _: () = {
            /// Get `($($Ti,)* $Tlast)` into the standard shape expected by `generic_wrapper`.
            type Tuple<$($Ti,)* $Tlast> = ($($Ti,)* $Tlast,);

            $crate::generic_wrapper! {
                impl<{
                    // SAFETY: `($($Ti,)* $Tlast)` is covariant over each `$Ti`.
                    $(
                        #[unsafe(covariant)] $Ti (Is: Sized),
                    )*
                    // SAFETY: `($($Ti,)* $Tlast)` is covariant over `$Tlast`.
                    #[unsafe(covariant)] $Tlast,
                }> ([Co] + [Contra])variantFamily<'_, _>
                // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
                for #[unsafe(not_a_foreign_fundamental_type)] Tuple<..>
                where {
                    $Tlast: ?Sized,
                }
            }
        };
    };
}

tuple_family!(T1);
tuple_family!(T2, T1);
tuple_family!(T3, T1, T2);
tuple_family!(T4, T1, T2, T3);
tuple_family!(T5, T1, T2, T3, T4);
tuple_family!(T6, T1, T2, T3, T4, T5);
tuple_family!(T7, T1, T2, T3, T4, T5, T6);
tuple_family!(T8, T1, T2, T3, T4, T5, T6, T7);
tuple_family!(T9, T1, T2, T3, T4, T5, T6, T7, T8);
tuple_family!(T10, T1, T2, T3, T4, T5, T6, T7, T8, T9);
tuple_family!(T11, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
tuple_family!(T12, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);


// ================================================================
//  `()`, `bool`, `char`, `f32`, `f64`, `i{N}`, `u{N}`, `str`
// ================================================================

atomic_types!(
    (), bool, char, f32, f64,
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    str,
);


// ================================================================
//  `cell::{Cell<T>, RefCell<T>}`,
//  `option::Option<T>`, `pin::Pin<T>`, `result::Result<T, E>`.
// ================================================================

generic_wrapper! {
    impl<{#[unvarying] T}> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] core::cell::Cell<..>
    where {T: ?Sized}
}

generic_wrapper! {
    impl<{#[unvarying] T}> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] core::cell::RefCell<..>
    where {T: ?Sized}
}

generic_wrapper! {
    impl<{
        // SAFETY: `Option<T>` is covariant over `T`.
        #[unsafe(covariant)] T (Is: Sized),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] core::option::Option<..>
}

generic_wrapper! {
    impl<{
        // SAFETY: `Pin<Ptr>` is covariant over `Ptr`.
        #[unsafe(covariant)] Ptr (Is: Sized),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] core::pin::Pin<..>
}

generic_wrapper! {
    impl<{
        // SAFETY: `Result<T, E>` is covariant over `T`.
        #[unsafe(covariant)] T (Is: Sized),
        // SAFETY: `Result<T, E>` is covariant over `E`.
        #[unsafe(covariant)] E (Is: Sized),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] core::result::Result<..>
}


// ================================================================
//  `cell::Ref<'a, T>`
// ================================================================

// Safety summary:
// - `Ref<'a, T<'varying>>` is covariant over `'varying`
//   if `T<'varying>` is covariant over `'varying`.
// - `Ref<'a, T<'varying>>` is contravariant over `'varying`
//   if `T<'varying>` is contravariant over it.

generic_wrapper! {
    impl<{
        'a,
        // SAFETY: `&'a T` is covariant over `T``.
        #[unsafe(covariant)] T (Is: 'a),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] core::cell::Ref<..>
    where {T: ?Sized}
}


// ================================================================
//  `cell::Ref<'varying, T>`    (VaryingCellRef<T>)
// ================================================================

// Safety summary:
// - `cell::Ref<'varying, T>` is covariant over `'varying` if `T<'varying>` is covariant over it.
// - `cell::Ref<'varying, T>` is never contravariant over `'varying`.

/// The `cell::Ref<'varying, T>` lifetime family.
///
/// If `T<'varying>` is covariant over `'varying`, then `cell::Ref<'varying, T>` is covariant
/// over `'varying`.
///
/// This lifetime family is never contravariant over `'varying`.
///
/// Note that this type itself is just a marker ZST for the family.
///
/// # Limitations
/// Due to current limitations of the trait solver, `VaryingCellRef<T>` requires that
/// `T<'varying>` outlives the `'upper` bound, instead of requiring that `T<'varying>` outlives
/// `'varying`.
///
/// As a result, if you want a more complicated lifetime family, you may have to define your own
/// lifetime family type instead of composing `VaryingCellRef<_>` with other lifetime family types.
pub struct VaryingCellRef<T: ?Sized>(PhantomData<fn() -> T>);

phantom_zst_methods!(impl<{T: ?Sized}> _ for VaryingCellRef<{T}>);

varying_ref_wrapper! {
    impl<T> CovariantFamily<'_, _>
    // SAFETY: `VaryingRef` is defined in this crate.
    for #[unsafe(not_a_foreign_fundamental_type)] VaryingCellRef<T>
    // SAFETY: `cell::Ref<'varying, T>` is covariant over both `'varying` and `T`.
    as core::cell::Ref<#[unsafe(covariant)] '_, #[unsafe(covariant)] T>
    where {T: ?Sized}
}


// ================================================================
//  `cell::RefMut<'a, T>`
// ================================================================

// Safety summary:
// - `RefMut<'a, mut U>` is bivariant over `'varying` (as it's entirely unused).
//   Below, `T<'varying>` families are used which implement `UnvaryingFamily`, making them
//   equivalent to `RefMut<'a, mut U>` for some type `U`. Unsafe transmutes aren't even needed.

generic_wrapper! {
    impl<{'a, #[unvarying] T (Is: 'a)}> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
    for #[unsafe(not_a_foreign_fundamental_type)] core::cell::RefMut<..>
    where {T: ?Sized}
}


// ================================================================
//  `cell::RefMut<'varying, T>`    (VaryingCellRefMut<T>)
// ================================================================

// Safety summary:
// - `RefMut<'varying, U>` is covariant over `'varying`. Below, `T<'varying>` families are used
//   which implement `UnvaryingFamily`, making them equivalent to `RefMut<'varying, U>` for some
//   type `U`. Unsafe transmutes aren't even needed.
// - `cell::RefMut<'varying, T>` is never contravariant over `'varying`.

/// The `cell::RefMut<'varying, T>` lifetime family.
///
/// If `T<'varying>` does not actually use `'varying` at all (making it some fixed type `U`
/// regardless of `'varying`), then `cell::RefMut<'varying, T>` is covariant over `'varying`.
///
/// This lifetime family is never contravariant over `'varying`.
///
/// Note that this type itself is just a marker ZST for the family.
///
/// # Limitations
/// Due to current limitations of the trait solver, `VaryingCellRefMut<T>` requires that
/// `T<'varying>` outlives the `'upper` bound, instead of requiring that `T<'varying>` outlives
/// `'varying`.
///
/// As a result, if you want a more complicated lifetime family, you may have to define your own
/// lifetime family type instead of composing `VaryingCellRefMut<_>` with other lifetime family
/// types.
pub struct VaryingCellRefMut<T: ?Sized>(PhantomData<fn(*mut T)>);

phantom_zst_methods!(impl<{T: ?Sized}> _ for VaryingCellRefMut<{T}>);

varying_ref_mut_wrapper! {
    impl<T> (Co+Contra)variantFamily<'_, _>
    // SAFETY: `VaryingCellRefMut` is defined in this crate.
    for #[unsafe(not_a_foreign_fundamental_type)] VaryingCellRefMut<T>
    as core::cell::RefMut<'_, #[unvarying] T>
    where {T: ?Sized}
}
