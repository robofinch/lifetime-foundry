#![expect(unsafe_code, reason = "map `Data` without invalidating `'stable` views to its contents")]
#![warn(
    clippy::missing_inline_in_public_items,
    reason = "almost every mapping function is tiny",
)]

#![expect(clippy::undocumented_unsafe_blocks, reason = "TODO")]


use core::mem::MaybeUninit;

#[cfg(feature = "either")]
use either::{Either, Either::Left, Either::Right};

use stable_view::AliasableRefMut;

use super::traits_and_types::{MapDataStrictest, MapViaAltMove, MapViaMove, MapViaMoveInto};


unsafe impl<T> MapDataStrictest<'_, T, T> for MapViaMove {
    #[inline]
    fn map_data_strictest(self, data: T) -> T {
        data
    }
}

unsafe impl<T> MapDataStrictest<'_, T, Option<T>> for MapViaMove {
    #[inline]
    fn map_data_strictest(self, data: T) -> Option<T> {
        Some(data)
    }
}

unsafe impl<T, E> MapDataStrictest<'_, T, Result<T, E>> for MapViaMove {
    #[inline]
    fn map_data_strictest(self, data: T) -> Result<T, E> {
        Ok(data)
    }
}

#[cfg(feature = "either")]
unsafe impl<L, R> MapDataStrictest<'_, L, Either<L, R>> for MapViaMove {
    #[inline]
    fn map_data_strictest(self, data: L) -> Either<L, R> {
        Left(data)
    }
}

unsafe impl<T, E> MapDataStrictest<'_, E, Result<T, E>> for MapViaAltMove {
    #[inline]
    fn map_data_strictest(self, data: E) -> Result<T, E> {
        Err(data)
    }
}

#[cfg(feature = "either")]
unsafe impl<L, R> MapDataStrictest<'_, R, Either<L, R>> for MapViaAltMove {
    #[inline]
    fn map_data_strictest(self, data: R) -> Either<L, R> {
        Right(data)
    }
}

// These next eight implementations basically just move the backing data into a long-lived stack
// frame. Not really much else can be done in `core`.

unsafe impl<'b, 'new_data, T> MapDataStrictest<'new_data, T, &'b mut T>
for MapViaMoveInto<&'b mut T>
where
    'b: 'new_data,
    T:  'b,
{
    #[inline]
    fn map_data_strictest(self, data: T) -> &'b mut T {
        *self.0 = data;
        self.0
    }
}

unsafe impl<'b, 'new_data, T> MapDataStrictest<'new_data, T, &'b mut T>
for MapViaMoveInto<AliasableRefMut<'b, T>>
where
    'b: 'new_data,
    T:  'b,
{
    #[inline]
    fn map_data_strictest(mut self, data: T) -> &'b mut T {
        *self.0 = data;
        AliasableRefMut::into_mut(self.0)
    }
}

unsafe impl<'b, 'new_data, T> MapDataStrictest<'new_data, T, AliasableRefMut<'b, T>>
for MapViaMoveInto<&'b mut T>
where
    'b: 'new_data,
    T:  'b,
{
    #[inline]
    fn map_data_strictest(self, data: T) -> AliasableRefMut<'b, T> {
        *self.0 = data;
        AliasableRefMut::from_mut(self.0)
    }
}

unsafe impl<'b, 'new_data, T> MapDataStrictest<'new_data, T, AliasableRefMut<'b, T>>
for MapViaMoveInto<AliasableRefMut<'b, T>>
where
    'b: 'new_data,
    T:  'b,
{
    #[inline]
    fn map_data_strictest(mut self, data: T) -> AliasableRefMut<'b, T> {
        *self.0 = data;
        self.0
    }
}

unsafe impl<'b, 'new_data, T> MapDataStrictest<'new_data, T, &'b mut T>
for MapViaMoveInto<&'b mut MaybeUninit<T>>
where
    'b: 'new_data,
    T:  'b,
{
    #[inline]
    fn map_data_strictest(self, data: T) -> &'b mut T {
        self.0.write(data)
    }
}

unsafe impl<'b, 'new_data, T> MapDataStrictest<'new_data, T, &'b mut T>
for MapViaMoveInto<AliasableRefMut<'b, MaybeUninit<T>>>
where
    'b: 'new_data,
    T:  'b,
{
    #[inline]
    fn map_data_strictest(self, data: T) -> &'b mut T {
        AliasableRefMut::into_mut(self.0).write(data)
    }
}

unsafe impl<'b, 'new_data, T> MapDataStrictest<'new_data, T, AliasableRefMut<'b, T>>
for MapViaMoveInto<&'b mut MaybeUninit<T>>
where
    'b: 'new_data,
    T:  'b,
{
    #[inline]
    fn map_data_strictest(self, data: T) -> AliasableRefMut<'b, T> {
        AliasableRefMut::from_mut(self.0.write(data))
    }
}

unsafe impl<'b, 'new_data, T> MapDataStrictest<'new_data, T, AliasableRefMut<'b, T>>
for MapViaMoveInto<AliasableRefMut<'b, MaybeUninit<T>>>
where
    'b: 'new_data,
    T:  'b,
{
    #[inline]
    fn map_data_strictest(self, data: T) -> AliasableRefMut<'b, T> {
        AliasableRefMut::from_mut(AliasableRefMut::into_mut(self.0).write(data))
    }
}
