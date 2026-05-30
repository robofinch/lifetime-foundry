mod traits_and_types;
mod impls;

#[cfg(feature = "alloc")]
mod alloc_impls;

#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
mod arc_impls;

mod stable_clone_raw;
mod non_null_option;

mod non_null_option_impls;


pub use self::traits_and_types::{
    BlanketMapData, DynDrop, MapDataStrict, MapDataStrictest, MapViaAlloc, MapViaAltMove,
    MapViaMove, MapViaMoveInto,
};

#[cfg(feature = "alloc")]
pub use self::alloc_impls::RcUninit;
#[cfg(feature = "alloc")]
pub use self::traits_and_types::{ErasedBox, ErasedRc};

#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
pub use self::arc_impls::ArcUninit;
#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
pub use self::traits_and_types::ErasedArc;
