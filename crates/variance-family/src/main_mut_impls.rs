//! Implementations for `&'a mut T`, `&'varying mut T` (as [`VaryingRefMut<T>`]), and `*mut T`.

#![expect(unsafe_code, reason = "allow unsafe code to rely on impls of lifetime family traits")]

use core::marker::PhantomData;

use crate::phantom_zst_methods;
use crate::traits::{
    ChangeBounds, ContravariantFamily, CovariantFamily, RawMutVarying, RawVarying, UnvaryingFamily,
    UpperBound, WithLifetime,
};


// Note: in below safety comments, "is covariant over" or "is contravariant over" means, more
// precisely, "is sound to covariantly (or contravariantly) cast with respect to". That is,
// manually-proven variance (and manually-proven soundness of casts) is the relevant concern,
// not compiler-assigned variance (and compiler-proven soundness of casts).

// ================================================================
//  &'a mut T
// ================================================================

// Safety summary:
// - `&'a mut U` is bivariant over `'varying` (as it's entirely unused). Below, `T<'varying>`
//   families are used which implement `UnvaryingFamily`, making them equivalent to `&'a mut U`
//   for some type `U`. Unsafe transmutes aren't even needed.

// We cannot use the `unvarying!` macro, since we need to place bounds on `T` involving
// `'lower` and `Upper`.

// SAFETY:
// - We can assume (by the safety condition of `WithLifetime`)
//   that `T::Is` does not use `'lower` or `Upper`,
//   so `&'a mut T::Is` does not use `'lower` or `Upper`.
// - `variance-family` is allowed to implement traits for this `#[fundamental]` type in `core`.
unsafe impl<'a, 'varying, 'lower, Upper, T> WithLifetime<'varying, 'lower, Upper> for &'a mut T
where
    Upper: UpperBound,
    T: ?Sized + WithLifetime<'varying, 'lower, Upper, Is: 'a>,
{
    type Is = &'a mut T::Is;
}

// SAFETY:
// We can assume (by the safety condition of `WithLifetime`)
// that `T::Is` does not use `'lower` or `Upper`,
// so `&'a mut T::Is` does not use `'lower` or `Upper`.
unsafe impl<'a, 'varying, 'lower, Upper, T> ChangeBounds<'varying, 'lower, Upper, &'a mut T::Is>
for &'a mut T
where
    Upper: UpperBound,
    T: ?Sized + WithLifetime<'varying, 'lower, Upper>,
{
    fn prove_equal<'other_lower, OtherUpper>(
        varying: RawMutVarying<'varying, 'other_lower, OtherUpper, Self>,
    ) -> *mut *mut &'a mut T::Is
    where
        Self: WithLifetime<'varying, 'other_lower, OtherUpper>,
        OtherUpper: UpperBound,
    {
       varying.cast()
    }
}

// SAFETY:
// `CovariantFamily::prove_covariance` is implemented with the function body `{ long }`,
// so this implementation is certainly sound.
unsafe impl<'a, 'lower, Upper, T> CovariantFamily<'lower, Upper> for &'a mut T
where
    Upper: UpperBound,
    T: ?Sized + UnvaryingFamily<'lower, Upper, Is: 'a>,
{
    fn prove_covariance<'long, 'short>(
        long: RawVarying<'long, 'lower, Upper, Self>,
    ) -> RawVarying<'short, 'lower, Upper, Self>
    where
        Upper: 'long,
        'long: 'short,
        'short: 'lower,
    {
        long
    }
}

// SAFETY:
// `ContravariantFamily::prove_contravariance` is implemented with the function body `{ short }`,
// so this implementation is certainly sound.
unsafe impl<'a, 'lower, Upper, T> ContravariantFamily<'lower, Upper> for &'a mut T
where
    Upper: UpperBound,
    T: ?Sized + UnvaryingFamily<'lower, Upper, Is: 'a>,
{
    fn prove_contravariance<'short, 'long>(
        short: RawVarying<'short, 'lower, Upper, Self>,
    ) -> RawVarying<'long, 'lower, Upper, Self>
    where
        Upper: 'long,
        'long: 'short,
        'short: 'lower,
    {
        short
    }
}


// ================================================================
//  &'varying mut T    (VaryingRefMut<T>)
// ================================================================

// Safety summary:
// - `&'varying mut U` is covariant over `'varying`. Below, `T<'varying>` families are used which
//   implement `UnvaryingFamily`, making them equivalent to `&'a mut U` for some type `U`.
//   Unsafe transmutes aren't even needed.
// - `&'varying mut T<'varying>` is never contravariant over `'varying`.

/// The `&'varying mut T<'varying>` lifetime family.
///
/// If `T<'varying>` does not actually use `'varying` at all (making it some fixed type `U`
/// regardless of `'varying`), then `&'varying mut T<'varying>` is covariant over `'varying`.
///
/// This lifetime family is never contravariant over `'varying`.
///
/// Note that this type itself is just a marker ZST for the family.
///
/// # Limitations
/// Due to current limitations of the trait solver, `VaryingRefMut<T>` requires that
/// `T<'varying>` outlives the `'upper` bound, instead of requiring that `T<'varying>` outlives
/// `'varying`.
///
/// As a result, if you want a more complicated lifetime family
/// (such as `&'varying mut Cow<'varying, [u8]>`),
/// you will have to define your own lifetime family type instead of composing
/// `VaryingRefMut<_>` with other lifetime family types.
pub struct VaryingRefMut<T: ?Sized>(PhantomData<fn(*mut T)>);

phantom_zst_methods!(impl<{T: ?Sized}> _ for VaryingRefMut<{T}>);

// SAFETY:
// - We can assume (by the safety condition of `WithLifetime`)
//   that `T::Is` does not use `'lower` or `&'upper ()`,
//   so `&'varying T::Is` does not use `'lower` or `&'upper ()`.
// - This is a local type.
unsafe impl<'varying, 'lower, 'upper, T> WithLifetime<'varying, 'lower, &'upper ()>
for VaryingRefMut<T>
where
    T: ?Sized + WithLifetime<'varying, 'lower, &'upper (), Is: 'upper>,
{
    type Is = &'varying T::Is;
}

// SAFETY:
// We can assume (by the safety condition of `WithLifetime`)
// that `T::Is` does not use `'lower` or `Upper`,
// so `&'a T::Is` does not use `'lower` or `Upper`.
unsafe impl<'varying, 'lower, Upper, T> ChangeBounds<'varying, 'lower, Upper, &'varying T::Is>
for VaryingRefMut<T>
where
    Upper: UpperBound,
    T: ?Sized + WithLifetime<'varying, 'lower, Upper>,
{
    fn prove_equal<'other_lower, OtherUpper>(
        varying: RawMutVarying<'varying, 'other_lower, OtherUpper, Self>,
    ) -> *mut *mut &'varying T::Is
    where
        Self: WithLifetime<'varying, 'other_lower, OtherUpper>,
        OtherUpper: UpperBound,
    {
       varying.cast()
    }
}

// SAFETY:
// `CovariantFamily::prove_covariance` is implemented with the function body `{ long }`,
// so this implementation is certainly sound.
unsafe impl<'lower, 'upper, T> CovariantFamily<'lower, &'upper ()> for VaryingRefMut<T>
where
    T: ?Sized + UnvaryingFamily<'lower, &'upper (), Is: 'upper>,
{
    fn prove_covariance<'long, 'short>(
        long: RawVarying<'long, 'lower, &'upper (), Self>,
    ) -> RawVarying<'short, 'lower, &'upper (), Self>
    where
        &'upper (): 'long,
        'long: 'short,
        'short: 'lower,
    {
        long
    }
}

// `&'varying mut T<'varying>` is never contravariant over `'varying`. It's always at best
// covariant, never bivariant.


// ================================================================
//  *mut T
// ================================================================

// Safety summary:
// - `*mut U` is bivariant over `'varying` (as it's entirely unused). Below, `T<'varying>`
//   families are used which implement `UnvaryingFamily`, making them equivalent to `*mut U`
//   for some type `U`.

// SAFETY:
// - We can assume (by the safety condition of `WithLifetime`)
//   that `T::Is` does not use `'lower` or `Upper`,
//   so `*mut T::Is` does not use `'lower` or `Upper`.
// - `variance-family` is allowed to implement traits for this type in `core`.
unsafe impl<'varying, 'lower, Upper, T> WithLifetime<'varying, 'lower, Upper> for *mut T
where
    Upper: UpperBound,
    T: ?Sized + WithLifetime<'varying, 'lower, Upper>,
{
    type Is = *mut T::Is;
}

// SAFETY:
// We can assume (by the safety condition of `WithLifetime`)
// that `T::Is` does not use `'lower` or `Upper`,
// so `*mut T::Is` does not use `'lower` or `Upper`.
unsafe impl<'varying, 'lower, Upper, T> ChangeBounds<'varying, 'lower, Upper, *mut T::Is>
for *mut T
where
    Upper: UpperBound,
    T: ?Sized + WithLifetime<'varying, 'lower, Upper>,
{
    fn prove_equal<'other_lower, OtherUpper>(
        varying: RawMutVarying<'varying, 'other_lower, OtherUpper, Self>,
    ) -> *mut *mut *mut T::Is
    where
        Self: WithLifetime<'varying, 'other_lower, OtherUpper>,
        OtherUpper: UpperBound,
    {
       varying.cast()
    }
}

// SAFETY:
// `CovariantFamily::prove_covariance` is implemented with the function body `{ long }`,
// so this implementation is certainly sound.
unsafe impl<'lower, Upper, T> CovariantFamily<'lower, Upper> for *mut T
where
    Upper: UpperBound,
    T: ?Sized + UnvaryingFamily<'lower, Upper>,
{
    fn prove_covariance<'long, 'short>(
        long: RawVarying<'long, 'lower, Upper, Self>,
    ) -> RawVarying<'short, 'lower, Upper, Self>
    where
        Upper: 'long,
        'long: 'short,
        'short: 'lower,
    {
        long
    }
}

// SAFETY:
// `ContravariantFamily::prove_contravariance` is implemented with the function body `{ short }`,
// so this implementation is certainly sound.
unsafe impl<'lower, Upper, T> ContravariantFamily<'lower, Upper> for *mut T
where
    Upper: UpperBound,
    T: ?Sized + UnvaryingFamily<'lower, Upper>,
{
    fn prove_contravariance<'short, 'long>(
        short: RawVarying<'short, 'lower, Upper, Self>,
    ) -> RawVarying<'long, 'lower, Upper, Self>
    where
        Upper: 'long,
        'long: 'short,
        'short: 'lower,
    {
        short
    }
}
