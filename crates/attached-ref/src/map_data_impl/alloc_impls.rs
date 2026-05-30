#![expect(unsafe_code, reason = "map `Data` without invalidating `'stable` views to its contents")]
#![warn(
    clippy::missing_inline_in_public_items,
    reason = "almost every mapping function is tiny",
)]

#![expect(missing_docs, clippy::undocumented_unsafe_blocks, reason = "TODO")]


use core::mem::MaybeUninit;
use core::fmt::{Debug, Formatter, Result as FmtResult};
use alloc::{boxed::Box, rc::Rc};

use super::traits_and_types::{DynDrop, MapDataStrictest, MapViaAlloc, MapViaMove, MapViaMoveInto};


pub struct RcUninit<T>(Rc<MaybeUninit<T>>);

impl<T> RcUninit<T> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Rc::new_uninit())
    }

    #[inline]
    #[must_use]
    pub fn inner(&self) -> &MaybeUninit<T> {
        &self.0
    }

    #[inline]
    #[must_use]
    pub fn inner_mut(&mut self) -> &mut MaybeUninit<T> {
        let inner = Rc::get_mut(&mut self.0);
        unsafe { inner.unwrap_unchecked() }
    }

    #[inline]
    #[must_use]
    pub fn into_inner(self) -> Rc<MaybeUninit<T>> {
        self.0
    }

    #[inline]
    #[must_use]
    pub fn write(mut self, data: T) -> Rc<T> {
        self.inner_mut().write(data);

        unsafe { self.into_inner().assume_init() }
    }
}

impl<T> Debug for RcUninit<T> {
    #[expect(clippy::missing_inline_in_public_items, reason = "in formatting, size matters more")]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("RcUninit").finish_non_exhaustive()
    }
}

impl<T> Default for RcUninit<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}


unsafe impl<T: 'static> MapDataStrictest<'_, Box<T>, Box<dyn DynDrop>> for MapViaMove {
    #[inline]
    fn map_data_strictest(self, data: Box<T>) -> Box<dyn DynDrop> {
        data
    }
}

unsafe impl<T: 'static> MapDataStrictest<'_, Rc<T>, Rc<dyn DynDrop>> for MapViaMove {
    #[inline]
    fn map_data_strictest(self, data: Rc<T>) -> Rc<dyn DynDrop> {
        data
    }
}

unsafe impl<T: 'static + Send + Sync> MapDataStrictest<'_, Box<T>, Box<dyn DynDrop + Send + Sync>>
for MapViaMove
{
    #[inline]
    fn map_data_strictest(self, data: Box<T>) -> Box<dyn DynDrop + Send + Sync> {
        data
    }
}

unsafe impl<T> MapDataStrictest<'_, T, Box<T>> for MapViaAlloc {
    #[inline]
    fn map_data_strictest(self, data: T) -> Box<T> {
        Box::new(data)
    }
}

unsafe impl<T> MapDataStrictest<'_, T, Rc<T>> for MapViaAlloc {
    #[inline]
    fn map_data_strictest(self, data: T) -> Rc<T> {
        Rc::new(data)
    }
}

unsafe impl<T> MapDataStrictest<'_, T, Box<T>> for MapViaMoveInto<Box<T>> {
    #[inline]
    fn map_data_strictest(mut self, data: T) -> Box<T> {
        *self.0 = data;
        self.0
    }
}

unsafe impl<T> MapDataStrictest<'_, T, Box<T>> for MapViaMoveInto<Box<MaybeUninit<T>>> {
    #[inline]
    fn map_data_strictest(mut self, data: T) -> Box<T> {
        self.0.write(data);

        unsafe { self.0.assume_init() }
    }
}

unsafe impl<T> MapDataStrictest<'_, T, Rc<T>> for MapViaMoveInto<RcUninit<T>> {
    #[inline]
    fn map_data_strictest(self, data: T) -> Rc<T> {
        self.0.write(data)
    }
}
