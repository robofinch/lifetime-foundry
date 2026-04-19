//! Implementations for:
//!
//! - `borrow::Cow`,
//! - `collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque}`,
//! - `rc::{Rc, Weak}`,
//! - `string::String`,
//! - `arc::{Arc, Weak}`,
//! - `vec::Vec<T>`.

#![expect(unsafe_code, reason = "implement the unsafe `aliasable-view` traits")]
#![expect(clippy::absolute_paths, reason = "one-off uses of many different types")]
#![warn(clippy::missing_inline_in_public_items, reason = "trivial impls")]

use variance_family::{Unvarying, UpperBound, VaryingRef, VaryingRefMut};

use crate::traits::{
    AliasableClone, AliasableView, AliasableViewMut, IntoAliasable, IntoAliasableMut, View,
    ViewMut,
};


// ================================================================
//  `borrow::Cow`
// ================================================================

// SAFETY:
// We basically have two separate cases, borrowed and owned; if both branches implement
// `AliasableView` with the same sort of view, then the view is valid for at least as long as
// guaranteed, since the only operations which switch the branch are mutations (which are not
// guaranteed to preserve the view).
//
// In the `Borrowed` case, the source value is of type `&'a B`, which implements `AliasableView`
// (so the resulting view won't be invalidated by operations on the source value that it shouldn't
// be invalidated by). The `'a` type parameter of that source value is a lifetime parameter
// of `Self`, so the reference being invalidated after `'a` expires is fine.
//
// In the `Owned` case, the source value is of type `B::Owned`, which likewise implements
// `AliasableView`. Any lifetime parameter in it is part of `B`, which is in turn part of `Self`,
// so it's fine if its view is invalidated after any of its lifetime parameters expire.
unsafe impl<'a, 'upper, B> AliasableView<&'upper ()> for alloc::borrow::Cow<'a, B>
where
    B: 'a + ?Sized + alloc::borrow::ToOwned<Owned: AliasableView<&'upper ()>>,
    for<'b> &'b B: Into<View<'b, B::Owned, &'upper ()>>,
{
    type View = <B::Owned as AliasableView<&'upper ()>>::View;

    #[inline]
    fn view(&self) -> View<'_, Self, &'upper ()> {
        match self {
            Self::Borrowed(borrowed) => borrowed.view().into(),
            Self::Owned(owned) => owned.view(),
        }
    }
}

impl<'a, 'upper, B> IntoAliasable<&'upper ()> for alloc::borrow::Cow<'a, B>
where
    B: 'a + ?Sized + alloc::borrow::ToOwned<Owned: AliasableView<&'upper ()>>,
    for<'b> &'b B: Into<View<'b, B::Owned, &'upper ()>>,
{
    type IntoAliasable = Self;

    #[inline]
    fn into_aliasable(self) -> Self::IntoAliasable {
        self
    }
}


// ================================================================
//  `collections::BTreeMap`
// ================================================================

// ... yeah, much like `[T]`, there's not a good way to represent this in `AliasableView`.

// Welp. Time to change the whole model of this crate.


// ================================================================
//  `vec::Vec`
// ================================================================

unsafe impl<'upper, T: 'upper> AliasableView<&'upper ()> for alloc::vec::Vec<T> {
    type View = VaryingRef<Unvarying<[T]>>;

    #[inline]
    fn view(&self) -> View<'_, Self, &'upper ()> {
        self
    }
}

impl<'upper, T: 'upper> IntoAliasable<&'upper ()> for alloc::vec::Vec<T> {
    type IntoAliasable = Self;

    #[inline]
    fn into_aliasable(self) -> Self::IntoAliasable {
        self
    }
}

unsafe impl<'upper, T: 'upper> AliasableViewMut<&'upper ()> for alloc::vec::Vec<T> {
    type ViewMut = VaryingRefMut<Unvarying<[T]>>;

    #[inline]
    fn view_mut(&mut self) -> ViewMut<'_, Self, &'upper ()> {
        self
    }
}

impl<'upper, T: 'upper> IntoAliasableMut<&'upper ()> for alloc::vec::Vec<T> {}

unsafe impl<'upper, T: 'upper + Clone> AliasableClone<&'upper ()> for alloc::vec::Vec<T> {}
