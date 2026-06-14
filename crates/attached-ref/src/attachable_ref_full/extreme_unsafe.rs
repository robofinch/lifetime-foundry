//! Temporary organization module.

use variance_family::LendFamily;

use crate::slot::SelfRefSlot;
use super::full_struct::{AttachableRefFull, SpeedBump};


impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    /// Construct an [`AttachableRefFull`] with the given backing data and slot. The slot may
    /// contain self-references to the given `data: Data` backing data.
    ///
    /// ***WARNING***: This is ***extremely*** `unsafe`, and is included *solely* for the sake
    /// of flexibility for other experienced `unsafe` authors. Use the safe constructors and
    /// mapping methods of this type if at all possible.
    ///
    /// # Safety
    /// See [`stable_view::concepts_and_safety`] for the "stable" jargon used below.
    ///
    /// The exact safety requirements of this function depend on the state of `slot`.
    ///
    /// If `slot` is in the [`NoRef`] state, then it ***must*** not contain stable (non-long-lived)
    /// data.
    ///
    /// If `slot` is in the [`Ref`] state, then it any stable data it contains must not be
    /// invalidated by performing the three kinds of operations permitted by [`StableView::view`]
    /// on `data`. (Technically, then, that stable data need not have been obtained from
    /// [`stable_view::StableView`] views of `data`, but it needs to behave as though it were.)
    ///
    /// If `slot` is in the [`RefMut`] state, then it any stable data it contains must not be
    /// invalidated by performing the three kinds of operations permitted by
    /// [`StableViewMut::view_mut`] on `data`. (That stable data need not have been obtained from
    /// [`stable_view::StableViewMut`] views of `data`, but it needs to behave as though it were.)
    ///
    /// # Robust Guarantee
    /// This function only moves `data`, and does not unwind (which could cause `data` to be
    /// unexpectedly dropped). Therefore, it does not invalidate any `'stable` data in `slot`.
    ///
    /// [`NoRef`]: crate::slot::SelfRefCases::NoRef
    /// [`Ref`]: crate::slot::SelfRefCases::Ref
    /// [`RefMut`]: crate::slot::SelfRefCases::RefMut
    /// [`StableView::view`]: stable_view::StableView::view
    /// [`StableViewMut::view_mut`]: stable_view::StableViewMut::view_mut
    #[inline]
    #[must_use]
    pub const unsafe fn from_slot_unchecked<'stable>(
        data: Data,
        slot: SelfRefSlot<'stable, 'upper, N, R, M>,
    ) -> Self
    where
        'data: 'stable,
    {
        let data = SpeedBump {
            speed_bump: data,
        };
        unsafe { Self::from_slot(data, slot) }
    }

    /// Unsafely get the backing `Data` and the slot for self-references of `self`.
    ///
    /// ***WARNING***: This is ***extremely*** `unsafe`, and is included *solely* for the sake
    /// of flexibility for other experienced `unsafe` authors. Use the safe destructors and
    /// mapping methods of this type if at all possible.
    ///
    /// # Safety
    /// See [`stable_view::concepts_and_safety`] for the "stable" jargon used below, and review
    /// its documentation for information about how to avoid invalidating self-references.
    ///
    /// `self`'s slot for self-references and backing data are returned. You are forbidden from
    /// performing operations on the returned `Data` value which invalidate stable self-references
    /// (in the returned slot) to that `Data`.
    ///
    /// This condition is vacuously fulfilled when the slot for self-references currently has no
    /// stable self-references to the backing data.
    ///
    /// In particular, this condition is certainly met when `Data` is a value like `()` or
    /// `Option::None` (which are incapable of providing stable references) or when `self` is in the
    /// [`NoRef`] state.
    ///
    /// [`NoRef`]: crate::slot::SelfRefCases::NoRef
    #[inline]
    #[must_use]
    pub unsafe fn into_raw_pieces(self) -> (Data, SelfRefSlot<'data, 'upper, N, R, M>) {
        // SAFETY INVARIANT: Our caller has `unsafe`ly asserted that `self.slot` has no `'stable`
        // references to `self.data`, so *no* manipulation of `self.data` (during at least `'data`)
        // can invalidate the `slot` value. Therefore, completely exposing `self.data` to
        // the caller's code is sound.
        let data = self.data.speed_bump;

        // SAFETY: As robustly guaranteed by the `slot` field, the erased lifetime can be soundly
        // unerased into any `'stable` lifetime such that `'data: 'stable` and, at least until
        // `'stable` ends, `self.data` is not manipulated in a way that invalidates `self.slot`.
        // Our caller has `unsafe`ly asserted that `self.slot` has no `'stable` references to
        // `self.data`, so *no* manipulation of `self.data` (during at least `'data`) can invalidate
        // `self.slot`. Therefore, we can soundly choose `'stable = 'data`.
        let slot = unsafe { self.slot.into_unerased::<'data>() };

        (data, slot)
    }
}
