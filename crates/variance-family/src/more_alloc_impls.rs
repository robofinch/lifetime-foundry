//! With the `more-impls` feature, also:
//!
//! - `collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque}`,
//! - `rc::Weak`,
//! - `sync::Weak`.

#![expect(unsafe_code, reason = "assert variance and permission to impl traits for `alloc` types")]

use crate::generic_wrapper;


generic_wrapper! {
    impl<{
        // SAFETY: `BTreeMap<K, V>` is covariant over `K`.
        #[unsafe(covariant)] K (Is: Sized),
        // SAFETY: `BTreeMap<K, V>` is covariant over `V`.
        #[unsafe(covariant)] V (Is: Sized),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `alloc`.
    for #[unsafe(not_a_foreign_fundamental_type)] alloc::collections::BTreeMap<..>
}

generic_wrapper! {
    impl<{
        // SAFETY: `BTreeSet<T>` is covariant over `T`.
        #[unsafe(covariant)] T (Is: Sized),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `alloc`.
    for #[unsafe(not_a_foreign_fundamental_type)] alloc::collections::BTreeSet<..>
}

generic_wrapper! {
    impl<{
        // SAFETY: `BinaryHeap<T>` is covariant over `T`.
        #[unsafe(covariant)] T (Is: Sized),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `alloc`.
    for #[unsafe(not_a_foreign_fundamental_type)] alloc::collections::BinaryHeap<..>
}

generic_wrapper! {
    impl<{
        // SAFETY: `LinkedList<T>` is covariant over `T`.
        #[unsafe(covariant)] T (Is: Sized),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `alloc`.
    for #[unsafe(not_a_foreign_fundamental_type)] alloc::collections::LinkedList<..>
}

generic_wrapper! {
    impl<{
        // SAFETY: `VecDeque<T>` is covariant over `T`.
        #[unsafe(covariant)] T (Is: Sized),
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `alloc`.
    for #[unsafe(not_a_foreign_fundamental_type)] alloc::collections::VecDeque<..>
}

generic_wrapper! {
    impl<{
        // SAFETY: `rc::Weak<T>` is covariant over `T`.
        #[unsafe(covariant)] T,
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `alloc`.
    for #[unsafe(not_a_foreign_fundamental_type)] alloc::rc::Weak<..>
    where {T: ?Sized}
}

generic_wrapper! {
    impl<{
        // SAFETY: `sync::Weak<T>` is covariant over `T`.
        #[unsafe(covariant)] T,
    }> ([Co] + [Contra])variantFamily<'_, _>
    // SAFETY: `variance-family` is allowed to implement traits for this type in `alloc`.
    for #[unsafe(not_a_foreign_fundamental_type)] alloc::sync::Weak<..>
    where {T: ?Sized}
}
