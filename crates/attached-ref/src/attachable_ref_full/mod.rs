//! The fully-flexible [`AttachableRefFull`] type.
#![expect(unsafe_code, reason = "perform unsafe lifetime erasure and extension of self-refs")]
#![warn(
    clippy::missing_inline_in_public_items,
    reason = "this is basically a generic wrapper type",
)]

#![expect(clippy::undocumented_unsafe_blocks, reason = "TODO")]

mod full_struct;

mod init;
mod destruct;

mod access_immut;
mod access_mut;

mod map_slot;
mod map_data;

mod trait_impls;
mod view_impls;

mod non_mut;
mod always_owned;
mod option_data;

mod extreme_unsafe;


pub use self::full_struct::AttachableRefFull;
pub(crate) use self::full_struct::SpeedBump;
