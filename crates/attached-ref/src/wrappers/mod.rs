//! Transparent wrappers around [`AttachableRefFull`].
//!
//! [`AttachableRefFull`]: crate::attachable_ref_full::AttachableRefFull

mod attachable_ref;
mod attachable_ref_mut;
mod attached_ref;
mod attached_ref_mut;


pub use self::{
    attachable_ref::AttachableRef,
    attachable_ref_mut::AttachableRefMut,
    attached_ref::AttachedRef,
    attached_ref_mut::AttachedRefMut,
};
