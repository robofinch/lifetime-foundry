use core::mem::transmute;

use crate::traits::{ContravariantFamily, CovariantFamily, UpperBound, Varying, WithLifetime};


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
        impl<'varying, 'lower, Upper, $($Ti,)* $R> WithLifetime<'varying, 'lower, Upper>
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
        // - If `Self::covariant_assertions()` does not panic,
        //   then `Self<'varying>` is covariant over `'varying`.
        //
        //   The former implies that each `Ti::contravariant_assertions()` and
        //   `R::covariant_assertions()` do not panic,
        //   in which case each `Ti<'varying>` is contravariant over `'varying`
        //   and `R<'varying>` is covariant over `'varying`,
        //   implying that `fn(.., Ti<'varying, ..) -> R<'varying>` is covariant over `'varying`.
        //
        // - No assertions are included other than those in `Self::covariant_assertions()`.
        // - The implementation safety requirements of `shorten` and `shorten_ref` are met.
        unsafe impl<'lower, Upper, $($Ti,)* $R> CovariantFamily<'lower, Upper>
        for fn($($Ti),*) -> $R
        where
            Upper: UpperBound,
            $(
                $Ti: ?Sized + ContravariantFamily<'lower, Upper>,
            )*
            $R: ?Sized + CovariantFamily<'lower, Upper>,
        {
            #[inline]
            fn covariant_assertions() {
                $(
                    $Ti::contravariant_assertions();
                )*
                $R::covariant_assertions();
            }

            #[inline]
            fn shorten<'l, 's>(
                long: Varying<'l, 'lower, Upper, Self>,
            ) -> Varying<'s, 'lower, Upper, Self>
            where
                Upper: 'l,
                'l: 's,
                's: 'lower,
                for<'varying> Varying<'varying, 'lower, Upper, Self>: Sized,
            {
                #![expect(
                    clippy::unnecessary_safety_comment,
                    reason = "implementation safety of method",
                )]
                // Implementation safety: this is a covariant cast with some assertions.
                // There are no possible sources of panics other than the
                // `Self::covariant_assertions()` call.
                Self::covariant_assertions();

                let src: fn($(Varying<'l, 'lower, Upper, $Ti>),*)
                    -> Varying<'l, 'lower, Upper, $R>
                    = long;

                // SAFETY: when the `dst` function is used, its arguments' `'varying` lifetimes
                // are effectively lengthened from `'s` to `'l` (`'l` being the arguments'
                // `'varying` lifetimes in `src`). Those are contravariant casts.
                // The underlying function would still return a value of the original type,
                // with `'varying = 'l`, which is shortened to `'s`. That's a covariant cast.
                // We called `Ti::contravariant_assertions()` and `R::covariant_assertions()`,
                // so those casts are sound.
                // Also see https://github.com/rust-lang/rust/issues/140803; since the types
                // are parameterized only by lifetimes (and we can assume that specializing on
                // `'static` is unsound), this transmute is not erroneous with CFI.
                #[expect(clippy::undocumented_unsafe_blocks, reason = "false positive")]
                let dst: fn($(Varying<'s, 'lower, Upper, $Ti>),*)
                    -> Varying<'s, 'lower, Upper, $R>
                    = unsafe { transmute(src) };

                dst
            }

            #[inline]
            fn shorten_ref<'l, 's, 'r>(
                long: &'r Varying<'l, 'lower, Upper, Self>,
            ) -> &'r Varying<'s, 'lower, Upper, Self>
            where
                Upper: 'l,
                'l: 's,
                's: 'lower,
                Varying<'l, 'lower, Upper, Self>: 'r,
                Varying<'s, 'lower, Upper, Self>: 'r,
            {
                #![expect(
                    clippy::unnecessary_safety_comment,
                    reason = "implementation safety of method",
                )]
                // Implementation safety: this is a covariant cast with some assertions.
                // There are no possible sources of panics other than the
                // `Self::covariant_assertions()` call.
                Self::covariant_assertions();

                let src: &'r fn($(Varying<'l, 'lower, Upper, $Ti>),*)
                    -> Varying<'l, 'lower, Upper, $R>
                    = long;

                // SAFETY: when the `dst` function is used, its arguments' `'varying` lifetimes
                // are effectively lengthened from `'s` to `'l` (`'l` being the arguments'
                // `'varying` lifetimes in `src`). Those are contravariant casts.
                // The underlying function would still return a value of the original type,
                // with `'varying = 'l`, which is shortened to `'s`. That's a covariant cast.
                // We called `Ti::contravariant_assertions()` and `R::covariant_assertions()`,
                // so those casts are sound.
                // Also see https://github.com/rust-lang/rust/issues/140803; since the types
                // are parameterized only by lifetimes (and we can assume that specializing on
                // `'static` is unsound), this transmute is not erroneous with CFI.
                #[expect(clippy::undocumented_unsafe_blocks, reason = "false positive")]
                let dst: &'r fn($(Varying<'s, 'lower, Upper, $Ti>),*)
                    -> Varying<'s, 'lower, Upper, $R>
                    = unsafe { transmute(src) };

                dst
            }
        }

        // SAFETY:
        // - If `Self::contravariant_assertions()` does not panic,
        //   then `Self<'varying>` is contravariant over `'varying`.
        //
        //   The former implies that each `Ti::covariant_assertions()` and
        //   `R::contravariant_assertions()` do not panic,
        //   in which case each `Ti<'varying>` is covariant over `'varying`
        //   and `R<'varying>` is contravariant over `'varying`,
        //   implying that `fn(.., Ti<'varying, ..) -> R<'varying>` is contravariant over it.
        //
        // - No assertions are included other than those in `Self::contravariant_assertions()`.
        // - The implementation safety requirements of `lengthen` and `lengthen_ref` are met.
        unsafe impl<'lower, Upper, $($Ti,)* $R> ContravariantFamily<'lower, Upper>
        for fn($($Ti),*) -> $R
        where
            Upper: UpperBound,
            $(
                $Ti: ?Sized + CovariantFamily<'lower, Upper>,
            )*
            $R: ?Sized + ContravariantFamily<'lower, Upper>,
        {
            #[inline]
            fn contravariant_assertions() {
                $(
                    $Ti::covariant_assertions();
                )*
                $R::contravariant_assertions();
            }

            #[inline]
            fn lengthen<'s, 'l>(
                short: Varying<'s, 'lower, Upper, Self>,
            ) -> Varying<'l, 'lower, Upper, Self>
            where
                Upper: 'l,
                'l: 's,
                's: 'lower,
                for<'varying> Varying<'varying, 'lower, Upper, Self>: Sized,
            {
                #![expect(
                    clippy::unnecessary_safety_comment,
                    reason = "implementation safety of method",
                )]
                // Implementation safety: this is a contravariant cast with some assertions.
                // There are no possible sources of panics other than the
                // `Self::contravariant_assertions()` call.
                Self::contravariant_assertions();

                let src: fn($(Varying<'s, 'lower, Upper, $Ti>),*)
                    -> Varying<'s, 'lower, Upper, $R>
                    = short;

                // SAFETY: when the `dst` function is used, its arguments' `'varying` lifetimes
                // are effectively shortened from `'l` to `'s` (`'s` being the arguments'
                // `'varying` lifetimes in `src`). Those are covariant casts.
                // The underlying function would still return a value of the original type,
                // with `'varying = 's`, which is lengthened to `'l`. That's a contravariant cast.
                // We called `Ti::covariant_assertions()` and `R::contravariant_assertions()`,
                // so those casts are sound.
                // Also see https://github.com/rust-lang/rust/issues/140803; since the types
                // are parameterized only by lifetimes (and we can assume that specializing on
                // `'static` is unsound), this transmute is not erroneous with CFI.
                #[expect(clippy::undocumented_unsafe_blocks, reason = "false positive")]
                let dst: fn($(Varying<'l, 'lower, Upper, $Ti>),*)
                    -> Varying<'l, 'lower, Upper, $R>
                    = unsafe { transmute(src) };

                dst
            }

            #[inline]
            fn lengthen_ref<'s, 'l, 'r>(
                short: &'r Varying<'s, 'lower, Upper, Self>,
            ) -> &'r Varying<'l, 'lower, Upper, Self>
            where
                Upper: 'l,
                'l: 's,
                's: 'lower,
                Varying<'l, 'lower, Upper, Self>: 'r,
                Varying<'s, 'lower, Upper, Self>: 'r,
            {
                #![expect(
                    clippy::unnecessary_safety_comment,
                    reason = "implementation safety of method",
                )]
                // Implementation safety: this is a contravariant cast with some assertions.
                // There are no possible sources of panics other than the
                // `Self::contravariant_assertions()` call.
                Self::contravariant_assertions();

                let src: &'r fn($(Varying<'s, 'lower, Upper, $Ti>),*)
                    -> Varying<'s, 'lower, Upper, $R>
                    = short;

                // SAFETY: when the `dst` function is used, its arguments' `'varying` lifetimes
                // are effectively shortened from `'l` to `'s` (`'s` being the arguments'
                // `'varying` lifetimes in `src`). Those are covariant casts.
                // The underlying function would still return a value of the original type,
                // with `'varying = 's`, which is lengthened to `'l`. That's a contravariant cast.
                // We called `Ti::covariant_assertions()` and `R::contravariant_assertions()`,
                // so those casts are sound.
                // Also see https://github.com/rust-lang/rust/issues/140803; since the types
                // are parameterized only by lifetimes (and we can assume that specializing on
                // `'static` is unsound), this transmute is not erroneous with CFI.
                #[expect(clippy::undocumented_unsafe_blocks, reason = "false positive")]
                let dst: &'r fn($(Varying<'l, 'lower, Upper, $Ti>),*)
                    -> Varying<'l, 'lower, Upper, $R>
                    = unsafe { transmute(src) };

                dst
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
