#![expect(unsafe_code, reason = "perform unsafe lifetime erasure and extension of self-refs")]

use core::marker::PhantomData;

use variance_family::LendFamily;

use crate::slot::{ErasedSelfRefSlot, SelfRefSlot};


/// Out of *extra* paranoia, disable any accidental `Debug`ing, `Clone`ing, or other immutable
/// access to `Data` (which would invalidate mutable self-references).
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "For now, `AttachableRefFull` is implemented across the `super` module",
)]
#[repr(transparent)]
pub(super) struct SpeedBump<Data: ?Sized> {
    pub(super) speed_bump_inner: Data,
}

/// # Robust Guarantee
/// This type semantically allows both covariant and contravariant casts of its `'upper`
/// parameter. That is, in many covariant, contravariant, and even invariant positions, the
/// `'upper` lifetime can be changed to any other lifetime (such that the where-bounds of this
/// struct still hold).
///
/// More precisely, `AttachableRefFull<'data, 'u1, N, R, M, Data>` can be soundly transmuted to
/// `AttachableRefFull<'data, 'u2, N, R, M, Data>`.
///
/// Notably, it is *not* generally the case that
/// `GenericType<AttachableRefFull<'data, 'u1, N, R, M, Data>>` can be soundly transmuted to
/// `GenericType<AttachableRefFull<'data, 'u2, N, R, M, Data>>`, since
/// `<AttachableRefFull<'data, 'u1, N, R, M, Data> as Trait>::Assoc` cannot generally be soundly
/// transmuted to `<AttachableRefFull<'data, 'u2, N, R, M, Data> as Trait>::Assoc`, and
/// `GenericType` may contain an associated type dependent on the exact `'erased` lifetime.
///
/// However -- perhaps barring questionable generativity-ish patterns reliant on references instead
/// of custom guard types -- references to this struct (such as `&`, `&mut`, `&&&&`, or
/// `&mut &&&mut` references) merely enable reading and writing values of this struct. Changing
/// the `'u1` lifetime of this struct under one or more nested references to `'u2` means that
/// reads and writes effectively perform transmutes between
/// `AttachableRefFull<'data, 'u1, N, R, M, Data>` and
/// `AttachableRefFull<'data, 'u2, N, R, M, Data>`, which is sound.
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "For now, `AttachableRefFull` is implemented across the `super` module",
)]
pub struct AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    Data:   ?Sized,
    // Oh! I can change `upper` with a macro!
    // And remove a bunch of `PhantomData` stuff to make the compiler happier! ....except in
    // generic code. So maybe `PhantomData` has to stay????
{
    /// # Safety of Use
    /// This is a lifetime-erased `SelfRefSlot<'stable, 'erased, N, R, M>`.
    ///
    /// ## Dropping
    ///
    /// Until destruction, it must be initialized for some `'stable`, though within the drop
    /// glue of this type, it is dropped and therefore briefly uninitialized, I suppose.
    /// Additionally, since `self.data` is dropped later in the drop glue (which can leave some
    /// parts of `self.slot` not `dereferenceable`), this field currently needs to be wrapped in
    /// `MaybeUninit` to avoid violating the protectors of references passed as arguments to the
    /// drop glue function. (At some point, `MaybeDangling` would be nice.)
    ///
    /// Since Rust is specified to drop a struct's field in order from first to last, it is critical
    /// that `self.slot` appear before `self.data`, so that any references in `self.slot` do not
    /// dangle *while* it's being dropped. (They may only dangle in the brief window where
    /// `self.slot` is dropped and `self` is still being destructed.)
    ///
    /// ## Contained References
    ///
    /// For writing `self.slot` -- noting that exposing a `&mut` reference to the self-ref slot
    /// generally allows both reads and writes -- `'stable` data written to `self.slot` must
    /// either be self-references to `self.data` *or* be valid for at least `'data`.
    ///
    /// Since `self.slot`'s data is (semantically) covariant over the `'stable` lifetime parameter,
    /// this implies the following robust guarantee.
    ///
    /// # Robust Guarantees
    ///
    /// ## Unerasure
    /// When reading `self.slot`, the erased lifetime can be unerased to any `'stable` lifetime
    /// such that `'data: 'stable` and, at least until `'stable` ends, `self.data` is not
    /// manipulated in a way that invalidates `self.slot`.
    ///
    /// ## Changing `'upper`
    /// See the robust guarantee of [`ErasedSelfRefSlot`] about `'erased`.
    pub(super) slot:     ErasedSelfRefSlot<'upper, N, R, M>,
    /// Make this struct covariant over `'data`, and ensure invariance over `R` and `M`.
    ///
    /// (The latter should already be guaranteed anyway, but it can't hurt to be doubly-sure.)
    ///
    /// Note that this struct is also covariant over `Data`.
    pub(super) variance: PhantomData<fn(*mut R, *mut M) -> &'data ()>,
    /// # Safety Invariant
    /// TODO: revamping semantics of `stable-view` rn.
    pub(super) data:     SpeedBump<Data>,
}

impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    /// # Safety
    /// Any `'stable` data in `slot` must consist only of:
    /// - references to the given `data: Data` value, obtained via [`StableView::view`], and/or
    /// - data that lives for at least `'data`.
    ///
    /// # Robust Guarantee
    /// This function only moves `data`, and does not unwind (which could cause `data` to be
    /// unexpectedly dropped). Therefore, it does not invalidate any `'stable` data in `slot`.
    #[inline]
    #[must_use]
    pub(super) const unsafe fn from_slot<'stable: 'stable>(
        data: SpeedBump<Data>,
        slot: SelfRefSlot<'stable, 'upper, N, R, M>,
    ) -> Self {
        let erased = unsafe { ErasedSelfRefSlot::erase(slot) };

        Self {
            slot:     erased,
            variance: PhantomData,
            data,
        }
    }
}
