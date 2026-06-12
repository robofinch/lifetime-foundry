//! Provide two `StableViewer(Mut)<'a, 'varying, 'data, Data>` lifetime families.

#![expect(unsafe_code, reason = "assert the variance of `StableViewer` and `StableViewerMut")]

use core::marker::PhantomData;

use variance_family::phantom_zst_methods;
use variance_family::{
    ChangeBounds, CovariantFamily, RawMutVarying, RawVarying, UpperBound, WithLifetime,
};

use crate::viewer::{StableViewer, StableViewerMut};

/// A phantom marker representing the `StableViewer<'a, 'varying, 'data, Data>` lifetime family.
///
/// See [`variance_family::LifetimeFamily`] for more.
pub struct VaryingStableViewer<'a, 'data, Data: ?Sized>(PhantomData<(&'a &'data (), &'a Data)>);

phantom_zst_methods!(impl<{Data: ?Sized}> _ for VaryingStableViewer<{'_, '_, Data}>);

// SAFETY:
// - We use separate `'lower` and `'upper` parameters, not just `'a` and `'data`, so the
//   `'varying`-parameterized type does not depend on `'lower` and `'upper`; it only depends
//   on `'varying` and `Self`.
// - `VaryingStableViewer` is defined in this crate.
unsafe impl<'varying, 'lower, 'upper, 'a, 'data, Data> WithLifetime<'varying, 'lower, &'upper ()>
for VaryingStableViewer<'a, 'data, Data>
where
    'lower: 'a,
    'data:  'upper,
    Data:   ?Sized,
{
    type Is = StableViewer<'a, 'varying, 'data, Data>;
}

// SAFETY:
// - We use separate `'lower` and `'upper` parameters, not just `'a` and `'data`, so the
//   `'varying`-parameterized type does not depend on `'lower` and `'upper`; it only depends
//   on `'varying` and `Self`. Apparently, though, this code is too generic for the compiler to
//   realize that fact. I suppose it thinks there could be a second impl of `WithLifetime`
//   for this type. However, that's impossible.
unsafe impl<'varying, 'lower, 'upper, 'a, 'data, Data>
    ChangeBounds<'varying, 'lower, &'upper (), StableViewer<'a, 'varying, 'data, Data>>
for VaryingStableViewer<'a, 'data, Data>
where
    'lower: 'a,
    'data:  'upper,
    Data:   ?Sized,
{
    fn prove_equal<'other_lower, OtherUpper>(
        varying: RawMutVarying<'varying, 'other_lower, OtherUpper, Self>,
    ) -> *mut *mut StableViewer<'a, 'varying, 'data, Data>
    where
        Self:       WithLifetime<'varying, 'other_lower, OtherUpper>,
        OtherUpper: UpperBound,
    {
        varying.cast()
    }
}

// SAFETY: Since `CovariantFamily::prove_covariance` is implemented with the function body
// `{ long }`, this implementation is certainly sound.
unsafe impl<'lower, 'upper, 'a, 'data, Data>
    CovariantFamily<'lower, &'upper ()> for VaryingStableViewer<'a, 'data, Data>
where
    'lower: 'a,
    'data:  'upper,
    Data:   ?Sized,
{
    fn prove_covariance<'long, 'short>(
        long: RawVarying<'long, 'a, &'data (), Self>,
    ) -> RawVarying<'short, 'a, &'data (), Self>
    where
        &'data (): 'long,
        'long: 'short,
        'short: 'a
    {
        long
    }
}

/// A phantom marker representing the `StableViewerMut<'a, 'varying, 'data, Data>` lifetime family.
///
/// See [`variance_family::LifetimeFamily`] for more.
pub struct VaryingStableViewerMut<'a, 'data, Data: ?Sized>(
    PhantomData<(&'a &'data (), &'a mut Data)>,
);

phantom_zst_methods!(impl<{Data: ?Sized}> _ for VaryingStableViewerMut<{'_, '_, Data}>);

// SAFETY:
// - We use separate `'lower` and `'upper` parameters, not just `'a` and `'data`, so the
//   `'varying`-parameterized type does not depend on `'lower` and `'upper`; it only depends
//   on `'varying` and `Self`.
// - `VaryingStableViewer` is defined in this crate.
unsafe impl<'varying, 'lower, 'upper, 'a, 'data, Data> WithLifetime<'varying, 'lower, &'upper ()>
for VaryingStableViewerMut<'a, 'data, Data>
where
    'lower: 'a,
    'data:  'upper,
    Data:   ?Sized,
{
    type Is = StableViewerMut<'a, 'varying, 'data, Data>;
}

// SAFETY:
// - We use separate `'lower` and `'upper` parameters, not just `'a` and `'data`, so the
//   `'varying`-parameterized type does not depend on `'lower` and `'upper`; it only depends
//   on `'varying` and `Self`. Apparently, though, this code is too generic for the compiler to
//   realize that fact. I suppose it thinks there could be a second impl of `WithLifetime`
//   for this type. However, that's impossible.
unsafe impl<'varying, 'lower, 'upper, 'a, 'data, Data>
    ChangeBounds<'varying, 'lower, &'upper (), StableViewerMut<'a, 'varying, 'data, Data>>
for VaryingStableViewerMut<'a, 'data, Data>
where
    'lower: 'a,
    'data:  'upper,
    Data:   ?Sized,
{
    fn prove_equal<'other_lower, OtherUpper>(
        varying: RawMutVarying<'varying, 'other_lower, OtherUpper, Self>,
    ) -> *mut *mut StableViewerMut<'a, 'varying, 'data, Data>
    where
        Self:       WithLifetime<'varying, 'other_lower, OtherUpper>,
        OtherUpper: UpperBound,
    {
        varying.cast()
    }
}

// SAFETY: Since `CovariantFamily::prove_covariance` is implemented with the function body
// `{ long }`, this implementation is certainly sound.
unsafe impl<'lower, 'upper, 'a, 'data, Data>
    CovariantFamily<'lower, &'upper ()> for VaryingStableViewerMut<'a, 'data, Data>
where
    'lower: 'a,
    'data:  'upper,
    Data:   ?Sized,
{
    fn prove_covariance<'long, 'short>(
        long: RawVarying<'long, 'a, &'data (), Self>,
    ) -> RawVarying<'short, 'a, &'data (), Self>
    where
        &'data (): 'long,
        'long: 'short,
        'short: 'a
    {
        long
    }
}
