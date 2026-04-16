use core::marker::PhantomData;

use crate::phantom_zst_methods;
use crate::traits::{
    ChangeBounds, ContravariantFamily, CovariantFamily, RawMutVarying, RawVarying, UpperBound,
    WithLifetime,
};


// Note: in below safety comments, "is covariant over" or "is contravariant over" means, more
// precisely, "is sound to covariantly (or contravariantly) cast with respect to". That is,
// manually-proven variance (and manually-proven soundness of casts) is the relevant concern,
// not compiler-assigned variance (and compiler-proven soundness of casts).

// ================================================================
//  &'a T
// ================================================================

// Safety summary:
// - `&'a T<'varying>` is covariant over `'varying` if `T<'varying>` is covariant over `'varying`.
// - `&'a T<'varying>` is contravariant over `'varying` if `T<'varying>` is contravariant over it.

// SAFETY:
// - We can assume (by the safety condition of `WithLifetime`)
//   that `T::Is` does not use `'lower` or `Upper`,
//   so `&'a T::Is` does not use `'lower` or `Upper`.
// - `variance-family` is allowed to implement traits for this `#[fundamental]` type in `core`.
unsafe impl<'a, 'varying, 'lower, Upper, T> WithLifetime<'varying, 'lower, Upper> for &'a T
where
    Upper: UpperBound,
    T: ?Sized + WithLifetime<'varying, 'lower, Upper>,
    T::Is: 'a,
{
    type Is = &'a T::Is;
}

// SAFETY:
// We can assume (by the safety condition of `WithLifetime`)
// that `T::Is` does not use `'lower` or `Upper`,
// so `&'a T::Is` does not use `'lower` or `Upper`.
unsafe impl<'a, 'varying, 'lower, Upper, T> ChangeBounds<'varying, 'lower, Upper, &'a T::Is>
for &'a T
where
    Upper: UpperBound,
    T: ?Sized + WithLifetime<'varying, 'lower, Upper>,
{
    fn prove_equal<'other_lower, OtherUpper>(
        varying: RawMutVarying<'varying, 'other_lower, OtherUpper, Self>,
    ) -> *mut *mut &'a T::Is
    where
        Self: WithLifetime<'varying, 'other_lower, OtherUpper>,
        OtherUpper: UpperBound,
    {
       varying.cast()
    }
}

// SAFETY:
// When `T<'varying>` is covariant over `'varying`, so is `&'a T<'varying>`.
unsafe impl<'a, 'lower, Upper, T> CovariantFamily<'lower, Upper> for &'a T
where
    Upper: UpperBound,
    T: ?Sized + CovariantFamily<'lower, Upper, Is: 'a>,
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
// When `T<'varying>` is contravariant over `'varying`, so is `&'a T<'varying>`.
unsafe impl<'a, 'lower, Upper, T> ContravariantFamily<'lower, Upper> for &'a T
where
    Upper: UpperBound,
    T: ?Sized + ContravariantFamily<'lower, Upper, Is: 'a>,
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


// ================================================================
//  &'varying T    (VaryingRef<T>)
// ================================================================

// Safety summary:
// - `&'varying T<'varying>` is covariant over `'varying` if `T<'varying>` is covariant over it.
// - `&'varying T<'varying>` is never contravariant over `'varying`.

/// The `&'varying T<'varying>` lifetime family.
///
/// If `T<'varying>` is covariant over `'varying`, then `&'varying T<'varying>` is covariant
/// over `'varying`.
///
/// This lifetime family is never contravariant over `'varying`.
///
/// Note that this type itself is just a marker ZST for the family.
///
/// # Limitations
/// Due to current limitations of the trait solver, `VaryingRef<T>` requires that
/// `T<'varying>` outlives the `'upper` bound, instead of requiring that `T<'varying>` outlives
/// `'varying`.
///
/// As a result, if you want a more complicated lifetime family
/// (such as `&'varying Cow<'varying, [u8]>`),
/// you will have to define your own lifetime family type instead of composing `VaryingRef<_>` with
/// other lifetime family types.
pub struct VaryingRef<T: ?Sized>(PhantomData<fn() -> T>);

phantom_zst_methods!(impl<{T: ?Sized}> _ for VaryingRef<{T}>);

// SAFETY:
// - We can assume (by the safety condition of `WithLifetime`)
//   that `T::Is` does not use `'lower` or `&'upper ()`,
//   so `&'varying T::Is` does not use `'lower` or `&'upper ()`.
// - This is a local type.
unsafe impl<'varying, 'lower, 'upper, T> WithLifetime<'varying, 'lower, &'upper ()>
for VaryingRef<T>
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
for VaryingRef<T>
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
// When `T<'varying>` is covariant over `'varying`, so is `&'varying T<'varying>`.
unsafe impl<'lower, 'upper, T> CovariantFamily<'lower, &'upper ()> for VaryingRef<T>
where
    T: ?Sized + CovariantFamily<'lower, &'upper (), Is: 'upper>,
{
    fn prove_covariance<'long, 'short>(
        long: RawVarying<'long, 'lower, &'upper (), Self>,
    ) -> RawVarying<'short, 'lower, &'upper (), Self>
    where
        &'upper (): 'long,
        'long: 'short,
        'short: 'lower,
    {
        long.cast()
    }
}

// `&'varying T<'varying>` is never contravariant over `'varying`. It's always at best covariant,
// never bivariant.


// ================================================================
//  *const T
// ================================================================

// Safety summary:
// - `*const T<'varying>` is covariant over `'varying` if `T<'varying>` is covariant over it.
// - `*const T<'varying>` is contravariant over it if `T<'varying>` is contravariant over it.

// SAFETY:
// - We can assume (by the safety condition of `WithLifetime`)
//   that `T::Is` does not use `'lower` or `Upper`,
//   so `*const T::Is` does not use `'lower` or `Upper`.
// - `variance-family` is allowed to implement traits for this type in `core`.
unsafe impl<'varying, 'lower, Upper, T> WithLifetime<'varying, 'lower, Upper> for *const T
where
    Upper: UpperBound,
    T: ?Sized + WithLifetime<'varying, 'lower, Upper>,
{
    type Is = *const T::Is;
}

// SAFETY:
// We can assume (by the safety condition of `WithLifetime`)
// that `T::Is` does not use `'lower` or `Upper`,
// so `*const T::Is` does not use `'lower` or `Upper`.
unsafe impl<'varying, 'lower, Upper, T> ChangeBounds<'varying, 'lower, Upper, *const T::Is>
for *const T
where
    Upper: UpperBound,
    T: ?Sized + WithLifetime<'varying, 'lower, Upper>,
{
    fn prove_equal<'other_lower, OtherUpper>(
        varying: RawMutVarying<'varying, 'other_lower, OtherUpper, Self>,
    ) -> *mut *mut *const T::Is
    where
        Self: WithLifetime<'varying, 'other_lower, OtherUpper>,
        OtherUpper: UpperBound,
    {
       varying.cast()
    }
}

// SAFETY:
// When `T<'varying>` is covariant over `'varying`, so is `*const T<'varying>`.
unsafe impl<'lower, Upper, T> CovariantFamily<'lower, Upper> for *const T
where
    Upper: UpperBound,
    T: ?Sized + CovariantFamily<'lower, Upper>,
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
// When `T<'varying>` is contravariant over `'varying`, so is `*const T<'varying>`.
unsafe impl<'lower, Upper, T> ContravariantFamily<'lower, Upper> for *const T
where
    Upper: UpperBound,
    T: ?Sized + ContravariantFamily<'lower, Upper>,
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
