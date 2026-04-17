//! [`shorten`], [`lengthen`], [`shorten_lend`], [`change_bounds_from`],
//! and [`change_bounds_into`] functions.
//!
//! These are useful for consumers of lifetime families.

#![expect(unsafe_code, reason = "unsafely rely on the lifetime family traits")]

use core::mem::{ManuallyDrop, transmute, transmute_copy};

use crate::traits::{
    ContravariantFamily, CovariantFamily, Lend, LendFamily, LifetimeFamily, UpperBound, Varying,
    WithLifetime,
};


/// Shorten a lifetime via a covariant cast.
#[inline]
#[must_use]
pub fn shorten<'short, 'long, 'lower, Upper, T>(
    long: Varying<'long, 'lower, Upper, T>,
) -> Varying<'short, 'lower, Upper, T>
where
    Upper: UpperBound + 'long,
    'long: 'short,
    'short: 'lower,
    T: ?Sized + CovariantFamily<'lower, Upper, Is: Sized>,
{
    // SAFETY: As guaranteed by the unsafe `CovariantFamily` trait, unsafely performing covariant
    // casts on `T<'varying>` is sound. Shortening an owned `T<'long>` to an owned `T<'short>` is
    // a covariant cast w.r.t. the lifetime. This cast is therefore sound.
    unsafe {
        transmute::<
            <T as WithLifetime<'long, 'lower, Upper>>::Is,
            <T as WithLifetime<'short, 'lower, Upper>>::Is,
        >(long)
    }
}

/// Lengthen a lifetime via a contravariant cast.
#[inline]
#[must_use]
pub fn lengthen<'short, 'long, 'lower, Upper, T>(
    short: Varying<'short, 'lower, Upper, T>,
) -> Varying<'long, 'lower, Upper, T>
where
    Upper: UpperBound + 'long,
    'long: 'short,
    'short: 'lower,
    T: ?Sized + ContravariantFamily<'lower, Upper, Is: Sized>,
{
    // SAFETY: As guaranteed by the unsafe `ContravariantFamily` trait, unsafely performing
    // contravariant casts on `T<'varying>` is sound. Lengthening an owned `T<'short>` to an owned
    // `T<'long>` is a contravariant cast w.r.t. the lifetime. This cast is therefore sound.
    unsafe {
        transmute::<
            <T as WithLifetime<'short, 'lower, Upper>>::Is,
            <T as WithLifetime<'long, 'lower, Upper>>::Is,
        >(short)
    }
}

/// Shorten the lifetime of a [`Lend`] via a covariant cast.
#[inline]
#[must_use]
pub fn shorten_lend<'short, 'long, Upper, T>(
    long: Lend<'long, Upper, T>,
) -> Lend<'short, Upper, T>
where
    Upper: UpperBound + 'long,
    'long: 'short,
    T: ?Sized + LendFamily<Upper>,
{
    let long = change_bounds_from::<'long, 'short, 'long, Upper, Upper, T>(long);
    // SAFETY: As guaranteed by the unsafe `CovariantFamily` trait, unsafely performing covariant
    // casts on `T<'varying>` is sound. Shortening an owned `T<'long>` to an owned `T<'short>` is
    // a covariant cast w.r.t. the lifetime. This cast is therefore sound.
    unsafe {
        transmute::<
            <T as WithLifetime<'long, 'short, Upper>>::Is,
            <T as WithLifetime<'short, 'short, Upper>>::Is,
        >(long)
    }
}

/// Abstract nonsense which allows the `'lower` and `Upper` bounds on a `'varying` lifetime to
/// be changed (noting that those bounds are prohibited from being included in the `T<'varying>`
/// type).
#[inline]
#[must_use]
pub fn change_bounds_from<'varying, 'lower, 'other_lower, Upper, OtherUpper, T>(
    other: Varying<'varying, 'other_lower, OtherUpper, T>,
) -> Varying<'varying, 'lower, Upper, T>
where
    Upper: UpperBound,
    OtherUpper: UpperBound,
    T: ?Sized
        + LifetimeFamily<'lower, Upper>
        + WithLifetime<'varying, 'other_lower, OtherUpper, Is: Sized>,
    Varying<'varying, 'lower, Upper, T>: Sized,
{
    let other = ManuallyDrop::new(other);
    // SAFETY: By the implementation safety conditions of `WithLifetime`, its `Is` associated
    // type does not use the upper or lower bound (even though it hypothetically *could* in an
    // unsound implementation, thus why the compiler does not recognize the types as the same).
    // The upper and lower bounds are the only differences between these types. Therefore, this is
    // a trivial transmute.
    // It does still copy out of `other` without moving out of `other`, so we need to avoid a
    // double-drop; we do so via `ManuallyDrop`.
    unsafe {
        transmute_copy::<
            <T as WithLifetime<'varying, 'other_lower, OtherUpper>>::Is,
            <T as WithLifetime<'varying, 'lower, Upper>>::Is,
        >(&other)
    }
}

/// Abstract nonsense which allows the `'lower` and `Upper` bounds on a `'varying` lifetime to
/// be changed (noting that those bounds are prohibited from being included in the `T<'varying>`
/// type).
#[inline]
#[must_use]
pub fn change_bounds_into<'varying, 'lower, 'other_lower, Upper, OtherUpper, T>(
    this: Varying<'varying, 'lower, Upper, T>,
) -> Varying<'varying, 'other_lower, OtherUpper, T>
where
    Upper: UpperBound,
    OtherUpper: UpperBound,
    T: ?Sized
        + LifetimeFamily<'lower, Upper>
        + WithLifetime<'varying, 'other_lower, OtherUpper, Is: Sized>,
    Varying<'varying, 'lower, Upper, T>: Sized,
{
    let this = ManuallyDrop::new(this);
    // SAFETY: By the implementation safety conditions of `WithLifetime`, its `Is` associated
    // type does not use the upper or lower bound (even though it hypothetically *could* in an
    // unsound implementation, thus why the compiler does not recognize the types as the same).
    // The upper and lower bounds are the only differences between these types. Therefore, this is
    // a trivial transmute.
    // It does still copy out of `other` without moving out of `other`, so we need to avoid a
    // double-drop; we do so via `ManuallyDrop`.
    unsafe {
        transmute_copy::<
            <T as WithLifetime<'varying, 'lower, Upper>>::Is,
            <T as WithLifetime<'varying, 'other_lower, OtherUpper>>::Is,
        >(&this)
    }
}
