//! Implementations for `fn(..Args) -> R` for arities 0-12.

#![expect(unsafe_code, reason = "allow unsafe code to rely on impls of lifetime family traits")]

use crate::traits::{
    ChangeBounds, ContravariantFamily, CovariantFamily, UpperBound, RawMutVarying, RawVarying,
    WithLifetime,
};


// Note: in below safety comments, "is covariant over" or "is contravariant over" means, more
// precisely, "is sound to covariantly (or contravariantly) cast with respect to". That is,
// manually-proven variance (and manually-proven soundness of casts) is the relevant concern,
// not compiler-assigned variance (and compiler-proven soundness of casts).

// ================================================================
//  fn(T1, .., Tn) -> R    (for argument arities 0..=12)
// ================================================================

// Safety summary:
// - `fn(T1<'varying, .., Tn<'varying>) -> R<'varying>` is covariant over `'varying` if each
//   `Ti<'varying>` is contravariant over `'varying` and `R<'varying>` is covariant over `'varying`.
// - `fn(T1<'varying, .., Tn<'varying>) -> R<'varying>` is contravariant over `'varying` if each
//   `Ti<'varying>` is covariant over `'varying` and `R<'varying>` is contravariant over `'varying`.

// NOTE: for soundness, this macro should not be exported, even just within this crate.
// It assumes that it is used with *this* crate's traits in scope (with the normal names).
// In particular, the `unsafe impl` could be broken in other environments.
macro_rules! fn_family {
    (fn($($Ti:ident),*) -> $R:ident) => {
        // SAFETY:
        // - We can assume (by the safety condition of `WithLifetime`)
        //   that no `$Ti::Is` nor `$R` uses `'lower` or `Upper`,
        //   so `fn($($Ti::Is),*) -> $R::Is` does not use `'lower` or `Upper`.
        // - `variance-family` is allowed to implement traits for these types in `core`.
        unsafe impl<'varying, 'lower, Upper, $($Ti,)* $R> WithLifetime<'varying, 'lower, Upper>
        for fn($($Ti),*) -> $R
        where
            Upper: UpperBound,
            $(
                $Ti: ?Sized + WithLifetime<'varying, 'lower, Upper>,
            )*
            $R: ?Sized + WithLifetime<'varying, 'lower, Upper>,
        {
            type Is = fn($($Ti::Is),*) -> $R::Is;
        }

        // SAFETY:
        // We can assume (by the safety condition of `WithLifetime`)
        // that no `$Ti::Is` nor `$R` uses `'lower` or `Upper`,
        // so `fn($($Ti::Is),*) -> $R::Is` does not use `'lower` or `Upper`.
        unsafe impl<'varying, 'lower, Upper, $($Ti,)* $R>
            ChangeBounds<'varying, 'lower, Upper, fn($($Ti::Is),*) -> $R::Is>
        for fn($($Ti),*) -> $R
        where
            Upper: UpperBound,
            $(
                $Ti: ?Sized + WithLifetime<'varying, 'lower, Upper>,
            )*
            $R: ?Sized + WithLifetime<'varying, 'lower, Upper>,
        {
            fn prove_equal<'other_lower, OtherUpper>(
                varying: RawMutVarying<'varying, 'other_lower, OtherUpper, Self>,
            ) -> *mut *mut fn($($Ti::Is),*) -> $R::Is
            where
                Self: WithLifetime<'varying, 'other_lower, OtherUpper>,
                OtherUpper: UpperBound,
            {
                varying.cast()
            }
        }

        // SAFETY:
        // When each `Ti<'varying>` is contravariant over `'varying`
        // and `R<'varying>` is covariant over `'varying`,
        // `fn(.., Ti<'varying, ..) -> R<'varying>` is covariant over `'varying`.
        unsafe impl<'lower, Upper, $($Ti,)* $R> CovariantFamily<'lower, Upper>
        for fn($($Ti),*) -> $R
        where
            Upper: UpperBound,
            $(
                $Ti: ?Sized + ContravariantFamily<'lower, Upper>,
            )*
            $R: ?Sized + CovariantFamily<'lower, Upper>,
        {
            fn prove_covariance<'long, 'short>(
                long: RawVarying<'long, 'lower, Upper, Self>,
            ) -> RawVarying<'short, 'lower, Upper, Self>
            where
                Upper: 'long,
                'long: 'short,
                'short: 'lower,
            {
                long.cast()
            }
        }

        // SAFETY:
        // When each `Ti<'varying>` is covariant over `'varying`
        // and `R<'varying>` is contravariant over `'varying`,
        // `fn(.., Ti<'varying, ..) -> R<'varying>` is contravariant over `'varying`.
        unsafe impl<'lower, Upper, $($Ti,)* $R> ContravariantFamily<'lower, Upper>
        for fn($($Ti),*) -> $R
        where
            Upper: UpperBound,
            $(
                $Ti: ?Sized + CovariantFamily<'lower, Upper>,
            )*
            $R: ?Sized + ContravariantFamily<'lower, Upper>,
        {
            fn prove_contravariance<'short, 'long>(
                short: RawVarying<'short, 'lower, Upper, Self>,
            ) -> RawVarying<'long, 'lower, Upper, Self>
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

fn_family!(fn() -> R);
fn_family!(fn(T1) -> R);
fn_family!(fn(T1, T2) -> R);
fn_family!(fn(T1, T2, T3) -> R);
fn_family!(fn(T1, T2, T3, T4) -> R);
fn_family!(fn(T1, T2, T3, T4, T5) -> R);
fn_family!(fn(T1, T2, T3, T4, T5, T6) -> R);
fn_family!(fn(T1, T2, T3, T4, T5, T6, T7) -> R);
fn_family!(fn(T1, T2, T3, T4, T5, T6, T7, T8) -> R);
fn_family!(fn(T1, T2, T3, T4, T5, T6, T7, T8, T9) -> R);
fn_family!(fn(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10) -> R);
fn_family!(fn(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11) -> R);
fn_family!(fn(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12) -> R);
