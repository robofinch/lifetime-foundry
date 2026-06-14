#![expect(clippy::undocumented_unsafe_blocks, reason = "TODO")]
#![expect(missing_docs, clippy::missing_docs_in_private_items, reason = "TODO")]

mod branded_tokens;
mod mapped_slot;
mod mapping;


pub use self::{
    branded_tokens::{
        BrandFamily, Branded, CloneDataToken, DataToken, NoRefMapToken, NonMutMapClonedToken,
        RefMapToken, RefMutMapToken, TakeDataToken,
    },
    mapped_slot::{MappedSlot, VaryingMappedSlot},
    mapping::{MapCases, MapClonedCases},
};
pub(crate) use self::mapping::{map_slot_cloned_impl, map_slot_impl};
