//! Implementations for `fn(..Args) -> R` for arities 0-12.

#![expect(unsafe_code, reason = "assert variance and permission to impl traits for `core` types")]


// ================================================================
//  fn(T1, .., Tn) -> R    (for argument arities 0..=12)
// ================================================================

macro_rules! fn_family {
    (fn($($Ti:ident),*) -> $R:ident) => {
        const _: () = {
            /// Get `fn($($Ti),*) -> $R` into the standard shape expected by `generic_wrapper`.
            type Fn<$R, $($Ti,)*> = fn($($Ti,)*) -> $R;

            $crate::generic_wrapper! {
                impl<{
                    // SAFETY: `fn($($Ti),*) -> $R` is covariant over `$R`.
                    #[unsafe(covariant)] $R,
                    // SAFETY: `fn($($Ti),*) -> $R` is contravariant over each `$Ti`.
                    $(
                        #[unsafe(contravariant)] $Ti,
                    )*
                }> ([Co] + [Contra])variantFamily<'_, _>
                // SAFETY: `variance-family` is allowed to implement traits for this type in `core`.
                for #[unsafe(not_a_foreign_fundamental_type)] Fn<..>
                where {
                    $R: ?Sized,
                    $(
                        $Ti: ?Sized,
                    )*
                }
            }
        };
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
