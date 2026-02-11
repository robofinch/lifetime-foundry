use core::mem::transmute;

use crate::invariant_zst;
use crate::traits::{ContravariantFamily, CovariantFamily, UpperBound, Varying, WithLifetime};


// We *could* use the public macros to implement everything besides the function pointer cases,
// but I think it's valuable to show the full `unsafe` code for at least the most crucial cases.

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

impl<'a, 'varying, 'lower, Upper, T> WithLifetime<'varying, 'lower, Upper> for &'a T
where
    Upper: UpperBound,
    T: ?Sized + WithLifetime<'varying, 'lower, Upper>,
    T::Is: 'a,
{
    type Is = &'a T::Is;
}

// SAFETY:
// - If `Self::covariant_assertions()` does not panic,
//   then `Self<'varying>` is covariant over `'varying`.
//
//   The former implies that `T::covariant_assertions()` does not panic,
//   in which case `T<'varying>` is covariant over `'varying`,
//   implying that `&'a T<'varying>` is covariant over `'varying`.
//
// - No assertions are included other than those in `Self::covariant_assertions()`.
// - The implementation safety requirements of `shorten` and `shorten_ref` are met.
unsafe impl<'a, 'lower, Upper, T> CovariantFamily<'lower, Upper> for &'a T
where
    Upper: UpperBound,
    T: ?Sized + CovariantFamily<'lower, Upper>,
    for<'varying> <T as WithLifetime<'varying, 'lower, Upper>>::Is: 'a,
{
    #[inline]
    fn covariant_assertions() {
        T::covariant_assertions();
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
        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is just a covariant cast with, possibly, some assertions.
        // Any possible sources of panics in `T::shorten_ref` must be included in
        // `T::covariant_assertions`, which are included in `Self::covariant_assertions`.

        T::shorten_ref(long)
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
        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is a covariant cast with some assertions.
        // There are no possible sources of panics other than the `Self::covariant_assertions()`
        // call.

        Self::covariant_assertions();

        let src: &'r &'a Varying<'l, 'lower, Upper, T> = long;
        // SAFETY: we are shortening the `'l` lifetime of `T<'l>` to `'s`, which is
        // at least as long as `'lower`. We called `T::covariant_assertions()` (within
        // `Self::covariant_assertions()`), so covariantly casting `T<'varying>` is sound.
        let dst: &'r &'a Varying<'s, 'lower, Upper, T> = unsafe { transmute(src) };
        dst
    }
}

// SAFETY:
// - If `Self::contravariant_assertions()` does not panic,
//   then `Self<'varying>` is contravariant over `'varying`.
//
//   The former implies that `T::contravariant_assertions()` does not panic,
//   in which case `T<'varying>` is contravariant over `'varying`,
//   implying that `&'a T<'varying>` is contravariant over `'varying`.
//
// - No assertions are included other than those in `Self::contravariant_assertions()`.
// - The implementation safety requirements of `lengthen` and `lengthen_ref` are met.
unsafe impl<'a, 'lower, Upper, T> ContravariantFamily<'lower, Upper> for &'a T
where
    Upper: UpperBound,
    T: ?Sized + ContravariantFamily<'lower, Upper>,
    for<'varying> <T as WithLifetime<'varying, 'lower, Upper>>::Is: 'a,
{
    #[inline]
    fn contravariant_assertions() {
        T::contravariant_assertions();
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
        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is just a contravariant cast with, possibly, some assertions.
        // Any possible sources of panics in `T::lengthen_ref` must be included in
        // `T::contravariant_assertions`, which are included in `Self::contravariant_assertions`.

        T::lengthen_ref(short)
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
        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is a contravariant cast with some assertions.
        // There are no possible sources of panics other than the `Self::contravariant_assertions()`
        // call.

        Self::contravariant_assertions();

        let src: &'r &'a Varying<'s, 'lower, Upper, T> = short;
        // SAFETY: we are lengthening the `'s` lifetime of `T<'s>` to `'l`, which is
        // at most as long as any lifetime in `Upper`. We called `T::contravariant_assertions()`
        // (within `Self::contravariant_assertions()`), so contravariantly casting `T<'varying>`
        // is sound.
        let dst: &'r &'a Varying<'l, 'lower, Upper, T> = unsafe { transmute(src) };
        dst
    }
}


// ================================================================
//  &'varying T    (VaryingRef<T>)
// ================================================================

// Safety summary:
// - `&'varying T<'varying>` is covariant over `'varying` if `T<'varying>` is covariant over it.
// - `&'varying T<'varying>` is never contravariant over `'varying`.

invariant_zst!(
    /// The `&'varying T<'varying>` lifetime family.
    ///
    /// If `T<'varying>` is covariant over `'varying`, then `&'varying T<'varying>` is covariant
    /// over `'varying`.
    ///
    /// This lifetime family is never contravariant over `'varying`.
    ///
    /// Note that this type itself is just a marker ZST for the family.
    pub struct VaryingRef<T: ?Sized>;
);

impl<'varying, 'lower, Upper, T> WithLifetime<'varying, 'lower, Upper> for VaryingRef<T>
where
    Upper: UpperBound,
    T: ?Sized + WithLifetime<'varying, 'lower, Upper>,
    T::Is: 'varying,
{
    type Is = &'varying T::Is;
}

// SAFETY:
// - If `Self::covariant_assertions()` does not panic,
//   then `Self<'varying>` is covariant over `'varying`.
//
//   The former implies that `T::covariant_assertions()` does not panic,
//   in which case `T<'varying>` is covariant over `'varying`,
//   implying that `&'varying T<'varying>` is covariant over `'varying`.
//
// - No assertions are included other than those in `Self::covariant_assertions()`.
// - The implementation safety requirements of `shorten` and `shorten_ref` are met.
unsafe impl<'lower, Upper, T> CovariantFamily<'lower, Upper> for VaryingRef<T>
where
    Upper: UpperBound,
    T: ?Sized + CovariantFamily<'lower, Upper>,
    for<'varying> <T as WithLifetime<'varying, 'lower, Upper>>::Is: 'varying,
{
    #[inline]
    fn covariant_assertions() {
        T::covariant_assertions();
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
        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is just a covariant cast with, possibly, some assertions.
        // Any possible sources of panics in `T::shorten_ref` must be included in
        // `T::covariant_assertions`, which are included in `Self::covariant_assertions`.

        T::shorten_ref(long)
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
        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is a covariant cast with some assertions.
        // There are no possible sources of panics other than the `Self::covariant_assertions()`
        // call.

        Self::covariant_assertions();

        let src: &'r &'l Varying<'l, 'lower, Upper, T> = long;
        let src: &'r &'s Varying<'l, 'lower, Upper, T> = src;
        // SAFETY: we are shortening the `'l` lifetime of `T<'l>` to `'s`, which is
        // at least as long as `'lower`. We called `T::covariant_assertions()` (within
        // `Self::covariant_assertions()`), so covariantly casting `T<'varying>` is sound.
        let dst: &'r &'s Varying<'s, 'lower, Upper, T> = unsafe { transmute(src) };
        dst
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

impl<'varying, 'lower, Upper, T> WithLifetime<'varying, 'lower, Upper> for *const T
where
    Upper: UpperBound,
    T: ?Sized + WithLifetime<'varying, 'lower, Upper>,
{
    type Is = *const T::Is;
}

// SAFETY:
// - If `Self::covariant_assertions()` does not panic,
//   then `Self<'varying>` is covariant over `'varying`.
//
//   The former implies that `T::covariant_assertions()` does not panic,
//   in which case `T<'varying>` is covariant over `'varying`,
//   implying that `*const T<'varying>` is covariant over `'varying`.
//
// - No assertions are included other than those in `Self::covariant_assertions()`.
// - The implementation safety requirements of `shorten` and `shorten_ref` are met.
unsafe impl<'lower, Upper, T> CovariantFamily<'lower, Upper> for *const T
where
    Upper: UpperBound,
    T: ?Sized + CovariantFamily<'lower, Upper>,
{
    #[inline]
    fn covariant_assertions() {
        T::covariant_assertions();
    }

    /// Shorten the `'varying` lifetime of `*const T<'varying>`.
    ///
    /// If the given pointer points to a valid value of type `Varying<'l, 'lower, Upper, T>`
    /// (also referred to as `T<'l>`), the returned pointer (which is the given pointer with a
    /// casted type) points to a valid value of type `Varying<'s, 'lower, Upper, T>`
    /// (also referred to as `T<'s>`).
    ///
    /// As the returned pointer is not modified (other than to change its type), any other
    /// qualities relevant for reads or writes through the pointer (such as alignment or provenance)
    /// are unchanged.
    ///
    /// `unsafe` code can rely on this guarantee.
    fn shorten<'l, 's>(
        long: Varying<'l, 'lower, Upper, Self>,
    ) -> Varying<'s, 'lower, Upper, Self>
    where
        Upper: 'l,
        'l: 's,
        's: 'lower,
        for<'varying> Varying<'varying, 'lower, Upper, Self>: Sized,
    {
        #![expect(clippy::as_conversions, reason = "`.cast()` requires a `Sized` bound")]

        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is a covariant cast with some assertions.
        // There are no possible sources of panics other than the `Self::covariant_assertions()`
        // call.
        Self::covariant_assertions();

        let src: *const Varying<'l, 'lower, Upper, T> = long;
        // Correctness of guarantee: we are shortening the `'l` lifetime of `T<'l>` to `'s`, which
        // is at least as long as `'lower`. We called `T::covariant_assertions()` (within
        // `Self::covariant_assertions()`), so covariantly casting `T<'varying>` is sound.
        src as *const Varying<'s, 'lower, Upper, T>
    }

    /// Shorten the `'varying` lifetime of `&(*const T<'varying>)`.
    ///
    /// If the referenced pointer points to a valid value of type `Varying<'l, 'lower, Upper, T>`
    /// (also referred to as `T<'l>`), that pointer (whose reference is returned with a casted
    /// type) also points to a valid value of type `Varying<'s, 'lower, Upper, T>` (also referred
    /// to as `T<'s>`).
    ///
    /// As the returned reference to the pointer is not modified (other than to change the
    /// pointer's type), any other qualities relevant for reads or writes through the pointer
    /// (such as alignment or provenance) are unchanged.
    ///
    /// `unsafe` code can rely on this guarantee.
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
        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is a covariant cast with some assertions.
        // There are no possible sources of panics other than the `Self::covariant_assertions()`
        // call.

        Self::covariant_assertions();

        let src: &'r *const Varying<'l, 'lower, Upper, T> = long;
        // SAFETY: we are shortening the `'l` lifetime of `T<'l>` to `'s`, which is
        // at least as long as `'lower`. We called `T::covariant_assertions()` (within
        // `Self::covariant_assertions()`), so covariantly casting `T<'varying>` is sound.
        let dst: &'r *const Varying<'s, 'lower, Upper, T> = unsafe { transmute(src) };
        dst
    }
}

// SAFETY:
// - If `Self::contravariant_assertions()` does not panic,
//   then `Self<'varying>` is contravariant over `'varying`.
//
//   The former implies that `T::contravariant_assertions()` does not panic,
//   in which case `T<'varying>` is contravariant over `'varying`,
//   implying that `*const T<'varying>` is contravariant over `'varying`.
//
// - No assertions are included other than those in `Self::contravariant_assertions()`.
// - The implementation safety requirements of `lengthen` and `lengthen_ref` are met.
unsafe impl<'lower, Upper, T> ContravariantFamily<'lower, Upper> for *const T
where
    Upper: UpperBound,
    T: ?Sized + ContravariantFamily<'lower, Upper>,
{
    #[inline]
    fn contravariant_assertions() {
        T::contravariant_assertions();
    }

    /// Lengthen the `'varying` lifetime of `*const T<'varying>`.
    ///
    /// If the given pointer points to a valid value of type `Varying<'s, 'lower, Upper, T>`
    /// (also referred to as `T<'s>`), the returned pointer (which is the given pointer with a
    /// casted type) points to a valid value of type `Varying<'l, 'lower, Upper, T>`
    /// (also referred to as `T<'l>`).
    ///
    /// As the returned pointer is not modified (other than to change its type), any other
    /// qualities relevant for reads or writes through the pointer (such as alignment or provenance)
    /// are unchanged.
    ///
    /// `unsafe` code can rely on this guarantee.
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
        #![expect(clippy::as_conversions, reason = "`.cast()` requires a `Sized` bound")]

        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is a contravariant cast with some assertions.
        // There are no possible sources of panics other than the `Self::contravariant_assertions()`
        // call.
        Self::contravariant_assertions();

        let src: *const Varying<'s, 'lower, Upper, T> = short;
        // Correctness of guarantee: we are lengthening the `'s` lifetime of `T<'s>` to `'l`, which
        // is at most as long as any lifetime in `Upper`. We called `T::contravariant_assertions()`
        // (within `Self::contravariant_assertions()`), so contravariantly casting `T<'varying>`
        // is sound.
        src as *const Varying<'l, 'lower, Upper, T>
    }

    /// Lengthen the `'varying` lifetime of `&(*const T<'varying>)`.
    ///
    /// If the referenced pointer points to a valid value of type `Varying<'s, 'lower, Upper, T>`
    /// (also referred to as `T<'s>`), that pointer (whose reference is returned with a casted
    /// type) also points to a valid value of type `Varying<'l, 'lower, Upper, T>` (also referred
    /// to as `T<'l>`).
    ///
    /// As the returned reference to the pointer is not modified (other than to change the
    /// pointer's type), any other qualities relevant for reads or writes through the pointer
    /// (such as alignment or provenance) are unchanged.
    ///
    /// `unsafe` code can rely on this guarantee.
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
        #![expect(clippy::unnecessary_safety_comment, reason = "implementation safety of method")]
        // Implementation safety: this is a contravariant cast with some assertions.
        // There are no possible sources of panics other than the `Self::contravariant_assertions()`
        // call.

        Self::contravariant_assertions();

        let src: &'r *const Varying<'s, 'lower, Upper, T> = short;
        // SAFETY: we are lengthening the `'s` lifetime of `T<'s>` to `'l`, which is
        // at most as long as any lifetime in `Upper`. We called `T::contravariant_assertions()`
        // (within `Self::contravariant_assertions()`), so contravariantly casting `T<'varying>`
        // is sound.
        let dst: &'r *const Varying<'l, 'lower, Upper, T> = unsafe { transmute(src) };
        dst
    }
}
