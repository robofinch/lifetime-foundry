#![expect(unsafe_code, reason = "map `Data` without invalidating `'stable` views to its contents")]

#![expect(missing_docs, clippy::undocumented_unsafe_blocks, clippy::missing_safety_doc, reason = "TODO")]

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, rc::Rc};
#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
use alloc::sync::Arc;


pub unsafe trait MapDataStrict<'new_data, Data, NewData> {
    #[must_use]
    fn map_data_strict(self, data: Data) -> NewData;
}

pub unsafe trait MapDataStrictest<'new_data, Data, NewData>:
    MapDataStrict<'new_data, Data, NewData>
{
    #[must_use]
    fn map_data_strictest(self, data: Data) -> NewData;
}

pub trait BlanketMapData {}

unsafe impl<'new_data, Data, NewData, M> MapDataStrict<'new_data, Data, NewData> for M
where
    M: MapDataStrictest<'new_data, Data, NewData> + BlanketMapData,
{
    #[inline]
    fn map_data_strict(self, data: Data) -> NewData {
        self.map_data_strictest(data)
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

impl BlanketMapData for MapViaAlloc {}

#[derive(Debug)]
#[repr(transparent)]
pub struct MapViaMoveInto<A>(pub A);

impl<A> BlanketMapData for MapViaMoveInto<A> {}

pub trait DynDrop: 'static {}

impl<T: ?Sized + 'static> DynDrop for T {}

#[cfg(feature = "alloc")]
pub type ErasedBox = Box<dyn DynDrop>;

#[cfg(feature = "alloc")]
pub type ErasedRc = Rc<dyn DynDrop>;

#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
pub type ErasedArc = Arc<dyn DynDrop + Send + Sync>;
