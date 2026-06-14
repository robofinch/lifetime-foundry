//! Mitigation for limitations in the pre-1.94.0 trait solver affecting closures.
#![expect(unsafe_code, reason = "perform unsafe lifetime erasure")]

use core::marker::PhantomData;
use core::mem::{ManuallyDrop, transmute};

use variance_family::{Lend, LendFamily};


/// A `Lend<'stable, &'upper (), T>` without any implied bounds between `'stable` and `'upper`.
///
/// (For example, `&'stable &'upper ()` has an implied `'upper: 'stable` bound.)
///
/// Values of this type are intended to be extremely short-lived; it's intended to be created
/// for the output of a closure and then immediately unwrapped by the caller, or vice-versa.
///
/// # Worked-around Problem
///
/// This type is a workaround for limitations in the pre-1.94.0 trait solver, regarding
/// higher-order closures. A higher-ranked `for<..> Fn*` trait bound whose return type's lifetime
/// varies with its input might look great on paper, but implied bounds could confuse the trait
/// solver.
///
/// In the case of this crate, some attempts to create a closure satisfying a bound like
/// ```rust
/// for<'a, 'stable> FnOnce(
///     CustomView<'a, 'stable, 'data, Data, View>,
///     PhantomData<&'a &'stable &'data ()>,
/// ) -> Lend<'stable, &'upper (), R>,
/// ```
/// resulted in the trait solver thinking that the closure was returning a value that lives
/// for `'data`, not `'upper`. The problem was solved in Rust 1.94.0, but the MSRV of this crate
/// is currently 1.85.
///
/// # Other Solutions
/// Instead of using closures and a higher-ranked `for<..> Fn*` bound, it is possible to use
/// a custom trait. However, stuffing captures into a struct and implementing the custom trait for
/// the struct is *far* too much boilerplate compared to using actual closures.
///
/// # Transparent
/// This type is guaranteed to be `repr(transparent)`. However, do not transmute this type
/// unless you **absolutely** know what you are doing.
///
/// # Advance Notice of Removal
/// Once this crate makes a breaking change that bumps the MSRV to 1.94 or higher, this type will
/// be removed.
#[expect(missing_debug_implementations, reason = "intended to be a *very* short-lived type")]
#[repr(transparent)]
pub struct LendWrapper<'stable, 'upper, T: LendFamily<&'upper ()>> {
    /// Maintains invariance over all three parameters.
    #[expect(clippy::type_complexity, reason = "the type is only mentioned here, so it's fine")]
    brand:  PhantomData<fn(*mut (&'stable (), &'upper (), T))>,
    /// # Safety Invariant
    /// Actually a `Lend<'stable, &'upper (), T>`.
    ///
    /// # Drop Check
    /// Note that we `impl Drop`, so dropck will assume that we could drop a
    /// `Lend<'stable, &'upper (), T>` in our `fn drop` implementation. (It doesn't actually look
    /// into the body of that function to determine that we don't, so we don't need to wrap
    /// this field in `ManuallyDrop` and drop it.)
    erased: Lend<'upper, &'upper (), T>,
}

impl<'stable, 'upper, T: LendFamily<&'upper ()>> LendWrapper<'stable, 'upper, T>
where
    'upper: 'stable,
{
    /// Temporarily remove the possible `'upper: 'stable` implied bound in
    /// `Lend<'stable, &'upper (), T>`, in order to return it from a higher-ranked closure.
    ///
    /// # Robust Guarantee
    /// This function does not unwind (unless UB has already occurred).
    #[inline]
    #[must_use]
    pub fn new(lend: Lend<'stable, &'upper (), T>) -> Self {
        // Note that calling `transmute` can only unwind if calling it would be UB.
        // Other than that, nothing we do here can unwind.

        // SAFETY: Since the two types differ only by a lifetime, the value is *bitwise* valid
        // at both the input and output type. The only concern is exposing this value with the
        // wrong lifetime to untrusted code.
        // The inner value can only be accessed in three ways:
        // - by dropping it,
        // - by calling `Self::into_lend`, or
        // - by other `unsafe` code performing transmutes (using the `repr(transparent)` guarantee).
        //
        // This is fine because:
        // - Specializing based on lifetimes is not permitted, so dropping
        //   `Lend<'upper, &'upper (), T>` runs the same code as dropping
        //   `Lend<'stable, &'upper (), T>`; therefore, we do not need to transmute
        //   `Lend<'upper, &'upper (), T>` to `Lend<'stable, &'upper (), T>` just to drop it.
        //   **However**, we need to worry about dropck; this `lend` might access data that lives
        //   for `'stable` when it's dropped. *At least* when `Lend<'stable, &'upper (), T>` could
        //   access data that lives for `'stable` when dropped, we need dropck to think that this
        //   type could do the same. It suffices to make dropck think that we could *always*
        //   (regardless of the `T` parameter) access data that lives for `'stable` when dropped;
        //   this is accomplished by implementing `Drop` for this type
        //   (and not using `#[may_dangle]`).
        // - `Self::into_lend` transmutes this value back to its proper lifetime (noting that
        //   this type is invariant over all its parameters, so variance doesn't break that fact).
        // - Other `unsafe` code is responsible for itself.
        let erased = unsafe {
            transmute::<
                Lend<'stable, &'upper (), T>,
                Lend<'upper, &'upper (), T>,
            >(lend)
        };

        // SAFETY INVARIANT: `erased` is actually a `Lend<'stable, &'upper (), T>`.
        Self {
            brand: PhantomData,
            erased,
        }
    }

    /// Unwrap this type into the source `Lend<'stable, &'upper (), T>`.
    ///
    /// # Robust Guarantee
    /// This function does not unwind (unless UB has already occurred).
    #[inline]
    #[must_use]
    pub fn into_lend(self) -> Lend<'stable, &'upper (), T> {
        // Note that calling `read` or `transmute` can only unwind if calling them would be UB.
        // Other than that, nothing we do here can unwind.

        let this = ManuallyDrop::new(self);
        let this_ref: &Self = &this;
        let erased: *const Lend<'upper, &'upper (), T> = &raw const this_ref.erased;

        // SAFETY: This is a common pattern for moving a non-`Copy` field out of a type that
        // implements `Drop`. We disabled the destructor of `self` by wrapping it in `ManuallyDrop`,
        // so this read cannot lead to a double drop.
        // More formally:
        // - `erased` is valid for reads, because:
        //   - It's not a null pointer (since it's inbounds of a Rust allocation)
        //   - It's dereferenceable for the type of `this.erased`, since it points to a Rust
        //     allocation large enough to store the `this.erased` value.
        //   - This does not race with any write, since we have exclusive ownership over `this`
        //   - We do not interleave accesses with pointers and references.
        // - It's properly aligned for the type of `this.erased`, since `Self` isn't `repr(packed)`.
        // - It trivially points to a valid value of the type of `this.erased`.
        let erased: Lend<'upper, &'upper (), T> = unsafe { erased.read() };

        // SAFETY: By the safety invariant of `self.erased`, `erased` is actually
        // a `Lend<'stable, &'upper (), T>`. Therefore, this value is bitwise valid as both the
        // input and output type, *and* exposing the output `Lend<'stable, &'upper (), T>`
        // to arbitrary code is sound.
        unsafe {
            transmute::<
                Lend<'upper, &'upper (), T>,
                Lend<'stable, &'upper (), T>,
            >(erased)
        }
    }
}

#[expect(clippy::empty_drop, reason = "only used to influence dropck")]
impl<'upper, T: LendFamily<&'upper ()>> Drop for LendWrapper<'_, 'upper, T> {
    #[inline]
    fn drop(&mut self) {}
}
