// See https://linebender.org/blog/doc-include for this README inclusion strategy
// File links are not supported by rustdoc
//!
//! [LICENSE-APACHE]: https://github.com/robofinch/lifetime-foundry/blob/main/LICENSE-APACHE
//! [LICENSE-MIT]: https://github.com/robofinch/lifetime-foundry/blob/main/LICENSE-MIT
//!
//! [`StableViewer`]: StableViewer
//! [`StableViewer<'a, 'stable, 'data, Data>`]: StableViewer
//! [`StableViewer::view`]: StableViewer::view
//! [`StableViewerMut`]: StableViewerMut
//! [`StableViewerMut::view_mut`]: StableViewerMut::view_mut
//!
//! [`StableView`]: StableView
//! [`StableViewMut`]: StableViewMut
//! [`StableClone`]: StableClone
//!
//! [`ReferenceViewKind`]: ReferenceViewKind
//! [`StableReferenceView`]: StableReferenceView
//! [`StableReferenceViewMut`]: StableReferenceViewMut
//!
//! [`RecursiveViewKind`]: RecursiveViewKind
//! [`RecursiveViewKind<(ViewT, ViewE)>`]: RecursiveViewKind
//! [`recursive_view!`]: recursive_view
//!
//! [`UnstableViewKind`]: UnstableViewKind
//!
//! [`DefaultViewKind`]: DefaultViewKind
//! [`DefaultStableView`]: DefaultStableView
//! [`DefaultStableViewMut`]: DefaultStableViewMut
//!
//! [`variance-family`]: variance_family
//!
//! <style>
//! .rustdoc-hidden { display: none; }
//! </style>
#![cfg_attr(doc, doc = include_str!("../README.md"))]

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;


mod traits;
mod macros;
mod aliasable;
mod viewer;
mod viewer_families;

mod view_kinds;
mod provided_view_kinds;

mod core_impls;

#[cfg(feature = "alloc")]
mod alloc_impls;

#[cfg(feature = "std")]
mod std_impls;

mod other_impls;


#[doc(hidden)]
pub mod __macro {
    pub use variance_family;

    pub use crate::macros::unsafe_recursive_view;
}


pub use self::{
    aliasable::{AliasableRefMut, VaryingAliasableRefMut},
    provided_view_kinds::{UnitViewKind, UnstableViewKind, ViewerViewKind},
    traits::{CustomView, CustomViewMut, StableClone, StableView, StableViewMut},
    view_kinds::{
        CollectionViewKind, DefaultStableView, DefaultStableViewMut, DefaultViewKind,
        RecursiveViewKind, ReferenceViewKind, StableReferenceView, StableReferenceViewMut, View,
        ViewMut, ZeroSizedViewKind,
    },
    viewer::{StableViewer, StableViewerMut},
    viewer_families::{VaryingStableViewer, VaryingStableViewerMut},
};
#[cfg(feature = "alloc")]
pub use self::aliasable::AliasableBox;


pub mod concepts_and_safety;
