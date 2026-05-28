//! Aliasable version of `Box<T>` which doesn't invalidate pointers to its pointee when moved.
//!
//! That is, this type allows its pointee to be aliased.

#![expect(unsafe_code, reason = "assert variance and soundness of lifetime extension")]

use core::{cmp::Ordering, mem::transmute, pin::Pin, ptr::NonNull};
use core::{
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};
use alloc::boxed::Box;

use variance_family::{Unvarying, VaryingRef, VaryingRefMut};

use crate::{
    traits::{StableView, StableViewMut},
    view_kinds::{PointerViewKind, SetDefaultView, SetDefaultViewMut},
};


/// A non-unique version of `Box<T>` which can be freely moved without invalidating pointers
/// or references derived from it.
///
/// In current aliasing models, moving a `Box<T>` may introduce an exclusive retag which invalidates
/// pointers or references derived from the moved-from value of the reference.
///
/// # Aliasing Guarantees
/// Unsafe code can rely on the following two guarantees of weakened aliasing requirements.
/// In particular, a pointer or reference guaranteed to not be invalidated may continue to be used
/// (which requires `unsafe`ly dereferencing a raw pointer or lifetime-extending a reference).
///
/// ### `&T`
/// Any `&T` directly obtained from a value of `Self` via methods provided by this crate[^1], as
/// well as all pointers or references derived from such a `&T`, will not be invalidated by moving
/// that value of `Self`, by performing coercions (e.g., where `'long: 'short`, an
/// `AliasableBox<[&'long T; N]>` may be coerced to `AliasableBox<[&'short T]>` no differently than
/// a move), or by performing any operation on a shared reference (`&Self`) to that value.
///
/// In particular, calling any methods of `AliasableBox` on that value of `Self` which take the
/// value as an owned `Self` argument or an exclusively borrowed `&mut Self` argument may
/// invalidate such pointers and references (or allow safe code to later invalidate such pointers
/// and references). (Aliasing rules may also invalidate those pointers and references due to
/// interactions among themselves, as normal.)
///
/// Any `unsafe` operation on an `&AliasableBox` value should be careful to not violate this
/// guarantee.
/// (Safe code cannot violate this guarantee, as doing so requires writing through a raw pointer.)
///
/// ### `&mut T`
/// Any `&mut T` directly obtained from a value of `Self` via methods provided by this crate[^1], as
/// well as all pointers or references derived from such a `&mut T`, will not be invalidated by
/// moving that value of `Self`, by performing coercions (e.g., where `'long: 'short`, an
/// `AliasableBox<[&'long T; N]>` may be coerced to `AliasableBox<[&'short T]>` no differently than
/// a move), or by performing no-ops on it.
///
/// In particular, calling any methods of `AliasableBox` on that value of `Self` (whether owned
/// or referenced) may invalidate such pointers and references (or allow safe code to later
/// invalidate such pointers and references). (Aliasing rules may also invalidate those pointers and
/// references due to interactions among themselves, as normal.)
///
/// [^1]: This qualifier is intended to exclude pathological third-party implementations and
///     pathological interpretations of these guarantees. The following lists cannot and are not
///     intended to be exhaustive.
///
///     Ways to obtain a `&T` to which the first guarantee applies include
///     `AliasableBox`'s [`Deref`], [`AsRef`], and [`StableView::view`] implementations.
///     Ways to obtain a `&mut T` to which the second guarantee applies include
///     `AliasableBox`'s [`DerefMut`], [`AsMut`], and [`StableViewMut::view_mut`]
///     implementations.
///
///     [`AliasableBox::into_box`] and [`AliasableBox::into_pin_box`] are intentionally not
///     listed, as they consume a `Self` value, so vacuously that value cannot be later used to
///     invalidate any pointers or references; the value would already be gone.
///
/// # Layout
/// This type is a transparent wrapper around a `NonNull<T>` and may be used in FFI (depending on
/// what `T` is). Of course, many invariants are required of that `NonNull<T>` beyond simply being
/// non-null; do not recklessly transmute this type and write to its pointer.
///
/// If a more suitable type ever becomes available (such as a pointer type with alignment, non-null,
/// dereferenceability, and pointee validity requirements but the weak aliasing requirements of
/// raw pointers), a breaking change might be made to change the layout.
#[repr(transparent)]
pub struct AliasableBox<T: ?Sized> {
    /// # Safety invariant
    /// This pointer can generally be converted into a valid `&'c T` or `&'c mut T`.
    /// This comes at the expense of invalidating some pointers and references previously derived
    /// from `self.ptr` (or, allow such pointers to be invalidated by safe code using the `&'c T`
    /// or `&'c mut T`).
    ///
    /// `self.ptr` semantically owns its pointee (and is responsible for dropping it).
    ///
    /// Therefore, methods of this type converting `self.ptr` into a (possibly mutable) reference
    /// must uphold the aliasing guarantees of this type by ensuring the following:
    /// - Methods of `AliasableBox` which take a `Self` or `Pin<Self>` argument (or `Drop::drop`)
    ///   must convert `self.ptr` into a `Box<T>` exactly once, with `Box::from_raw` (though pinning
    ///   invariants must also be upheld).
    /// - Methods of `AliasableBox` which take an `&mut Self` argument are permitted
    ///   to convert `self.ptr` to a `&mut T` or `&T`.
    /// - Methods of `AliasableBox` which take an `&Self` argument are permitted to convert
    ///   `self.ptr` only to a `&T`.
    /// - No method may directly read or write `self.ptr` (instead, they should convert it to a
    ///   reference or `Box` first, if needed). (This isn't critical for soundness, but slightly
    ///   simplifies explanations of soundness below.)
    /// - No method may ever write anything but a valid value of type `T` into the pointee.
    ///   (We never do this, but this condition is included for completeness.) Variance is slightly
    ///   delicate; see below.
    /// - No method may expose a raw pointer to the pointee of `self.ptr` with a documented
    ///   guarantee that a value not valid for type `T` may be written to that location. (Such
    ///   a guarantee would be extremely incorrect. Again, we don't do that, this is included for
    ///   completeness.)
    /// - Only [`Self::from_box`] is permitted to directly construct `Self`. (This is included to
    ///   head off any possible invariants about never overwriting `self.ptr` with an invalid
    ///   pointer and whatnot.)
    ///
    /// In particular, methods taking `Self` or `Pin<Self>` (or `Drop::drop`), `&mut Self`, or
    /// `&Self` which directly manipulate `self.ptr` should cite the first, second, or third
    /// invariant, respectively, and no method should do anything listed in the last four bullet
    /// points, but there's no need to repeat those conditions everywhere.
    ///
    /// ## Sufficiency of those requirements
    ///
    /// ### Aliasing guarantees
    /// A `&T` obtained directly from `Self` through the intended means, and any pointers or
    /// references derived from that `&T`, derives from `self.ptr` (or a moved-to or moved-from
    /// version of `self.ptr`, if any retagging occurs when a raw pointer is moved). Moves of
    /// `Self` (and coercions) do not retag such pointers and references in a problematic way.
    /// No operation writes through `self.ptr` (or a pointer derived from it) when accessed through
    /// a `&Self` value; third-party `unsafe` code is explicitly warned against doing so in this
    /// type's documentation, and for the code here, functions taking `&AliasableBox` arguments
    /// (which are all methods of `AliasableBox`, in the case of this crate) are only permitted to
    /// convert `self.ptr` to a `&T`; that is, they can read through `self.ptr` (or references
    /// derived from it) but not write through it (or references derived from it).
    ///
    /// A `&mut T` obtained directly from `Self` through the intended means, and any pointers or
    /// references derived from that `&mut T`, derives from `self.ptr` (or a moved-to or moved-from
    /// version of `self.ptr`, if any retagging occurs when a raw pointer is moved). Moves of
    /// `Self` (and coercions) do not retag such pointers and references in a problematic way.
    /// All other operations are allowed to invalidate such references, so methods of `AliasableBox`
    /// taking `Self`, `&Self`, or `&mut Self` arguments can read or write through `self.ptr` (or
    /// references derived from it) without violating the aliasing guarantee.
    ///
    /// ### Stable view traits
    /// - [`StableView`] only prohibits the application of moves, coercions, and immutable
    ///   operations in any quantity and order to a `Self` value from invalidating pointers or
    ///   references derived from a call to [`StableView::view`] on that `Self` value
    ///   (which converts `self.ptr` into a `&T`).
    ///
    ///   The only methods of `AliasableBox` which invalidate such pointers and references are
    ///   those which convert `self.ptr` to a `&mut T`, and functions which take `&Self` arguments
    ///   are not permitted to do that.
    /// - [`StableViewMut`] only prohibits moves, coercions, and no-ops (in any quantity and order)
    ///   on a `Self` value from invalidating pointers or references derived from a call to
    ///   [`StableViewMut::view_mut`] on that `Self` value.
    ///
    ///   Moving a `NonNull` does not trigger any problematic exclusive retag, so that condition is
    ///   fulfilled, and methods of `AliasableBox` are freely permitted to invalidate other
    ///   pointers and references.
    /// - `AliasableBox` does not implement [`StableClone`].
    ///
    /// ### Converting `self.ptr` into a reference
    /// - It's always properly aligned; none of `AliasableBox`'s `&mut` methods mutate the
    ///   pointer itself, and it is constructed from a necessarily-properly-aligned `&mut T`
    ///   reference in [`Self::from_box`].
    /// - It's non-null (it's in a `NonNull`).
    /// - It's dereferenceable, since it is constructed from a necessarily-dereferenceable `&mut T`,
    ///   and we do not permit the user to deallocate or otherwise invalidate the pointee's
    ///   allocation. Moreover, the provenance should not be invalidated; it is constructed from a
    ///   `Box<U>` where `U` is a subtype of `T` (accounting for covariance), implying that
    ///   the pointee can only be accessed though pointers and references derived from that source
    ///   reference. Since that source box is discarded in [`Self::from_box`] (exposing only
    ///   `self.ptr` or one of its sibling pointers), all such pointers and references are derived
    ///   from `self.ptr` OR moved-to or moved-from versions of `self.ptr`, that is, sibling
    ///   pointers of `self.ptr` (noting that while moving a `Box` or `&mut` could result in
    ///   problematic retags, moving the raw pointer contained in `NonNull` is fine). (In
    ///   particular, under stacked borrows, all the siblings of `self.ptr` have compatible
    ///   `SharedReadWrite` permissions. Under tree borrows, all the siblings have identical
    ///   permissions and are considered to be at the same node of the tree.) Therefore, assuming
    ///   that the code of users of this type is UB-free, the provenance of `self.ptr` should not be
    ///   invalidated, so it would remain dereferenceable.
    /// - It points to a valid value of type `T`. We are covariant over `T`, so the pointee might
    ///   have originally been of type `U` where `U` is a subtype of `T`, meaning that the pointee
    ///   is also a valid value of type `T`. Note that we expose ways to write values of type `T`
    ///   to the pointee; however, only methods taking `&mut AliasableBox<T>` are permitted to do
    ///   so. The invariance of `&mut V` over `V` and the exclusivity of `&mut` and `AliasableBox`
    ///   imply that the sole source `AliasableBox<U>` must have been irrevocably coerced into
    ///   `AliasableBox<T>`, and then that `AliasableBox<T>` was mutably borrowed. In other words,
    ///   nothing is capable of soundly assuming that reading a `U` from the pointee is supposed
    ///   to always be possible; it suffices for the pointee to only be valid for type `T`, not `U`.
    /// - Aliasing rules are satisfied:
    ///   - When converting into a `&'c T` (which necessarily occurs via some method
    ///     of `AliasableBox` taking `&Self` or `&mut Self`), it is documented that all pointers and
    ///     references derived from a `&mut T` previously obtained via the `self` value are
    ///     invalidated (or may be invalidated by safe code); therefore, users are prohibited from
    ///     continuing to use such pointers and references.
    ///     (Note that `unsafe` would need to be involved in their continued usage to either
    ///     dereference a raw pointer or lifetime-extend a reference, so this does not place a
    ///     necessarily-unsound safety requirement on safe code.)
    ///
    ///     Therefore, out of all the previously-existing references and pointers with permission to
    ///     access the pointee of `self.ptr` while `self` exists -- all of which must be derived
    ///     from `self.ptr` or one of its sibling pointers, since the source `Box<T>` used to
    ///     construct `Self` was consumed -- only the ones derived from a `&T` are permitted to
    ///     be used. Additionally, the returned reference (and pointers derived from it) is allowed
    ///     to be used for some lifetime `'d` (possibly longer than `'c`) such that the value of
    ///     `self` is only moved, coerced, or immutably accessed during `'d` (else, the returned
    ///     reference would be potentially invalidated, as per our documentation), which does not
    ///     permit references (or pointers) with write permissions over the pointee of `self.ptr`
    ///     to be constructed from `self` (except via the returned reference).
    ///
    ///     Therefore, while the returned reference (and references or pointers derived from it)
    ///     is live, no pointers or references not derived from the returned reference (whether
    ///     previously existing or constructed while the returned reference is live) will mutate
    ///     (or assert exclusive permissions over) the pointee of the returned reference's pointee.
    ///
    ///   - When converting into a `&'c mut T` (which necessarily occurs via some method of
    ///     `AliasableBox` taking a `&mut Self` argument), it is documented that all pointers and
    ///     references derived from a `&mut T` *or* `&T` previously obtained via the `self` value
    ///     are invalidated (or may be invalidated by safe code); therefore, users are prohibited
    ///     from continuing to use such pointers and references. (Note that `unsafe` would need to
    ///     be involved in their continued usage to either dereference a raw pointer or
    ///     lifetime-extend a reference, so this does not place a necessarily-unsound safety
    ///     requirement on safe code.)
    ///
    ///     The source `Box<T>` used to construct `Self` was consumed, precluding the usage of
    ///     pointers or references *not* derived from `self.ptr` or one of its sibling pointers.
    ///     Since previously-existing such pointers and references are invalidated (other than
    ///     `self.ptr`, and possibly its sibling pointers but those should no longer be accessible,
    ///     so they don't matter), only `self.ptr` and the newly-constructed `&'c mut T` (and any
    ///     pointers or references derived from them) may be used to access the pointee of
    ///     `self.ptr`. The returned reference (and pointers derived from it) is allowed to be used
    ///     for some lifetime `'d` (possibly longer than `'c`) such that the value of `self` is only
    ///     moved or coerced or has no-ops performed on it during `'d` (else, the returned reference
    ///     would be potentially invalidated, as per our documentation), which does not permit
    ///     references (or pointers) that alias the pointee of `self.ptr` to be constructed from
    ///     `self` (except via the returned reference).
    ///
    ///     Therefore, while the returned reference (and references or pointers derived from it)
    ///     is live, no pointers or references not derived from the returned reference (whether
    ///     previously existing or constructed while the returned reference is live) will access
    ///     (or assert exclusive permissions over) the pointee of the returned reference's pointee.
    ///
    /// ### Converting `self.ptr` into a `Box<T>`
    ///
    /// - Since we only do this conversion *exactly once* in a method of `AliasableBox` taking
    ///   `Self` or `Pin<Self>` or in `Drop::drop`, using `Box::<T>::from_raw`, we know that this
    ///   conversion happens exactly once. Therefore, a double-drop does not occur from calling
    ///   `Box::<T>::from_raw` more than once (or independently dropping the `T`, which we never do;
    ///   no methods are permitted to consume `self.ptr` into a `T`, only into a `Box<T>`).
    /// - We construct an `AliasableBox` via `Box::into_raw`, which uses the global allocator, so
    ///   the pointee of `self.ptr` is allocated by the global allocator.
    /// - For non-ZSTs, the pointee is an allocation with the appropriate layout, since we got
    ///   the pointer from `Box::into_raw` and only performed covariant coercions since then;
    ///   `Box<T>` is also covariant over `T`, so clearly this is sound.
    /// - For ZSTs, the pointer must be non-null and sufficiently aligned. We get it from
    ///   `Box::into_raw`, so this constraint holds.
    /// - The pointee is a valid value of type `T`. See above about converting `self.ptr` into a
    ///   reference, which goes into the impact of covariance over `T` on this invariant.
    ///
    /// [`StableClone`]: crate::traits::StableClone
    ptr: NonNull<T>,
}

impl<T: ?Sized> AliasableBox<T> {
    /// Convert a `Box<T>` into an aliasable version.
    #[inline]
    #[must_use]
    pub fn from_box(boxed: Box<T>) -> Self {
        let ptr = Box::into_raw(boxed);
        // SAFETY: `Box::into_raw` guarantees that its return value is non-null.
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        // SAFETY INVARIANT: using the explicit constructor is only permitted in
        // `AliasableBox::from_box`, which is this function.
        Self { ptr }
    }

    /// Convert an aliasable version of `Box<T>` back into its source form.
    #[inline]
    #[must_use]
    pub fn into_box(self) -> Box<T> {
        // SAFETY: this is a method of `AliasableBox` with a `Self` argument which directly
        // manipulates `self.ptr`, so as per the first safety invariant of `self.ptr`, we can (and
        // must) consume `self.ptr` into a `Box<T>` with `Box::from_raw`.
        unsafe { Box::from_raw(self.ptr.as_ptr()) }
    }

    /// Convert an `Pin<Box<T>>` into an aliasable version.
    #[inline]
    #[must_use]
    pub fn from_pin_box(pin: Pin<Box<T>>) -> Pin<Self> {
        // SAFETY: we do not move out of the returned `Box<T>` in this function or in
        // `Self::from_box`, and the returned value is pinned (so by the pinning invariant, if
        // `T: !Unpin`, we never move out of this reference or otherwise invalidate the pointee
        // until the pointee is dropped).
        let boxed: Box<T> = unsafe { Pin::into_inner_unchecked(pin) };
        let boxed = Self::from_box(boxed);
        // SAFETY:
        // - The `Deref` and `DerefMut` implementations of `AliasableBox` do not move out of the
        //   pointee, and our `Drop` impl defers to `Box`'s drop impl to drop and deallocate the
        //   `T` (and since `Box<T>` supports `Box::pin`, we know that it will not allow the
        //   `T` allocation to be invalidated or repurposed until `T::drop` returns or panics).
        //   Therefore, those trait implementations are well-behaved.
        // - We do not have other problematic pin projections or something. We are not a
        //   `#[fundamental]` type with myriad soundness concerns around pinning. The pointee is
        //   properly pinned.
        unsafe { Pin::new_unchecked(boxed) }
    }

    /// Convert an aliasable version of `Pin<Box<T>>` back into its source form.
    #[inline]
    #[must_use]
    pub fn into_pin_box(pin: Pin<Self>) -> Pin<Box<T>> {
        // SAFETY: we treat `boxed` as pinned, namely, we do not move or overwrite (or otherwise
        // invalidate) its pointee in this function or in `Self::into_box`, and the returned
        // value is pinned (so by the pinning invariant, if `T: !Unpin`, we never move out of
        // the reference or otherwise invalidate the pointee until the pointee is dropped).
        let boxed = unsafe { Pin::into_inner_unchecked(pin) };
        let boxed: Box<T> = boxed.into_box();
        Box::into_pin(boxed)
    }
}

impl<T: ?Sized> Drop for AliasableBox<T> {
    fn drop(&mut self) {
        // SAFETY: this is the `Drop::drop` impl of `AliasableBox` which directly
        // manipulates `self.ptr`, so as per the first safety invariant of `self.ptr`, we can (and
        // must) consume `self.ptr` into a `Box<T>` with `Box::from_raw`.
        let _drop = unsafe { Box::from_raw(self.ptr.as_ptr()) };
    }
}

impl<T: ?Sized> Deref for AliasableBox<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: this is a method of `AliasableBox` with a `&Self` argument, so as per
        // the safety invariant of `self.ptr`, creating a `&'c T` from `self.ptr` is sound.
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for AliasableBox<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: this is a method of `AliasableBox` with a `&mut Self` argument, so as per
        // the safety invariant of `self.ptr`, creating a `&'c mut T` from `self.ptr` is sound.
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: ?Sized> AsRef<T> for AliasableBox<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        // Note that the aliasing guarantees of `AliasableBox` apply to the returned reference.
        self
    }
}

impl<T: ?Sized> AsMut<T> for AliasableBox<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        // Note that the aliasing guarantees of `AliasableBox` apply to the returned reference.
        self
    }
}

impl<T: ?Sized> From<Box<T>> for AliasableBox<T> {
    #[inline]
    fn from(boxed: Box<T>) -> Self {
        Self::from_box(boxed)
    }
}

impl<T: ?Sized> From<Pin<Box<T>>> for Pin<AliasableBox<T>> {
    #[inline]
    fn from(pin: Pin<Box<T>>) -> Self {
        AliasableBox::from_pin_box(pin)
    }
}

// SAFETY: By the aliasing guarantee of `AliasableBox` for `&T` references obtained from
// `Deref::deref` (among other methods), performing moves, coercions, or immutable operations
// in any quantity and order on the source `Self` value will not invalidate the returned `&T` view.
unsafe impl<'a, 'data, T> StableView<'a, 'data, AliasableBox<T>> for PointerViewKind
where
    T: ?Sized + 'data,
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    unsafe fn view<'stable>(data: &'a AliasableBox<T>) -> &'stable T
    where
        'data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a T = data;

        // SAFETY: See the "`transmute` in `view` Implementation" section of the `StableView` docs.
        unsafe {
            transmute::<
                &'a T,
                &'stable T,
            >(stable_eq_a)
        }
    }
}

impl<'data, T: ?Sized + 'data> SetDefaultView<'_, 'data> for AliasableBox<T> {
    type Default = PointerViewKind;
}

// SAFETY: By the aliasing guarantee of `AliasableBox` for `&mut T` references obtained from
// `DerefMut::deref_mut` (among other methods), performing moves or coercions (in any quantity
// and order) on the source `Self` value will not invalidate the returned `&mut T` view.
unsafe impl<'a, 'data, T> StableViewMut<'a, 'data, AliasableBox<T>> for PointerViewKind
where
    T: ?Sized + 'data,
{
    type ViewMut = VaryingRefMut<Unvarying<T>>;

    #[inline]
    unsafe fn view_mut<'stable>(data: &'a mut AliasableBox<T>) -> &'stable mut T
    where
        'data: 'stable,
        'stable: 'a,
    {
        let stable_eq_a: &'a mut T = data;

        // SAFETY: See the "`transmute` in `view_mut` Implementation" section of the
        // `StableViewMut` docs.
        unsafe {
            transmute::<
                &'a mut T,
                &'stable mut T,
            >(stable_eq_a)
        }
    }
}

impl<'data, T: ?Sized + 'data> SetDefaultViewMut<'_, 'data> for AliasableBox<T> {
    type DefaultMut = PointerViewKind;
}

// SAFETY: Since `AliasableBox<T>` acts like `Box<T>`,
// it can be `Send` if `Box<T>` is `Send`. We know that `Box<T>` is `Send` iff `T` is `Send`.
unsafe impl<T: ?Sized + Send> Send for AliasableBox<T> {}

// SAFETY: Since `AliasableBox<T>` acts like `Box<T>`,
// it can be `Sync` if `Box<T>` is `Sync`. We know that `Box<T>` is `Sync` iff `T` is `Sync`.
unsafe impl<T: ?Sized + Sync> Sync for AliasableBox<T> {}

impl<T: ?Sized + Debug> Debug for AliasableBox<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&**self, f)
    }
}

impl<A: ?Sized + PartialEq<B>, B: ?Sized> PartialEq<AliasableBox<B>> for AliasableBox<A> {
    fn eq(&self, other: &AliasableBox<B>) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

impl<A: ?Sized + PartialEq<B>, B: ?Sized> PartialEq<&B> for AliasableBox<A> {
    fn eq(&self, other: &&B) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

impl<A: ?Sized + PartialEq<B>, B: ?Sized> PartialEq<&mut B> for AliasableBox<A> {
    fn eq(&self, other: &&mut B) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

impl<T: ?Sized + Eq> Eq for AliasableBox<T> {}

impl<A: ?Sized + PartialOrd<B>, B: ?Sized> PartialOrd<AliasableBox<B>> for AliasableBox<A> {
    fn partial_cmp(&self, other: &AliasableBox<B>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<A: ?Sized + PartialOrd<B>, B: ?Sized> PartialOrd<&B> for AliasableBox<A> {
    fn partial_cmp(&self, other: &&B) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<A: ?Sized + PartialOrd<B>, B: ?Sized> PartialOrd<&mut B> for AliasableBox<A> {
    fn partial_cmp(&self, other: &&mut B) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<T: ?Sized + Ord> Ord for AliasableBox<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: ?Sized + Hash> Hash for AliasableBox<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state);
    }
}
