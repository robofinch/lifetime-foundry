#![expect(unsafe_code, reason = "map `Data` without invalidating `'stable` views to its contents")]
#![warn(
    clippy::missing_inline_in_public_items,
    reason = "almost every mapping function is tiny",
)]

#![expect(missing_docs, clippy::undocumented_unsafe_blocks, clippy::missing_safety_doc, reason = "TODO")]

use core::marker::PhantomData;
use core::fmt::{Debug, Formatter, Result as FmtResult};


pub trait MapData<'new_data, Data, NewData> {
    #[must_use]
    fn map_data(self, data: Data) -> NewData;
}

pub unsafe trait MapDataStrict<'new_data, Data, NewData> {
    #[must_use]
    fn map_data_strict(self, data: Data) -> NewData;
}

pub unsafe trait MapDataStrictest<'new_data, Data, NewData> {
    #[must_use]
    fn map_data_strictest(self, data: Data) -> NewData;
}

pub trait BlanketMapData {}

impl<'new_data, Data, NewData, M> MapData<'new_data, Data, NewData> for M
where
    M: MapDataStrict<'new_data, Data, NewData> + BlanketMapData,
{
    #[inline]
    fn map_data(self, data: Data) -> NewData {
        self.map_data_strict(data)
    }
}

unsafe impl<'new_data, Data, NewData, M> MapDataStrict<'new_data, Data, NewData> for M
where
    M: MapDataStrictest<'new_data, Data, NewData> + BlanketMapData,
{
    #[inline]
    fn map_data_strict(self, data: Data) -> NewData {
        self.map_data_strictest(data)
    }
}


/// Compose two implementations of [`MapData`], [`MapDataStrict`], or [`MapDataStrictest`].
pub struct ComposeMaps<M1, D, M2>(pub M1, pub PhantomData<D>, pub M2);

impl<M1: Clone, D, M2: Clone> Clone for ComposeMaps<M1, D, M2> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData, self.2.clone())
    }
}

impl<M1: Copy, D, M2: Copy> Copy for ComposeMaps<M1, D, M2> {}

impl<M1: Debug, D, M2: Debug> Debug for ComposeMaps<M1, D, M2> {
    #[expect(clippy::missing_inline_in_public_items, reason = "in formatting, size matters more")]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("ComposeMaps")
            .field(&self.0)
            .field(&self.1)
            .field(&self.2)
            .finish()
    }
}

impl<M1: Default, D, M2: Default> Default for ComposeMaps<M1, D, M2> {
    #[inline]
    fn default() -> Self {
        Self(M1::default(), PhantomData, M2::default())
    }
}

impl<'new_data, Data, M1, D, M2, NewData> MapData<'new_data, Data, NewData>
for ComposeMaps<M1, D, M2>
where
    M1: MapData<'new_data, Data, D>,
    M2: MapData<'new_data, D, NewData>,
{
    #[inline]
    fn map_data(self, data: Data) -> NewData {
        self.2.map_data(self.0.map_data(data))
    }
}

unsafe impl<'new_data, Data, M1, D, M2, NewData> MapDataStrict<'new_data, Data, NewData>
for ComposeMaps<M1, D, M2>
where
    M1: MapDataStrict<'new_data, Data, D>,
    M2: MapDataStrict<'new_data, D, NewData>,
{
    #[inline]
    fn map_data_strict(self, data: Data) -> NewData {
        self.2.map_data_strict(self.0.map_data_strict(data))
    }
}

unsafe impl<'new_data, Data, M1, D, M2, NewData> MapDataStrictest<'new_data, Data, NewData>
for ComposeMaps<M1, D, M2>
where
    M1: MapDataStrictest<'new_data, Data, D>,
    M2: MapDataStrictest<'new_data, D, NewData>,
{
    #[inline]
    fn map_data_strictest(self, data: Data) -> NewData {
        self.2.map_data_strictest(self.0.map_data_strictest(data))
    }
}


#[derive(Debug, Default, Clone, Copy)]
pub struct MapViaMove;

impl BlanketMapData for MapViaMove {}

#[derive(Debug, Default, Clone, Copy)]
pub struct MapViaAltMove;

impl BlanketMapData for MapViaAltMove {}

#[derive(Debug, Default, Clone, Copy)]
pub struct MapViaAlloc;

#[derive(Debug)]
#[repr(transparent)]
pub struct MapViaMoveInto<A>(pub A);

impl<A> BlanketMapData for MapViaMoveInto<A> {}

pub trait DynDrop: 'static {}

impl<T: ?Sized + 'static> DynDrop for T {}
