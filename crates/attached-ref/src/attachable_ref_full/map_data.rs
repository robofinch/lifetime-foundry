#![expect(unsafe_code, reason = "perform unsafe lifetime erasure and extension of self-refs")]

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
use crate::mapping::{ErasedAliasableBox, ErasedBox, ErasedRc};
#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
use crate::mapping::ErasedArc;

use super::full_struct::{AttachableRefFull, SpeedBump};


impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    #[inline]
    #[must_use]
    pub fn wrap_data_in_some(self) -> AttachableRefFull<'data, 'upper, N, R, M, Option<Data>> {
        unsafe { self.map_data(Some) }
    }

    #[inline]
    #[must_use]
    pub fn wrap_data_in_ok<E>(self) -> AttachableRefFull<'data, 'upper, N, R, M, Result<Data, E>> {
        unsafe { self.map_data(Ok) }
    }

    #[inline]
    #[must_use]
    pub fn wrap_data_in_err<T>(self) -> AttachableRefFull<'data, 'upper, N, R, M, Result<T, Data>> {
        unsafe { self.map_data(Err) }
    }

    #[cfg(feature = "either")]
    #[inline]
    #[must_use]
    pub fn wrap_data_in_left<Right>(
        self,
    ) -> AttachableRefFull<'data, 'upper, N, R, M, Either<Data, Right>> {
        unsafe { self.map_data(Either::Left) }
    }

    #[cfg(feature = "either")]
    #[inline]
    #[must_use]
    pub fn wrap_data_in_right<Left>(
        self,
    ) -> AttachableRefFull<'data, 'upper, N, R, M, Either<Left, Data>> {
        unsafe { self.map_data(Either::Right) }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    #[must_use]
    pub fn wrap_data_in_box(
        self
    ) -> AttachableRefFull<'data, 'upper, N, R, M, AliasableBox<Data>> {

        let mut allocation = Box::new_uninit();

        let map = |data| {
            allocation.write(data);

            let boxed = unsafe { allocation.assume_init() };

            AliasableBox::from_box(boxed)
        };

        unsafe { self.map_data(map) }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    #[must_use]
    pub fn wrap_data_in_std_box(
        self
    ) -> AttachableRefFull<'data, 'upper, N, R, M, Box<Data>> {

        let mut allocation = Box::new_uninit();

        let map = |data| {
            allocation.write(data);

            unsafe { allocation.assume_init() }
        };

        unsafe { self.map_data(map) }
    }

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

            unsafe { allocation.assume_init() }
        };

        unsafe { self.map_data(map) }
    }

    #[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
    #[inline]
    #[must_use]
    pub fn wrap_data_in_arc(
        self
    ) -> AttachableRefFull<'data, 'upper, N, R, M, Arc<Data>> {

        let mut allocation = Arc::new_uninit();

        let map = |data| {
            let dest = Arc::get_mut(&mut allocation);
            let dest = unsafe { dest.unwrap_unchecked() };
            dest.write(data);

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
    #[inline]
    #[must_use]
    pub fn erase_data(
        self,
    ) -> AttachableRefFull<'data, 'upper, N, R, M, ErasedAliasableBox> {
        let map = |data: AliasableBox<Data>| -> ErasedAliasableBox {
            let raw = AliasableBox::into_raw(data);
            unsafe { AliasableBox::from_raw(raw) }
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
    #[inline]
    #[must_use]
    pub fn erase_data(
        self,
    ) -> AttachableRefFull<'data, 'upper, N, R, M, ErasedBox> {
        let map = |data: Box<Data>| -> ErasedBox {
            let raw = Box::into_raw(data);
            unsafe { Box::from_raw(raw) }
        };

        unsafe { self.map_data(map) }
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
    #[inline]
    #[must_use]
    pub fn erase_data(
        self,
    ) -> AttachableRefFull<'data, 'upper, N, R, M, ErasedRc> {
        let map = |data: Rc<Data>| -> ErasedRc {
            let raw = Rc::into_raw(data);
            unsafe { Rc::from_raw(raw) }
        };

        unsafe { self.map_data(map) }
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
    #[inline]
    #[must_use]
    pub fn erase_data(
        self,
    ) -> AttachableRefFull<'data, 'upper, N, R, M, ErasedArc> {
        let map = |data: Arc<Data>| -> ErasedArc {
            let raw = Arc::into_raw(data);
            unsafe { Arc::from_raw(raw) }
        };

        unsafe { self.map_data(map) }
    }
}

impl<'data, 'upper, N, R, M, Data> AttachableRefFull<'data, 'upper, N, R, M, Data>
where
    'upper: 'data,
    R:      LendFamily<&'upper ()>,
    M:      LendFamily<&'upper ()>,
{
    #[inline]
    #[must_use]
    pub fn try_change_data<F: FnOnce(Data) -> Data>(mut self, f: F) -> Self {
        if matches!(self.get(), SelfRefCases::NoRef(_)) {
            let data = self.data.speed_bump_inner;

            self.data.speed_bump_inner = f(data);
        }

        self
    }

    #[inline]
    #[must_use]
    pub unsafe fn map_data<NewData, F: FnOnce(Data) -> NewData>(
        self,
        f: F,
    ) -> AttachableRefFull<'data, 'upper, N, R, M, NewData> {
        let slot = MaybeUninit::new(self.slot);
        let data = self.data.speed_bump_inner;

        let data = SpeedBump {
            speed_bump_inner: f(data),
        };

        let slot = unsafe { slot.assume_init() };

        AttachableRefFull {
            slot,
            variance: self.variance,
            data,
        }
    }

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
        let data = self.data.speed_bump_inner;

        let result = f(data);

        let slot = unsafe { slot.assume_init() };

        match result {
            Ok((new_data, ok)) => {
                let new_data = SpeedBump {
                    speed_bump_inner: new_data,
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
                self.data.speed_bump_inner = old_data;

                Err((self, err))
            }
        }
    }
}
