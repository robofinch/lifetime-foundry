#![warn(
    clippy::missing_inline_in_public_items,
    reason = "this is basically a generic wrapper type",
)]

#![expect(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks, reason = "TODO")]

mod full_struct;

mod init;
mod destruct;

mod access_immut;
mod access_mut;

mod map_slot;
mod map_data;

mod trait_impls;

mod non_mut;
mod always_owned;
mod option_data;


pub use self::full_struct::AttachableRefFull;
