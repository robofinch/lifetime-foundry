#![expect(unsafe_code, reason = "assert variance and soundness of lifetime extension")]

use core::{cmp::Ordering, marker::PhantomData, pin::Pin, ptr::NonNull};
use core::{
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

use variance_family::{Unvarying, VaryingRef, VaryingRefMut};

use crate::traits::{
    AliasableView, AliasableViewMut, IntoAliasable, IntoAliasableMut, View, ViewMut,
};


/// A non-unique version of `&'a mut T` which can be freely moved without invalidating pointers
/// or references derived from it.
///
/// In current aliasing models, moving a `&'a mut T` introduces an exclusive retag which invalidates
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
/// that value of `Self`, by dropping it, by performing coercions (e.g., where `'long: 'short`, an
/// `AliasableRefMut<'long, T>` may be coerced to `AliasableRefMut<'short, T>` no differently than
/// a move), or by performing any operation on a shared reference (`&Self`) to that value.
///
/// In particular, calling any methods of `AliasableRefMut` on that value of `Self` which take the
/// value as an owned `Self` argument or an exclusively borrowed `&mut Self` argument may
/// invalidate such pointers and references (or allow safe code to later invalidate such pointers
/// and references). Moreover, once the `'a` lifetime expires, pointers and references to the `T`
/// pointee could be invalidated through other means. (Aliasing rules may also invalidate those
/// pointers and references due to interactions among themselves, as normal.)
///
/// Any `unsafe` operation on an `&AliasableRefMut` value should be careful to not violate this
/// guarantee.
/// (Safe code cannot violate this guarantee, as doing so requires writing through a raw pointer.)
///
/// ### `&mut T`
/// Any `&mut T` directly obtained from a value of `Self` via methods provided by this crate[^1], as
/// well as all pointers or references derived from such a `&mut T`, will not be invalidated by
/// moving that value of `Self`, by dropping it, or by performing coercions (e.g., where
/// `'long: 'short`, an `AliasableRefMut<'long, T>` may be coerced to `AliasableRefMut<'short, T>`
/// no differently than a move).
///
/// In particular, calling any methods of `AliasableRefMut` on that value of `Self` (whether owned
/// or referenced) may invalidate such pointers and references (or allow safe code to later
/// invalidate such pointers and references). Moreover, once the `'a` lifetime expires, pointers
/// and references to the `T` pointee could be invalidated through other means. (Aliasing rules may
/// also invalidate those pointers and references due to interactions among themselves, as normal.)
///
/// [^1]: This qualifier is intended to exclude pathological third-party implementations and
///   pathological interpretations of these guarantees. The following lists cannot and are not
///   intended to be exhaustive.
///
///   Ways to obtain a `&T` to which the first guarantee applies include
///   `AliasableRefMut`'s [`Deref`], [`AsRef`], and [`AliasableView::view`] implementations.
///   Ways to obtain a `&mut T` to which the second guarantee applies include `AliasableRefMut`'s
///   [`DerefMut`], [`AsMut`], and [`AliasableViewMut::view_mut`] implementations.
///
///   [`AliasableRefMut::into_mut`] and [`AliasableRefMut::into_pin_mut`] are intentionally not
///   listed, as they consume a `Self` value, so vacuously that value cannot be later used to
///   invalidate any pointers or references; the value would already be gone.
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
pub struct AliasableRefMut<'a, T: ?Sized> {
    /// # Safety invariant
    /// This pointer can always be converted (possibly unsoundly, due to this type's aliasing
    /// guarantees) into a valid `&'c T` or `&'c mut T` where `'a: 'c`. This would come at the
    /// expense of invalidating some pointers and references previously derived from `self.ptr`
    /// (or, allow such pointers to be invalidated by safe code using the `&'c T` or `&'c mut T`).
    ///
    /// Therefore, methods of this type converting `self.ptr` into a (possibly mutable) reference
    /// must uphold the aliasing guarantees of this type by ensuring the following:
    /// - Methods of `AliasableRefMut` which take an `&mut Self` or `Self` argument are permitted
    ///   to convert `self.ptr` to a `&mut T` or `&T`.
    /// - Methods of `AliasableRefMut` which take an `&Self` argument are permitted to convert
    ///   `self.ptr` only to a `&T`.
    /// - The sole method taking `Pin<Self>` is sort of a special case. But neither of the two
    ///   aliasing guarantees of this type apply to `into_pin_mut`, so that doesn't matter.
    /// - No method may directly read or write to `self.ptr` (instead, they should convert it to a
    ///   reference first, if needed). (This isn't critical for soundness, but slightly simplifies
    ///   explanations of soundness below.)
    /// - No method may ever write anything but a valid value of type `T` into the pointee.
    ///   (Trivially, we never do this, but this condition is included for completeness.)
    /// - No method may expose a raw pointer to the pointee of `self.ptr` with a documented
    ///   guarantee that a value not valid for type `T` may be written to that location. (Such
    ///   a guarantee would be extremely incorrect. Again, we trivially don't do that, this is
    ///   included for completeness.)
    /// - Only [`Self::from_mut`] is permitted to directly construct `Self`. (This is included to
    ///   head off any possible invariants about never overwriting `self.ptr` with an invalid
    ///   pointer and whatnot.)
    ///
    /// In particular, methods taking `&mut Self` or `Self` which directly manipulate `self.ptr`
    /// should check the first invariant; methods taking `&Self` which directly manipulate
    /// `self.ptr` should check the second invariant; `into_pin_mut` does its own thing; and no
    /// method should do anything listed in the last four bullet points, but there's no need to
    /// repeat those conditions everywhere.
    ///
    /// ## Sufficiency of those requirements
    ///
    /// ### Aliasing guarantees
    /// A `&T` obtained directly from `Self` through the intended means, and any pointers or
    /// references derived from that `&T`, derives from `self.ptr` (or a moved-to or moved-from
    /// version of `self.ptr`, if any retagging occurs when a raw pointer is moved). Moves of
    /// `Self` (and coercions) do not retag such pointers and references in a problematic way.
    /// Dropping an `AliasableRefMut` may assert exclusive access over the `NonNull` field (but
    /// that's not transitive, and does not invalidate other pointers to its pointee) and is
    /// otherwise a no-op. No operation writes through `self.ptr` (or a pointer derived from it)
    /// when accessed through a `&Self` value; third-party `unsafe` code is explicitly warned
    /// against doing so in this type's documentation, and for the code here, functions taking
    /// `&AliasableRefMut` arguments (which are all methods of `AliasableRefMut`, in the case of
    /// this crate) are only permitted to convert `self.ptr` to a `&T`; that is, they can read
    /// through `self.ptr` (or references derived from it) but not write through it (or references
    /// derived from it).
    ///
    /// A `&mut T` obtained directly from `Self` through the intended means, and any pointers or
    /// references derived from that `&mut T`, derives from `self.ptr` (or a moved-to or moved-from
    /// version of `self.ptr`, if any retagging occurs when a raw pointer is moved). Moves of
    /// `Self` (and coercions) do not retag such pointers and references in a problematic way.
    /// Dropping an `AliasableRefMut` may assert exclusive access over the `NonNull` field (but
    /// that's not transitive, and does not invalidate other pointers to its pointee) and is
    /// otherwise a no-op. All other operations are allowed to invalidate such references, so
    /// methods of `AliasableRefMut` taking `Self`, `&Self`, or `&mut Self` arguments can read or
    /// write through `self.ptr` (or references derived from it) without violating the aliasing
    /// guarantee.
    ///
    /// ### Aliasable view traits
    /// - [`AliasableView`] only prohibits the application of moves, coercions, and immutable
    ///   operations in any quantity and order to a `Self` value from invalidating pointers or
    ///   references derived from a call to [`AliasableView::view`] on that `Self` value
    ///   (which converts `self.ptr` into a `&T`), and permits the view to be invalidated once
    ///   lifetime parameters of the implementing type expire.
    ///
    ///   The only methods of `AliasableRefMut` which invalidate such pointers and references are
    ///   those which convert `self.ptr` to a `&mut T`, and functions which take `&Self` arguments
    ///   are not permitted to do that. Once lifetime `'a` expires, references to the pointee
    ///   of `self.ptr` might be invalidated through other means, which is fine.
    /// - [`AliasableViewMut`] only prohibits moves and coercions (in any quantity and order) on
    ///   a `Self` value from invalidating pointers or references derived from a call to
    ///   [`AliasableViewMut::view_mut`] on that `Self` value, and permits the view to be
    ///   invalidated once lifetime parameters of the implementing type expire.
    ///
    ///   Moving a `NonNull` does not trigger any problematic exclusive retag, so that condition is
    ///   fulfilled, and methods of `AliasableRefMut` are freely permitted to invalidate other
    ///   pointers and references. Once lifetime `'a` expires, references to the pointee
    ///   of `self.ptr` might be invalidated through other means, which is fine.
    /// - `AliasableRefMut` does not implement [`AliasableClone`].
    ///
    /// ### Converting `self.ptr` into a reference
    /// - It's always properly aligned; none of `AliasableRefMut`'s `&mut` methods mutate the
    ///   pointer itself, and it is constructed from a necessarily-properly-aligned `&mut T`
    ///   reference in [`Self::from_mut`].
    /// - It's non-null (it's in a `NonNull`).
    /// - It's dereferenceable, since it is constructed from a necessarily-dereferenceable `&mut T`,
    ///   and we do not permit the user to deallocate or otherwise invalidate the pointee's
    ///   allocation. Moreover, the provenance should not be invalidated; it is constructed from a
    ///   `&'b mut T` with `'b: 'a` (accounting for covariance), implying that for lifetime `'b`,
    ///   the pointee can only be accessed though pointers and references derived from that source
    ///   reference. Since that source reference is discarded in [`Self::from_mut`] (exposing only
    ///   `self.ptr` or one of its sibling pointers), during at least lifetime `'a`, all such
    ///   pointers and references are derived from `self.ptr` OR moved-to or moved-from versions of
    ///   `self.ptr`, that is, sibling pointers of `self.ptr` (noting that while moving a `Box` or
    ///   `&mut` could result in problematic retags, moving the raw pointer contained in `NonNull`
    ///   is fine). (In particular, under stacked borrows, all the siblings of `self.ptr` should
    ///   have `SharedReadWrite` permissions. I'm not sure if the raw pointers are retagged when
    ///   moved, or if they all use the same tag, but that shouldn't matter.) Therefore, assuming
    ///   that the code of users of this type is UB-free, the provenance of `self.ptr` should not
    ///   be invalidated, so it would remain dereferenceable.
    /// - It points to a valid value of type `T`, since we are invariant over `T` and we only
    ///   expose ways to write (or read) values of type `T` to the pointee (note that invariance
    ///   prevents a supertype value from being written to the pointee of a more-restrictive subtype
    ///   pointer), and the pointee is initially a valid value of type `T` when `self.ptr` is
    ///   constructed from a reference.
    /// - Aliasing rules are satisfied:
    ///   - When converting into a `&'c T` where `'a: 'c` (which necessarily occurs via some method
    ///     of `AliasableRefMut`), it is documented that all pointers and references derived from a
    ///     `&mut T` previously obtained via the `self` value are invalidated (or may be invalidated
    ///     by safe code); therefore, users are prohibited from continuing to use such pointers
    ///     and references.
    ///     (Note that `unsafe` would need to be involved in their continued usage to either
    ///     dereference a raw pointer or lifetime-extend a reference, so this does not place a
    ///     necessarily-unsound safety requirement on safe code.)
    ///
    ///     Therefore, out of all the previously-existing references and pointers with permission to
    ///     access the pointee of `self.ptr` during `'a` -- all of which must be derived from
    ///     `self.ptr` or one of its sibling pointers, since the source `&'b mut T` reference
    ///     (with `'b: 'a`) used to construct `Self` precludes the usage of other pointers or
    ///     references during lifetime `'a` -- only the ones derived from a `&T` are permitted to
    ///     be used. Those pointers only have shared permissions over the pointee of `self.ptr` for
    ///     lifetime at most `'a`. Additionally, the returned reference (and pointers derived from
    ///     it) is allowed to be used for some lifetime `'d` (possibly longer than `'c`) such that
    ///     `'a: 'd` and the value of `self` is only moved, dropped, coerced, or immutably accessed
    ///     during `'d` (else, the returned reference would be potentially invalidated, as per our
    ///     documentation), which does not permit references (or pointers) with write permissions
    ///     over the pointee of `self.ptr` to be constructed from `self` (except via the returned
    ///     reference).
    ///
    ///     Therefore, while the returned reference (and references or pointers derived from it)
    ///     is live, no pointers or references not derived from the returned reference (whether
    ///     previously existing or constructed while the returned reference is live) will mutate
    ///     (or assert exclusive permissions over) the pointee of the returned reference's pointee.
    ///
    ///   - When converting into a `&'c mut T` where `'a: 'c` (which necessarily occurs via some
    ///     method of `AliasableRefMut` *other* than the ones taking `&Self` arguments), it is
    ///     documented that all pointers and references derived from a `&mut T` *or* `&T` previously
    ///     obtained via the `self` value are invalidated (or may be invalidated by safe code);
    ///     therefore, users are prohibited from continuing to use such pointers and references.
    ///     (Note that `unsafe` would need to be involved in their continued usage to either
    ///     dereference a raw pointer or lifetime-extend a reference, so this does not place a
    ///     necessarily-unsound safety requirement on safe code.)
    ///
    ///     During lifetime `'a`, the source `&'b mut T` reference (with `'b: 'a`) used to
    ///     construct `Self` precludes the usage of pointers or references *not* derived from
    ///     `self.ptr` or one of its sibling pointers. Since previously-existing such pointers
    ///     and references are invalidated (other than `self.ptr`, and possibly its sibling pointers
    ///     but those should no longer be accessible, so they don't matter), only `self.ptr` and the
    ///     newly-constructed `&'c mut T` (and any pointers or references derived from them) may
    ///     be used to access the pointee of `self.ptr`. The returned reference (and pointers
    ///     derived from it) is allowed to be used for some lifetime `'d` (possibly longer than
    ///     `'c`) such that `'a: 'd` and the value of `self` is only moved, dropped, or coerced
    ///     during `'d` (else, the returned reference would be potentially invalidated, as per our
    ///     documentation), which does not permit references (or pointers) that alias the pointee
    ///     of `self.ptr` to be constructed from `self` (except via the returned reference).
    ///
    ///     Therefore, while the returned reference (and references or pointers derived from it)
    ///     is live, no pointers or references not derived from the returned reference (whether
    ///     previously existing or constructed while the returned reference is live) will access
    ///     (or assert exclusive permissions over) the pointee of the returned reference's pointee.
    ///
    /// [`AliasableClone`]: crate::traits::AliasableClone
    ptr:       NonNull<T>,
    _variance: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> AliasableRefMut<'a, T> {
    /// Convert an `&'a mut T` into an aliasable version.
    #[inline]
    #[must_use]
    pub const fn from_mut(ptr: &'a mut T) -> Self {
        // SAFETY: references are non-null.
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        // SAFETY INVARIANT: using the explicit constructor is only permitted in
        // `AliasableRefMut::from_mut`, which is this function.
        Self {
            ptr,
            _variance: PhantomData,
        }
    }

    /// Convert an aliasable version of `&'a mut T` back into its source form.
    #[inline]
    #[must_use]
    pub const fn into_mut(mut self) -> &'a mut T {
        // SAFETY: this is a method of `AliasableRefMut` with a `Self` argument, so as per
        // the safety invariant of `self.ptr`, creating a `&'a mut T` from `self.ptr` is sound.
        unsafe { self.ptr.as_mut() }
    }

    /// Convert an `Pin<&'a mut T>` into an aliasable version.
    #[inline]
    #[must_use]
    pub const fn from_pin_mut(ptr: Pin<&'a mut T>) -> Pin<Self> {
        // SAFETY: we do not move out of the returned `&'a mut T` in this function or in
        // `Self::from_mut`, and the returned value is pinned (so by the pinning
        // invariant, if `T: !Unpin`, we never move out of this reference or otherwise invalidate
        // the pointee until the pointee is dropped).
        let ptr: &'a mut T = unsafe { ptr.get_unchecked_mut() };
        let ptr = Self::from_mut(ptr);
        // SAFETY:
        // - The `Deref` and `DerefMut` implementations of `AliasableRefMut` do not move out of the
        //   pointee, and it does not implement `Drop`. Therefore, those trait implementations
        //   are well-behaved.
        // - Since the source `&'a mut T` reference is pinned, we know that the produced pinned
        //   value (if `!Unpin`) remains pinned until it is dropped (even though, ordinarily,
        //   we would not be able to guarantee what happens after lifetime `'a`).
        // - We do not have other problematic pin projections or something. We are not a
        //   `#[fundamental]` type with myriad soundness concerns around pinning. The pointee is
        //   properly pinned.
        unsafe { Pin::new_unchecked(ptr) }
    }

    /// Convert an aliasable version of `Pin<&'a mut T>` back into its source form.
    #[inline]
    #[must_use]
    pub const fn into_pin_mut(pin: Pin<Self>) -> Pin<&'a mut T> {
        // SAFETY: we treat `ptr` as pinned, namely, we do not move or overwrite (or otherwise
        // invalidate) its pointee in this function or in `Self::into_mut`, and the returned
        // value is pinned (so by the pinning invariant, if `T: !Unpin`, we never move out of
        // the reference or otherwise invalidate the pointee until the pointee is dropped).
        let ptr = unsafe { Pin::into_inner_unchecked(pin) };
        let ptr: &'a mut T = ptr.into_mut();
        // SAFETY:
        // - The `Deref` and `DerefMut` implementations of `&'a mut T` do not move out of the
        //   pointee, and it does not implement `Drop`. Therefore, those trait implementations
        //   are well-behaved.
        // - Since the source `Aliasable<'a, T>` reference is pinned, we know that the produced
        //   pinned value (if `!Unpin`) remains pinned until it is dropped (even though, ordinarily,
        //   we would not be able to guarantee what happens after lifetime `'a`).
        // - Mutable references do not have other problematic pin projections or something, and if
        //   there were, it'd be the responsibility of `core` to fix that, not us. The pointee is
        //   properly pinned.
        unsafe { Pin::new_unchecked(ptr) }
    }
}

impl<T: ?Sized> Deref for AliasableRefMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: this is a method of `AliasableRefMut` with a `&Self` argument, so as per
        // the safety invariant of `self.ptr`, creating a `&'c T` from `self.ptr`
        // where `'a: 'c` is sound.
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for AliasableRefMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: this is a method of `AliasableRefMut` with a `&mut Self` argument, so as per
        // the safety invariant of `self.ptr`, creating a `&'c mut T` from `self.ptr`
        // where `'a: 'c` is sound.
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: ?Sized> AsRef<T> for AliasableRefMut<'_, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        // Note that the aliasing guarantees of `AliasableRefMut` apply to the returned reference.
        self
    }
}

impl<T: ?Sized> AsMut<T> for AliasableRefMut<'_, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        // Note that the aliasing guarantees of `AliasableRefMut` apply to the returned reference.
        self
    }
}

impl<'a, T: ?Sized> From<&'a mut T> for AliasableRefMut<'a, T> {
    #[inline]
    fn from(ptr: &'a mut T) -> Self {
        Self::from_mut(ptr)
    }
}

// SAFETY: By the aliasing guarantee of `AliasableRefMut` for `&T` references obtained from
// `Deref::deref` (among other methods), performing moves, coercions, or immutable operations
// in any quantity and order on the source `Self` value will not invalidate the returned `&T` view.
// (In fact, `AliasableRefMut` guarantees that dropping it will not invalidate views, either,
// which is stronger than the requirement imposed by `AliasableView`.)
unsafe impl<'upper, T> AliasableView<&'upper ()> for AliasableRefMut<'_, T>
where
    T: ?Sized + 'upper,
{
    type View = VaryingRef<Unvarying<T>>;

    #[inline]
    fn view(&self) -> View<'_, Self, &'upper ()> {
        self
    }
}

impl<'a, 'upper, T> IntoAliasable<&'upper ()> for &'a mut T
where
    T: ?Sized + 'upper,
{
    type IntoAliasable = AliasableRefMut<'a, T>;

    #[inline]
    fn into_aliasable(self) -> Self::IntoAliasable {
        AliasableRefMut::from_mut(self)
    }
}

// SAFETY: By the aliasing guarantee of `AliasableRefMut` for `&mut T` references obtained from
// `DerefMut::deref_mut` (among other methods), performing moves or coercions (in any quantity
// and order) on the source `Self` value will not invalidate the returned `&mut T` view.
// (In fact, `AliasableRefMut` guarantees that dropping it will not invalidate views, either, which
// is stronger than the requirement imposed by `AliasableViewMut`.)
unsafe impl<'upper, T> AliasableViewMut<&'upper ()> for AliasableRefMut<'_, T>
where
    T: ?Sized + 'upper,
{
    type ViewMut = VaryingRefMut<Unvarying<T>>;

    #[inline]
    fn view_mut(&mut self) -> ViewMut<'_, Self, &'upper ()> {
        self
    }
}

impl<'upper, T: ?Sized + 'upper> IntoAliasableMut<&'upper ()> for &mut T {}

// SAFETY: Since `AliasableRefMut<'_, T>` acts like `&mut T`,
// it can be `Send` if `&mut T` is `Send`. We know that `&mut T` is `Send` iff `T` is `Send`.
unsafe impl<T: ?Sized + Send> Send for AliasableRefMut<'_, T> {}

// SAFETY: Since `AliasableRefMut<'_, T>` acts like `&mut T`,
// it can be `Sync` if `&mut T` is `Sync`. We know that `&mut T` is `Sync` iff `T` is `Sync`.
unsafe impl<T: ?Sized + Sync> Sync for AliasableRefMut<'_, T> {}

impl<T: ?Sized + Debug> Debug for AliasableRefMut<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&**self, f)
    }
}

impl<A: ?Sized + PartialEq<B>, B: ?Sized> PartialEq<AliasableRefMut<'_, B>>
for AliasableRefMut<'_, A>
{
    fn eq(&self, other: &AliasableRefMut<'_, B>) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

impl<A: ?Sized + PartialEq<B>, B: ?Sized> PartialEq<&B> for AliasableRefMut<'_, A> {
    fn eq(&self, other: &&B) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

impl<A: ?Sized + PartialEq<B>, B: ?Sized> PartialEq<&mut B> for AliasableRefMut<'_, A> {
    fn eq(&self, other: &&mut B) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

impl<T: ?Sized + Eq> Eq for AliasableRefMut<'_, T> {}

impl<A: ?Sized + PartialOrd<B>, B: ?Sized> PartialOrd<AliasableRefMut<'_, B>>
for AliasableRefMut<'_, A>
{
    fn partial_cmp(&self, other: &AliasableRefMut<'_, B>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<A: ?Sized + PartialOrd<B>, B: ?Sized> PartialOrd<&B> for AliasableRefMut<'_, A> {
    fn partial_cmp(&self, other: &&B) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<A: ?Sized + PartialOrd<B>, B: ?Sized> PartialOrd<&mut B> for AliasableRefMut<'_, A> {
    fn partial_cmp(&self, other: &&mut B) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<T: ?Sized + Ord> Ord for AliasableRefMut<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: ?Sized + Hash> Hash for AliasableRefMut<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state);
    }
}
