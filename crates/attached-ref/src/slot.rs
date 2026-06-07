#![expect(unsafe_code, reason = "perform unsafe lifetime erasure and extension")]

use core::mem::{ManuallyDrop, MaybeUninit, transmute};

use variance_family::{Lend, LendFamily};


#[expect(missing_docs, reason = "TODO")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SelfRefCases<N, R, M> {
    NoRef(N),
    Ref(R),
    RefMut(M),
}

#[expect(missing_docs, reason = "TODO")]
pub type SelfRefSlot<'stable, 'upper, N, R, M> = SelfRefCases<
    N,
    Lend<'stable, &'upper (), R>,
    Lend<'stable, &'upper (), M>,
>;

/// A version of [`SelfRefSlot<'stable, 'upper, N, R, M>`] with its `'stable` and `'upper`
/// lifetimes unsafely erased to `'erased`.
///
/// This functionality doesn't *need* to exist as a type, but it hopefully provides a
/// *slightly* simpler `unsafe` facade to [`AttachableRefFull`] compared to direct lifetime
/// transmutes and handling of [`MaybeUninit`].
///
/// # Robust Guarantee
/// This type semantically allows both covariant and contravariant casts of its `'erased`
/// parameter. That is, in many covariant, contravariant, and even invariant positions, the
/// `'erased` lifetime can be changed to any other lifetime (such that the where-bounds of this
/// struct still hold).
///
/// More precisely, `ErasedSelfRefSlot<'e1, N, R, M>` can be soundly transmuted to
/// `ErasedSelfRefSlot<'e2, N, R, M>`.
///
/// Notably, it is *not* generally the case that `GenericType<ErasedSelfRefSlot<'e1, N, R, M>>`
/// can be soundly transmuted to `GenericType<ErasedSelfRefSlot<'e2, N, R, M>>`, since
/// `<ErasedSelfRefSlot<'e1, N, R, M> as Trait>::Assoc` cannot generally be soundly
/// transmuted to `<ErasedSelfRefSlot<'e2, N, R, M> as Trait>::Assoc`, and `GenericType` may
/// contain an associated type dependent on the exact `'erased` lifetime.
///
/// However -- perhaps barring questionable generativity-ish patterns reliant on references instead
/// of custom guard types -- references to this struct (such as `&`, `&mut`, `&&&&`, or
/// `&mut &&&mut` references) merely enable reading and writing values of this struct. Changing
/// the `'e1` lifetime of this struct under one or more nested references to `'e2` means that
/// reads and writes effectively perform transmutes between `ErasedSelfRefSlot<'e1, N, R, M>` and
/// `ErasedSelfRefSlot<'e2, N, R, M>`, which is sound.
///
/// [`AttachableRefFull`]: crate::attachable_ref_full::AttachableRefFull
pub(crate) struct ErasedSelfRefSlot<'erased, N, R, M>
where
    R: LendFamily<&'erased ()>,
    M: LendFamily<&'erased ()>,
{
    /// # Proof of Robust Guarantee about `'erased`
    /// The first `'erased` lifetime is fake and should not matter, even though it *does* tie this
    /// struct to a particular `'erased` lifetime, which (in general) can provide a
    /// genuinely-invariant-not-bivariant lifetime parameter to an associated type of a trait.
    ///
    /// The second `'erased` lifetime is fed to `Lend<'_, &'erased (), _>`; in other words, this
    /// struct directly uses an associated type of [`variance_family::WithLifetime`] where
    /// `'erased` is used in the `Upper` type parameter of that trait, but this struct does not
    /// itself use `Upper`. By the safety invariant of `WithLifetime`, `'erased` should not actually
    /// appear in the expanded associated type, which *at least* makes direct covariant and
    /// contravariant casts of this struct sound (since `unsafe` code in this crate does not attach
    /// any significance to the `'erased` lifetime of this type).
    ///
    /// # Safety Invariant
    /// For some (possibly-not-actually-nameable) `'s` lifetime which lasts at least until
    /// this type's destructor is running or when it is forgotten, the value in `erased` must be
    /// initialized to a valid value of type `SelfRefSlot<'s, 'erased, N, S, E>`.
    ///
    /// The exact value might not actually be left valid for all of `'s`; it can be switched out,
    /// and old value taken out of `self.erased` might then be invalidated. Additionally,
    /// `self.erased` could actually be taken out of *first* and then replaced, so long as the
    /// safety invariant is maintained whenever `self` is exposed to Arbitrary User Code (TM).
    /// (Beware of unwinds.)
    ///
    /// The `erased` field is dropped in the `Drop::drop` implementation of this type,
    /// after which point the field should be considered uninitialized.
    ///
    /// Note that the methods of this type are careful not to expose the dangling
    /// `'s = 'erased` lifetime outside `MaybeUninit`.
    ///
    /// `MaybeUninit` is used here solely to remove `dereferenceable` (and possibly `noalias`)
    /// requirements from the `SelfRefSlot` value. This can be relevant in destructors of
    /// self-referential structs, where the incorrect `'s = 'erased` lifetime could otherwise
    /// cause unsoundness.
    erased: MaybeUninit<SelfRefSlot<'erased, 'erased, N, R, M>>,
}

#[expect(unreachable_pub, reason = "control visibility at type definition")]
impl<'erased, N, R, M> ErasedSelfRefSlot<'erased, N, R, M>
where
    R: LendFamily<&'erased ()>,
    M: LendFamily<&'erased ()>,
{
    /// # Safety
    /// The `'stable` data in `unerased` must not be invalidated until the returned `Self` value
    /// is either consumed via [`Self::into_unerased`], dropped, or forgotten.
    ///
    /// Note that covariance over `'stable` (and the ability to arbitrarily shorten `'stable`)
    /// is enforced by the type system, making this requirement sensible.
    ///
    /// # Robust Guarantee
    ///
    /// This function does not unwind.
    #[inline]
    #[must_use]
    pub const unsafe fn erase<'stable>(unerased: SelfRefSlot<'stable, 'erased, N, R, M>) -> Self {
        let unerased = MaybeUninit::new(unerased);

        // NOTE: the meat of this type's methods are never the lifetime transmutes, but instead
        // are the `assume_init*` parts (or safety invariants) afterwards.
        // This boilerplate is just included here for completeness.

        // SAFETY: This is just a lifetime transmute. Lifetimes do not affect codegen and therefore
        // do not affect layout, so both types have the same size and align. Moreover, *any* byte
        // pattern (of the correct size) is valid for the source and destination `MaybeUninit`
        // types, satisfying the safety docs of `transmute`.
        let erased = unsafe {
            transmute::<
                MaybeUninit<SelfRefSlot<'stable, 'erased, N, R, M>>,
                MaybeUninit<SelfRefSlot<'erased, 'erased, N, R, M>>,
            >(unerased)
        };

        // SAFETY INVARIANT: The `'stable` data is not invalidated until the returned `Self` value
        // is destructed or forgotten (including inside `Self::into_unerased`). Since the slot
        // is semantically covariant over `'stable`, this implies that `self.erased` is initialized
        // to a valid value of type `SelfRefSlot<'s, 'erased, N, S, E>` for the
        // possibly-unnameably `'s` that lasts until the returned `Self` is destructed or forgotten.
        // Note that the above moves -- including the bitwise moves of `MaybeUninit` values --
        // preserve the validity of the given value.
        Self { erased }
    }

    /// # Safety
    /// The `'stable` data erased into `self` must not have been invalidated from when it was
    /// written to `self` up until now. Additionally, although `'stable` could be set to an
    /// overly-long lifetime like `'static`, the return value must only be used while the
    /// `'stable` data has not yet been invalidated.
    ///
    /// Note that covariance over `'stable` (and the ability to arbitrarily shorten `'stable`)
    /// is enforced by the type system, making this requirement sensible.
    #[inline]
    #[must_use]
    pub unsafe fn into_unerased<'stable>(self) -> SelfRefSlot<'stable, 'erased, N, R, M> {
        let this = ManuallyDrop::new(self);
        let this_ref: &Self = &this;

        let erased = &raw const this_ref.erased;

        // SAFETY INVARIANT: This is a standard approach for moving out of a field of a type that
        // implements `Drop`, after disarming the destructor. Since `this.erased` is not
        // necessarily `Copy`, we can consider `this.erased` to be left semantically uninitialized,
        // and only use this `erased` value from then on. Since `this` is forgotten thanks to
        // `ManuallyDrop`, it doesn't matter that `this.erased` is no longer initialized; the safety
        // invariant no longer enforces anything.
        //
        // SAFETY: As mentioned directly above, this is standard. Formally:
        // - `erased` is valid for reads, since:
        //   - `erased` is non-null, since it points to an in-bounds part of a Rust allocation.
        //   - `erased` is dereferenceable, since it points to a single allocation which it has
        //     provenance over, since (as per the documentation of `addr_of!`) it inherited the
        //     provenance and permissions of `this_ref` over the fields of `this`, and we have not
        //     yet touched `this` again (which could end the lifetime of `this_ref`).
        //   - The read does not race with any other accesses to `erased`'s pointee, since this
        //     function has exclusive ownership over `this`.
        //   - We are not interleaving accesses to `this.erased` between references and pointers;
        //     access is nested (`&this -> this_ref -> erased`).
        // - `erased` is properly aligned, since `Self` is not `repr(packed)`, implying that
        //   the `this.erased` field is properly aligned for the type of `this.erased`.
        // - The pointee is, trivially, a valid value of type `MaybeUninit<..>`.
        // Additionally, we do not trigger a double-drop.
        let erased = unsafe { erased.read() };

        // SAFETY: Same as the `transmute` in `erase`. The meat of this method is `assume_init`.
        let unerased = unsafe {
            transmute::<
                MaybeUninit<SelfRefSlot<'erased, 'erased, N, R, M>>,
                MaybeUninit<SelfRefSlot<'stable, 'erased, N, R, M>>,
            >(erased)
        };

        // SAFETY: By the safety invariant, and by the caller's assertions, this is effectively a
        // lifetime transmute from `SelfRefSlot<'s, 'erased, N, R, M>` to
        // `SelfRefSlot<'stable, 'erased, N, R, M>`, split across two places and time, where `'s`
        // is a lifetime that lasts until `self` is destructed or forgotten.
        // The caller unsafely asserts that the `'stable`/`'s` data has not and will not be
        // invalidated while the returned value is used; therefore, `unerased` is properly
        // initialized (probably via a covariant shortening from `'s` to `'stable`, but perhaps with
        // an overly-long `'stable` lifetime which the caller is responsible for dealing with).
        unsafe { unerased.assume_init() }
    }

    /// # Safety
    /// The `'stable` data erased into `self` must not have been invalidated from when it was
    /// written to `self` up until now. Additionally, although `'stable` could be set to an
    /// overly-long lifetime like `'static`, the return value must only be used while the
    /// `'stable` data has not yet been invalidated.
    ///
    /// Note that covariance over `'stable` (and the ability to arbitrarily shorten `'stable`)
    /// is enforced by the type system, making this requirement sensible.
    #[inline]
    #[must_use]
    pub const unsafe fn unerase_ref<'a, 'stable>(
        &'a self,
    ) -> &'a SelfRefSlot<'stable, 'erased, N, R, M> {
        let erased = &self.erased;

        // NOTE: the meat of this type's methods are never the lifetime transmutes, but instead
        // are the `assume_init*` parts (or safety invariants) afterwards.
        // This boilerplate is just included here for completeness.

        // SAFETY: This is just a lifetime transmute. Lifetimes do not affect codegen and therefore
        // do not affect layout, so the pointee types of the two reference types have the same
        // size and align. Moreover, both pointees are `MaybeUninit`, so *any* byte pattern is
        // valid for the destination pointee.
        // Formally:
        // - The value is properly aligned.
        // - The value is non-null.
        // - The value is dereferenceable (points to a single allocation which it has provenance
        //   over),
        // - The value is a valid value of type `&MaybeUninit<..>`, as discussed above.
        // - The aliasing rules are enforced, since the outermost lifetime (`'a`) is unchanged
        //   and the type of pointer/reference is not changed.
        let unerased = unsafe {
            transmute::<
                &'a MaybeUninit<SelfRefSlot<'erased, 'erased, N, R, M>>,
                &'a MaybeUninit<SelfRefSlot<'stable, 'erased, N, R, M>>,
            >(erased)
        };

        // SAFETY INVARIANT: We don't write `'stable` data to `self.erased`. (Note that covariance
        // over `'stable` means that not even interior mutability could soundly do that.)
        //
        // SAFETY: We are accessing a `SelfRefSlot<'s, 'erased, N, R, M>`
        // as `&'a SelfRefSlot<'stable, 'erased, N, R, M>` for some lifetime `'s`. As in
        // `Self::into_unerased`, `'s` is the lifetime discussed in the safety invariant of
        // `self.erased`, and the caller asserts that it has not yet expired. Reads via the returned
        // reference effectively perform a lifetime transmute from `'s` to `'stable`, split across
        // two places and times. The caller unsafely asserts that the `'stable` data has not and
        // will not be invalidated while the returned value is used; therefore, the pointee of
        // `unerased` is properly initialized (probably via a covariant shortening from `'s` to
        // `'stable`, but perhaps with an overly-long `'stable` lifetime which the caller is
        // responsible for dealing with).
        unsafe { unerased.assume_init_ref() }
    }

    /// # Safety
    /// The `'stable` data erased into `self` must not have been invalidated from when it was
    /// written to `self` up until now. Additionally, although `'stable` could be set to an
    /// overly-long lifetime like `'static`, the return value must only be used while the
    /// `'stable` data has not yet been invalidated.
    ///
    /// Lastly, and most importantly, any `'stable` data written to `self` via the returned
    /// mutable reference must not be invalidated until the returned `Self` value is either
    /// consumed via [`Self::into_unerased`], dropped, or forgotten.
    ///
    /// Note that covariance over `'stable` (and the ability to arbitrarily shorten `'stable`)
    /// is enforced by the type system, making this requirement sensible.
    #[inline]
    #[must_use]
    pub const unsafe fn unerase_mut<'a, 'stable>(
        &'a mut self,
    ) -> &'a mut SelfRefSlot<'stable, 'erased, N, R, M> {
        let erased = &mut self.erased;

        // SAFETY: Same as the `transmute` in `unerase_ref`.
        // `assume_init_mut` is the meat of this method.
        let unerased = unsafe {
            transmute::<
                &'a mut MaybeUninit<SelfRefSlot<'erased, 'erased, N, R, M>>,
                &'a mut MaybeUninit<SelfRefSlot<'stable, 'erased, N, R, M>>,
            >(erased)
        };

        // SAFETY INVARIANT: The caller could write `'stable` data to `self.erased`, but they
        // make the same guarantee required by `Self::erase`, which implies that the safety
        // invariant of `self.erased` is satisfied. (See the comment in `Self::erased` for more.)
        //
        // SAFETY: We are mutably accessing a `SelfRefSlot<'s, 'erased, N, R, M>`
        // as `&'a mut SelfRefSlot<'stable, 'erased, N, R, M>`, where the lifetime `'s` is the
        // one discussed in the safety invariant of `self.erased`. Reads via the returned reference
        // effectively perform a lifetime transmute from `'s` to `'stable`, split across two places
        // and times, and writes effectively perform a transmute from `'stable` to `'s`.
        // The caller unsafely asserts that the `'stable` data has not and will not be invalidated
        // while the returned value is used; therefore, the pointee of `unerased` is properly
        // initialized, and can be soundly used for reads or writes (though perhaps with an
        // overly-long lifetime which the caller is responsible for dealing with, and the soundness
        // of writes also depends on the above safety invariant being upheld).
        unsafe { unerased.assume_init_mut() }
    }

    /// # Safety
    /// For now, this may only be called in the destructor.
    unsafe fn unerase_drop<'a>(&'a mut self) {
        let erased = &mut self.erased;

        // SAFETY: Same as the `transmute` in `unerase_ref`.
        // `assume_init_drop` is the meat of this method.
        let unerased = unsafe {
            transmute::<
                &'a mut MaybeUninit<SelfRefSlot<'erased, 'erased, N, R, M>>,
                &'a mut MaybeUninit<SelfRefSlot<'_, 'erased, N, R, M>>,
            >(erased)
        };

        // SAFETY INVARIANT: We're in the destructor, so the safety invariant no longer enforces
        // anything.
        //
        // SAFETY: We call this method only once in the destructor. As per the safety invariant of
        // `self.erased`, we know that there is *some* `'s` lifetime live until at least the
        // destructor such that `unerased`'s pointee is valid and initialized as a value of type
        // `SelfRefSlot<'s, 'erased, N, R, M>`. We have not yet invalidated `self.erased` in
        // the destructor up to now, so it is still valid and initialized here.
        // Note that the `'_` lifetime in the above transmute can be inferred to last only within
        // this destructor. The borrow checker can't "pathologically *choose*" a lifetime or
        // something, all that matters is that *some* lifetime works; therefore, this call is sound.
        unsafe {
            unerased.assume_init_drop();
        }
    }

    // TODO (as needed): `unerase_take`, `replace`, `erase_write`.
}

impl<'erased, N, R, M> Drop for ErasedSelfRefSlot<'erased, N, R, M>
where
    R: LendFamily<&'erased ()>,
    M: LendFamily<&'erased ()>,
{
    fn drop(&mut self) {
        // SAFETY: We are calling this in the destructor.
        unsafe {
            self.unerase_drop();
        }
    }
}
