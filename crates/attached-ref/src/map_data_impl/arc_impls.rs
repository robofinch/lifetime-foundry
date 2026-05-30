#![expect(unsafe_code, reason = "map `Data` without invalidating `'stable` views to its contents")]
#![warn(
    clippy::missing_inline_in_public_items,
    reason = "almost every mapping function is tiny",
)]

#![expect(missing_docs, clippy::undocumented_unsafe_blocks, reason = "TODO")]

use core::mem::MaybeUninit;
use core::fmt::{Debug, Formatter, Result as FmtResult};
use alloc::sync::Arc;

use super::traits_and_types::{DynDrop, MapDataStrictest, MapViaAlloc, MapViaMove, MapViaMoveInto};


pub struct ArcUninit<T>(Arc<MaybeUninit<T>>);

impl<T> ArcUninit<T> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Arc::new_uninit())
    }

    #[inline]
    #[must_use]
    pub fn inner(&self) -> &MaybeUninit<T> {
        &self.0
    }

    #[inline]
    #[must_use]
    pub fn inner_mut(&mut self) -> &mut MaybeUninit<T> {
        let inner = Arc::get_mut(&mut self.0);
        unsafe { inner.unwrap_unchecked() }
    }

    #[inline]
    #[must_use]
    pub fn into_inner(self) -> Arc<MaybeUninit<T>> {
        self.0
    }

    #[inline]
    #[must_use]
    pub fn write(mut self, data: T) -> Arc<T> {
        self.inner_mut().write(data);

        unsafe { self.into_inner().assume_init() }
    }
}

impl<T> Debug for ArcUninit<T> {
    #[expect(clippy::missing_inline_in_public_items, reason = "in formatting, size matters more")]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ArcUninit").finish_non_exhaustive()
    }
}

impl<T> Default for ArcUninit<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}


unsafe impl<T: 'static> MapDataStrictest<'_, Arc<T>, Arc<dyn DynDrop>> for MapViaMove {
    #[inline]
    fn map_data_strictest(self, data: Arc<T>) -> Arc<dyn DynDrop> {
        data
    }
}

unsafe impl<T: 'static + Send + Sync> MapDataStrictest<'_, Arc<T>, Arc<dyn DynDrop + Send + Sync>>
for MapViaMove
{
    #[inline]
    fn map_data_strictest(self, data: Arc<T>) -> Arc<dyn DynDrop + Send + Sync> {
        data
    }
}

unsafe impl<T> MapDataStrictest<'_, T, Arc<T>> for MapViaAlloc {
    #[inline]
    fn map_data_strictest(self, data: T) -> Arc<T> {
        Arc::new(data)
    }
}

unsafe impl<T> MapDataStrictest<'_, T, Arc<T>> for MapViaMoveInto<ArcUninit<T>> {
    #[inline]
    fn map_data_strictest(self, data: T) -> Arc<T> {
        self.0.write(data)
    }
}
