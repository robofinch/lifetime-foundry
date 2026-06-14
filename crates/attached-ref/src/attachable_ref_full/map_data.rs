//! Temporary organization module.

use core::mem::MaybeUninit;
#[cfg(feature = "alloc")]
use alloc::{boxed::Box, rc::Rc};
#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
use alloc::sync::Arc;

#[cfg(feature = "either")]
use either::Either;

#[cfg(feature = "alloc")]
use stable_view::AliasableBox;
use variance_family::LendFamily;

use crate::slot::SelfRefCases;
#[cfg(feature = "alloc")]
use crate::erased::{DynDestruct, ErasedAliasableBox, ErasedBox, ErasedRc};
#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
use crate::erased::ErasedArc;

use super::full_struct::{AttachableRefFull, SpeedBump};


impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    /// Wrap the backing data in [`Some`]. See also [`new_owned_ref`]; together, either owned or
    /// self-referential data are supported by `Option<Data>`.
    ///
    /// [`new_owned_ref`]: AttachableRefFull::new_owned_ref
    #[inline]
    #[must_use]
    pub fn wrap_data_in_some(self) -> AttachableRefFull<'data, 'upper, N, R, M, Option<Data>> {
        unsafe { self.map_data(Some) }
    }

    /// Wrap the backing data in [`Ok`].
    ///
    /// This may not be particularly useful, but it *is* sound.
    ///
    /// With the `either` feature, `Self::wrap_data_in_left` and `Self::wrap_data_in_right` have
    /// better semantics for mixing `AttachableRefFull` values with different data sources; try to
    /// only use `Result` when there are success/error semantics.
    #[inline]
    #[must_use]
    pub fn wrap_data_in_ok<E>(self) -> AttachableRefFull<'data, 'upper, N, R, M, Result<Data, E>> {
        unsafe { self.map_data(Ok) }
    }

    /// Wrap the backing data in [`Err`].
    ///
    /// This may not be particularly useful, but it *is* sound.
    ///
    /// With the `either` feature, `Self::wrap_data_in_left` and `Self::wrap_data_in_right` have
    /// better semantics for mixing `AttachableRefFull` values with different data sources; try to
    /// only use `Result` when there are success/error semantics.
    #[inline]
    #[must_use]
    pub fn wrap_data_in_err<T>(self) -> AttachableRefFull<'data, 'upper, N, R, M, Result<T, Data>> {
        unsafe { self.map_data(Err) }
    }

    /// Wrap the backing data in [`Either::Left`].
    ///
    /// Together with [`Self::wrap_data_in_right`], `AttachableRefFull` values with different
    /// `Data` sources can be merged into the same type while maintaining full information
    /// about the actual source `Data` type (unlike the various `AttachableRefFull::erase_*_data`
    /// methods).
    #[cfg(feature = "either")]
    #[inline]
    #[must_use]
    pub fn wrap_data_in_left<Right>(
        self,
    ) -> AttachableRefFull<'data, 'upper, N, R, M, Either<Data, Right>> {
        unsafe { self.map_data(Either::Left) }
    }

    /// Wrap the backing data in [`Either::Right`].
    ///
    /// Together with [`Self::wrap_data_in_left`], `AttachableRefFull` values with different
    /// `Data` sources can be merged into the same type while maintaining full information
    /// about the actual source `Data` type (unlike the various `AttachableRefFull::erase_*_data`
    /// methods).
    #[cfg(feature = "either")]
    #[inline]
    #[must_use]
    pub fn wrap_data_in_right<Left>(
        self,
    ) -> AttachableRefFull<'data, 'upper, N, R, M, Either<Left, Data>> {
        unsafe { self.map_data(Either::Right) }
    }

    /// Wrap the backing data in [`AliasableBox`], enabling `&'stable Data` self-references or
    /// later application of [`erase_boxed_data`].
    ///
    /// [`erase_boxed_data`]: AttachableRefFull::erase_boxed_data
    #[cfg(feature = "alloc")]
    #[inline]
    #[must_use]
    pub fn wrap_data_in_box(
        self
    ) -> AttachableRefFull<'data, 'upper, N, R, M, AliasableBox<Data>> {
        let mut allocation = Box::new_uninit();

        let map = |data| {
            allocation.write(data);

            // SAFETY: We called `allocation.write(_)` to fully initialize its contents.
            let boxed = unsafe { allocation.assume_init() };

            AliasableBox::from_box(boxed)
        };

        unsafe { self.map_data(map) }
    }

    /// Wrap the backing data in [`Box`], enabling later application of [`erase_std_boxed_data`].
    ///
    /// Unlike [`AliasableBox`], [`Box`] currently has `noalias` semantics that prevents obtaining
    /// `&'stable Data` self-references from a `Box<Data>`.
    ///
    /// [`erase_std_boxed_data`]: AttachableRefFull::erase_std_boxed_data
    #[cfg(feature = "alloc")]
    #[inline]
    #[must_use]
    pub fn wrap_data_in_std_box(
        self
    ) -> AttachableRefFull<'data, 'upper, N, R, M, Box<Data>> {
        let mut allocation = Box::new_uninit();

        let map = |data| {
            allocation.write(data);

            // SAFETY: We called `allocation.write(_)` to fully initialize its contents.
            unsafe { allocation.assume_init() }
        };

        unsafe { self.map_data(map) }
    }

    /// Wrap the backing data in [`Rc`], enabling `&'stable Data` self-references,
    /// later application of [`erase_rc_data`], and cheap clones.
    ///
    /// [`erase_rc_data`]: AttachableRefFull::erase_rc_data
    #[cfg(feature = "alloc")]
    #[inline]
    #[must_use]
    pub fn wrap_data_in_rc(
        self
    ) -> AttachableRefFull<'data, 'upper, N, R, M, Rc<Data>> {
        let mut allocation = Rc::new_uninit();

        let map = |data| {
            let dest = Rc::get_mut(&mut allocation);
            let dest = unsafe { dest.unwrap_unchecked() };
            dest.write(data);

            // SAFETY: We called `allocation.write(_)` to fully initialize its contents.
            unsafe { allocation.assume_init() }
        };

        unsafe { self.map_data(map) }
    }

    /// Wrap the backing data in [`Arc`], enabling `&'stable Data` self-references,
    /// later application of [`erase_arc_data`], and cheap clones.
    ///
    /// [`erase_arc_data`]: AttachableRefFull::erase_arc_data
    #[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
    #[inline]
    #[must_use]
    pub fn wrap_data_in_arc(
        self
    ) -> AttachableRefFull<'data, 'upper, N, R, M, Arc<Data>> {
        let mut allocation = Arc::new_uninit();

        let map = |data| {
            let dest = Arc::get_mut(&mut allocation);

            // SAFETY: There are no other `Arc` or `Weak` pointers to the same allocation as
            // the `allocation` `Arc`, so `Arc::get_mut` necessarily returned `Some`.
            let dest = unsafe { dest.unwrap_unchecked() };

            dest.write(data);

            // SAFETY: We called `allocation.write(_)` to fully initialize its contents.
            unsafe { allocation.assume_init() }
        };

        unsafe { self.map_data(map) }
    }
}

#[cfg(feature = "alloc")]
impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, AliasableBox<Data>>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    Data:   'static,
{
    /// TODO.
    #[inline]
    #[must_use]
    pub fn erase_boxed_data(
        self,
    ) -> AttachableRefFull<'data, 'upper, N, R, M, ErasedAliasableBox> {
        let map = |data: AliasableBox<Data>| -> ErasedAliasableBox {
            let raw: *mut Data = AliasableBox::into_raw(data);
            let raw: *mut dyn DynDestruct = raw;
            unsafe { AliasableBox::<dyn DynDestruct>::from_raw(raw) }
        };

        unsafe { self.map_data(map) }
    }
}

#[cfg(feature = "alloc")]
impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Box<Data>>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    Data:   'static,
{
    /// TODO.
    #[inline]
    #[must_use]
    pub fn erase_std_boxed_data(
        self,
    ) -> AttachableRefFull<'data, 'upper, N, R, M, ErasedBox> {
        unsafe { self.map_data::<ErasedBox, _>(|data| data) }
    }
}

#[cfg(feature = "alloc")]
impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Rc<Data>>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    Data:   'static,
{
    /// TODO.
    #[inline]
    #[must_use]
    pub fn erase_rc_data(
        self,
    ) -> AttachableRefFull<'data, 'upper, N, R, M, ErasedRc> {
        unsafe { self.map_data::<ErasedRc, _>(|data| data) }
    }
}

#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Arc<Data>>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
    Data:   Send + Sync + 'static,
{
    /// TODO.
    #[inline]
    #[must_use]
    pub fn erase_arc_data(
        self,
    ) -> AttachableRefFull<'data, 'upper, N, R, M, ErasedArc> {
        unsafe { self.map_data::<ErasedArc, _>(|data| data) }
    }
}

impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    /// Swap out the backing data if `self` is in the [`NoRef`] state.
    ///
    /// [`NoRef`]: SelfRefCases::NoRef
    #[inline]
    #[must_use]
    pub fn try_change_data<F: FnOnce(Data) -> Data>(mut self, f: F) -> Self {
        if matches!(self.get(), SelfRefCases::NoRef(_)) {
            let data = self.data.speed_bump;

            self.data.speed_bump = f(data);
        }

        self
    }

    /// Change the backing data type. Generally, soundly calling this function requires either:
    /// - wrapping the original backing `Data` in something to produce the `NewData` value,
    /// - leaking the original backing data,
    /// - perhaps something conditional on `Data`, such as checking whether it's [`None`], in which
    ///   case no stable self-references are possible, and the original backing data could be
    ///   discarded.
    ///
    /// # Safety
    /// TODO; heavily relates to `stable_view`.
    #[inline]
    #[must_use]
    pub unsafe fn map_data<NewData, F: FnOnce(Data) -> NewData>(
        self,
        f: F,
    ) -> AttachableRefFull<'data, 'upper, N, R, M, NewData> {
        let slot = MaybeUninit::new(self.slot);
        let data = self.data.speed_bump;

        let data = SpeedBump {
            speed_bump: f(data),
        };

        let slot = unsafe { slot.assume_init() };

        AttachableRefFull {
            slot,
            variance: self.variance,
            data,
        }
    }

    /// A fallible variant of [`Self::map_data`]. For full flexibility, auxiliary data can be
    /// returned on either success or error.
    ///
    /// The `NewData` or `Data` returned by `f` is used as the backing data of the returned
    /// `AttachableRefFull`.
    ///
    /// Technically, this method is symmetric across `Ok` and `Err` (so `Result` could be used
    /// as a discount `Either`), but *generally*, `f` is intended to return the original `Data`
    /// on error, in which case this method returns the source `self` on error.
    ///
    /// # Errors
    /// Passes up any `E` error returned by `f`, in addition to the `AttachableRefFull` with
    /// the given `Data` as its backing data. (*Generally*, this means returning `self` on error.)
    ///
    /// The `Data`
    /// When the given `f` returns an error, the original `Data` is returned to its place in `self`
    /// and returned alongside the error data returned by `f`.
    ///
    /// # Safety
    /// TODO; heavily relates to `stable_view`.
    #[expect(clippy::type_complexity, reason = "Allow `f` to return auxiliary data")]
    #[inline]
    pub unsafe fn try_map_data<NewData, F, T, E>(
        mut self,
        f: F,
    ) -> Result<(AttachableRefFull<'data, 'upper, N, R, M, NewData>, T), (Self, E)>
    where
        F: FnOnce(Data) -> Result<(NewData, T), (Data, E)>,
    {
        let slot = MaybeUninit::new(self.slot);
        let data = self.data.speed_bump;

        let result = f(data);

        let slot = unsafe { slot.assume_init() };

        match result {
            Ok((new_data, ok)) => {
                let new_data = SpeedBump {
                    speed_bump: new_data,
                };

                let this = AttachableRefFull {
                    slot,
                    variance: self.variance,
                    data:     new_data,
                };

                Ok((this, ok))
            }
            Err((old_data, err)) => {
                self.slot = slot;
                self.data.speed_bump = old_data;

                Err((self, err))
            }
        }
    }
}
