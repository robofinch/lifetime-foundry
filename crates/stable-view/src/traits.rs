//! The core interface of this crate.

#![expect(unsafe_code, reason = "Allow `unsafe` code to rely on implementations being correct")]

use variance_family::{CovariantFamily, Varying};


/// The [`StableView::View`] associated with some `Data` and view kind (such as
/// [`ReferenceViewKind`] or [`DefaultViewKind`]).
///
/// [`ReferenceViewKind`]: crate::view_kinds::ReferenceViewKind
/// [`DefaultViewKind`]: crate::view_kinds::DefaultViewKind
pub type CustomView<'a, 'stable, 'data, Data, V>
    = Varying<'stable, 'a, &'data (), <V as StableView<'a, 'data, Data>>::View>;

/// The [`StableViewMut::ViewMut`] associated with some `Data` and view kind (such as
/// [`ReferenceViewKind`] or [`DefaultViewKind`]).
///
/// [`ReferenceViewKind`]: crate::view_kinds::ReferenceViewKind
/// [`DefaultViewKind`]: crate::view_kinds::DefaultViewKind
pub type CustomViewMut<'a, 'stable, 'data, Data, V>
    = Varying<'stable, 'a, &'data (), <V as StableViewMut<'a, 'data, Data>>::ViewMut>;

/// Get temporary "views" whose `'stable` data is suitable for self-references to the views' source
/// `Data` values in self-referential structs.
///
/// The primary interface for using this trait is [`StableViewer`]; you should not need to
/// directly use this trait's method. Implementations of this trait are also provided by various
/// "view kinds", such as [`ReferenceViewKind`] and [`DefaultViewKind`], so you should not need
/// to implement this trait, either.
///
/// If you need a `'stable` reference to something not covered by existing implementations,
/// consider wrapping it in an [`AliasableBox`].
///
/// See the crate-level documentation for more.
///
#[cfg_attr(feature = "alloc", doc = "[`AliasableBox`]: crate::aliasable::AliasableBox")]
#[cfg_attr(not(feature = "alloc"), doc = "[`AliasableBox`]: https://docs.rs/stable-view/0/stable_view/struct.AliasableBox.html")]
/// [`StableViewer`]: crate::viewer::StableViewer
/// [`ReferenceViewKind`]: crate::view_kinds::ReferenceViewKind
/// [`DefaultViewKind`]: crate::view_kinds::DefaultViewKind
pub trait StableView<'a, 'data, Data: ?Sized, __ImplyBound = &'a &'data ()> {
    /// A temporary view of `Data` whose `'stable` lifetime is suitable for self-references to the
    /// source `Data` value in self-referential structs.
    type View: CovariantFamily<'a, &'data (), Is: Sized>;

    /// Get a temporary "view" whose `'stable` data is suitable for self-references to the given
    /// `Data` value in self-referential structs.
    ///
    /// The primary interface for using this trait is [`StableViewer`]; you should not need to
    /// directly use this method. Implementations of this trait are also provided by various
    /// "view kinds", such as [`ReferenceViewKind`] and [`DefaultViewKind`], so you should not need
    /// to implement this trait, either.
    ///
    /// If you need a `'stable` reference to something not covered by existing implementations,
    /// consider wrapping it in an [`AliasableBox`].
    ///
    /// See the crate-level documentation for more.
    ///
    /// The rest of this method's documentation is targeted at experience Rust programmers with a
    /// solid understanding of `unsafe`. Read [`concepts_and_safety`] before proceeding.
    ///
    /// # Safety
    ///
    /// Where the implementor's type is `Self` and the source data type is `Data`, while `'data`
    /// has not yet ended, any stable data obtained via applying `Self`'s implementation of
    /// `StableView<'a, 'data, Data>::view::<'stable>` to a source `Data` value can be used at a
    /// given moment so long as, starting from when the view was created up to when it is used,
    /// only the following three kinds of operations (in any quantity and ordering) are applied to
    /// the source `Data` value:
    ///
    /// - moves, including any accompanying retag and other effects in the aliasing model,
    /// - "permitted coercions", which are any type of [coercions] available in or before stable
    ///   Rust 1.85, except `Deref` and `DerefMut` coercions, and
    /// - any (sound) operations which use data derived from the source `Data` value only through
    ///   shared/immutable references to the relevant parts of `Data`.
    ///
    /// ## Step-by-step Breakdown
    ///
    /// The clause about `'data` ending ensures that, say, `&'b T` can implement this trait with a
    /// `&'stable T` view where `'b: 'data`, even though the `&'stable T` may be invalidated after
    /// lifetime `'b` ends (even if only the three permitted kinds of operations are performed on
    /// the source `Data = &'b T` value).
    ///
    /// The language about only applying the three kinds of operations essentially means that those
    /// operations *do not invalidate* the `'stable` data obtained via the returned view, and so
    /// long as the `'stable` data has not been invalidated, it can (of course) still be used. Note
    /// that this trait does *not* imply that the `'stable` data is immediately invalidated when a
    /// different operation, like dropping the source `Data` value, is performed. For example,
    /// [`StableClone`] extends the list of operations which do not invalidate the `'stable` data.
    ///
    /// The "in any quantity and ordering" clause ensures that you can chain these operations
    /// together, even if the type of the source `Data` value changes. For example, you could coerce
    /// the `Data` value to some other type `D` (using a non-`DerefMut` coercion), perform an
    /// immutable operation on the coerced value, move it, wrap it into an a `&mut D`
    /// **without actually using that mutable reference**, and call `Box::leak` on the `D` value.
    /// (Note that creating a `&mut D` would assert `noalias` permissions over the inline data in
    /// `D`, but would not affect any `'stable` references to its contents. Additionally, *not*
    /// running the destructor of the source `Data` value is a no-op, and thus permitted under the
    /// first case.)
    ///
    /// The first operation is mainly to account for moving a `noalias` pointer like `&mut Data`.
    /// Moving a `&Data` reference is covered by the third operation, and moving other values is
    /// covered by the second operation (since a move generally only affects inline data). However,
    /// in the case of `noalias` pointers, materializing or moving a `&mut Data` might also grant
    /// permission to the compiler to read or write the pointee, depending on what Rust's aliasing
    /// model ends up being. In any case, a `&mut Data` generally cannot provide a `&'stable Data`,
    /// due to the guarantee of the first operation not being met.
    ///
    /// The second operation includes only operations which only read and write inline data of
    /// the `Data` value. Note that "only reading and writing inline data" could include transmuting
    /// to a different type with problematic operations that fall under the first or third kind,
    /// which is why this kind is restricted to a better-defined list. The "in stable Rust 1.85"
    /// qualifier is included to guard against any future user-defined coercions that could be
    /// problematic. Note that Rust 1.85 was chosen as the first version of the 2024 edition, not as
    /// the first version with some crucial coercion.
    ///
    /// The third operation could be called an "immutable operation" on the source `Data` value, if
    /// not for the possibility of internal mutability within `Data`, which could escalate a `&`
    /// reference to part of `Data` to a `&mut` reference to another part of `Data`.) This operation
    /// includes `Deref` coercions, though not `DerefMut` coercions.
    ///
    /// ## Implications
    ///
    /// We trivially know that only those three kinds of operations are performed on the source
    /// `Data` value at least while lifetime `'a` has not ended and the view has not been discarded,
    /// during which the source `Data` value is under a `&` borrow. Therefore, calling this method
    /// with `'stable = 'a` is necessarily sound.
    ///
    /// Conversely, calling this method with an overly-long `'stable` is permissible, even if the
    /// source `Data` value is invalidated before `'stable` ends. The `'stable` lifetime could also
    /// be transmuted to be longer sometime after calling this method. However, you would need to be
    /// careful to discard the `'stable` data before invalidating the source `Data` value, and the
    /// borrow checker wouldn't help you.
    ///
    /// # Dangers of Use
    ///
    /// ## Dangers of lifetime-transmuting a view
    ///
    /// Functions that take owned `Data` arguments or exclusively-borrowed `&mut Data` arguments
    /// (or which can transitively access an owned `Data` or `&mut Data`), including [`Drop::drop`],
    /// [`mem::drop`], and [`StableViewMut::view_mut`], are (in general) allowed to invalidate
    /// previously-returned views of those `Data` values (or to enable safe code to later invalidate
    /// previously-returned views). Some functions, such as `Box::new` (*when it does not unwind
    /// after OOM*) and [`mem::forget`], may be known to only perform permitted operations (possibly
    /// only under certain conditions), but be cautious.
    ///
    /// (Note that [`mem::forget`] does invalidate the location of a `Data` value, but a sound
    /// implementation of this method cannot hand out views which reference data stored inline in
    /// the source `Data`. [`mem::forget`] could perhaps be seen as semantically moving the `Data`
    /// value to some location that can never be accessed again.)
    ///
    /// As views may have nontrivial destructors, dropping an unsafely lifetime-extended view may
    /// count as a usage of that view; if a view is not known to have no drop glue, be careful not
    /// to perform any operation that could invalidate a view before dropping it. In particular,
    /// drop (or leak) views before dropping the `Data` source of those views.
    ///
    /// For example, when working with panicky functions which only invalidate the `Data` source on
    /// error (perhaps by dropping the `Data` value during unwinding), such as `Box::new(data)`, one
    /// sound approach is to wrap views in `ManuallyDrop` before calling the panicky function and
    /// only unwrap the views after the function's successful return; this ensures that views are
    /// not improperly accessed in their destructors during unwinding. A leak is far preferable to
    /// UB. (Using `Box::new_uninit()` to avoid unexpectedly dropping `data` is also possible.)
    /// See [this `yoke` issue](https://github.com/unicode-org/icu4x/issues/7431) for the real-world
    /// bug motivating this warning.
    ///
    /// If using an overly-long `'stable` lifetime (such that the view might be invalidated before
    /// `'stable` ends, by an operation on the source `Data` value not covered by the three cases),
    /// great caution must be taken. For example, a reference passed to a function could, roughly
    /// speaking, be "used" (and have spurious compiler-inserted "fake reads") at many points
    /// throughout the function, such as the start and end of the function. If the view
    /// will be invalidated partway through the function, then the view should be wrapped in
    /// [`MaybeUninit`] (or `MaybeDangling`, once the latter is stabilized). For more about this
    /// constraint, see
    /// [this comment](https://github.com/unicode-org/icu4x/issues/3696#issuecomment-1642572298) of
    /// "Yoke vs strong protection". See the following for a brief mention of using "fake reads"
    /// to model the dereferenceability of references:
    /// <https://github.com/rust-lang/unsafe-code-guidelines/issues/381>.
    ///
    /// # Soundness of Implementation
    ///
    /// Below is an analytical approach to describing this trait, though looking at the source
    /// code of this crate (`alloc_impls.rs` in particular) may be more helpful.
    ///
    /// While this trait is a safe trait, this method still needs to be implemented *soundly*, and
    /// that includes upholding the `unsafe` contract of this method. Actually, if the
    /// implementation of this method has no `unsafe`, it is necessarily sound; only a `&'a Data` is
    /// provided, so stable data cannot be obtained safely (since unless `'a = 'data`, `'stable`
    /// could be longer than `'a` and shorter than `'data`), and any long-lived data is not
    /// invalidated by any operations on the source `Data` value for at least `'data`. (See
    /// [`concepts_and_safety`] for the meanings of "stable" and "long-lived".) It is an `unsafe`
    /// transmute from `'a` data to `'stable` stable data which needs to be careful to uphold the
    /// guarantees of this method.
    ///
    /// The guarantees about the first and second operation kinds prohibit returning stable
    /// references to inline data (for example, `Option<T>` cannot provide an `&'stable Option<T>`
    /// or `Option<&'stable T>` view referencing its data) **or** to data behind `noalias` pointers,
    /// such as `&mut T` and (currently) `Box<T>`. See [`concepts_and_safety`] for more about
    /// `noalias` pointers. It suffices to think of a `&mut T` or `Box<T>` as being equivalent to a
    /// `T` stored inline, for the purposes of obtaining `'stable` data.
    ///
    /// The guarantee about the third operation primarily restricts how `Data` uses internal
    /// mutability; for example, `Mutex<Vec<u8>>` certainly wouldn't be able to provide a
    /// `&'stable [u8]` view to its contents, even though `Vec<u8>` can. A hypothetical
    /// `struct InvalidateOnMove(UnsafeCell<Vec<u8>>, Cell<usize>)` wouldn't be able to
    /// provide a `&'stable [u8]` view if it had the following method, even if moves and permitted
    /// coercions alone or `&self` methods alone would not invalidate the view:
    ///
    /// ```ignore
    /// // Suppose this method is called at the start of any method which accesses `self.0`,
    /// // strictly before accessing `self.0`. (Since `Self`'s own code is trusted by this impl,
    /// // this `fn` need not be `unsafe`.)
    /// pub fn invalidate_on_move(&self) {
    ///     let this: *Self = self;
    ///     let this_addr = this.addr();
    ///     if self.1.replace(this_addr) != this_addr {
    ///         // SAFETY: Since
    ///         // - `Self: !Send`,
    ///         // - we are called at the start of any method which accesses `self.0` (strictly
    ///         //   before `self.0` is accessed),
    ///         // - our address has changed since the last time we were called,
    ///         // it follows that any previous borrow of `self.0` has ended (else, `self` could not
    ///         // have been moved). We can therefore obtain (short-lived) exclusive access over it.
    ///         let buf: &mut Vec<u8> = unsafe { &mut *self.0.get() };
    ///         *buf = Vec::new();
    ///     }
    /// }
    /// ```
    ///
    /// The following rough guidelines should be sufficient for soundly implementing this trait:
    /// - Returning `'stable` references to data guaranteed to be on the heap is sound, *except* for
    ///   data behind `&mut T` or `Box<T>`. Those two types currently assert exclusive access over
    ///   their pointees when moved. Treat moving a `&mut T` or `Box<T>` value as though it directly
    ///   moves the `T`.
    ///
    ///   Note that `CString` internally uses `Box<[u8]>`, so `Cow<'a, CStr>` and `CString` have the
    ///   same problem as `Box`. However, most similar `std` types (including, for instance,
    ///   `String`, as well as the current implementations of `OsString` and `PathBuf`) internally
    ///   use `Vec<u8>`, which does *not* currently have stringent aliasing requirements.
    ///
    ///   For further reading, see "Yoke vs noalias", especially this comment:
    ///   <https://github.com/unicode-org/icu4x/issues/2095#issuecomment-1200095048>
    ///
    ///   Types in `std` guaranteed to store data on the heap (or in a sufficiently-long-lived part
    ///   of the stack) include `Vec<T>`, `String`, `Cow<'b, [T]>`, and `Cow<'b, str>` where
    ///   `'b: 'data`.
    ///
    ///   Many Rust types like `HashMap` currently do not explicitly guarantee that they store all
    ///   their keys and values on the heap. See
    ///   <https://internals.rust-lang.org/t/could-collections-hypothetically-store-keys-and-values-inline/24195>.
    /// - Building off the previous point, do not rely on implementation details. Wrapping something
    ///   in an [`AliasableBox`] is far cheaper than the cost of relying on implementation details.
    ///
    ///   It may seem tempting to assert that moving a `bumpalo::Bump` does not invalidate its
    ///   allocations (meaning that lifetime-extending its allocations to `'stable` is sound), or to
    ///   assert that a `memmap2::Mmap`'s memory-mapped slice can soundly be extended to `'stable`.
    ///   You might even be correct, to some extent. Still, you should only rely on details that are
    ///   guaranteed.
    /// - Avoid returning references to data that may be on the stack, except for references valid
    ///   for at least `'data` (which refer to data in long-lived stack frames).
    ///
    ///   For instance, where `'b: 'data`, if `Data` is similar to `&'b T` or
    ///   [`AliasableRefMut<'b, T>`], then views of `Data` can soundly contain references of
    ///   lifetime `'b` (or other pointers guaranteed to be valid for at least lifetime `'data`) to
    ///   that `T` referenced by `Data`, which can be soundly shorted to `'stable` references.
    /// - Where `Data` is some `Generic<T>` type which can coerce to `Generic<UnsizedType>` or
    ///   `Generic<SuperType>`, don't provide pathological methods on *any* of those three types
    ///   (not just `Generic<T>` is restricted).
    /// - Don't check the address of `&Data` to decide whether old views of `Data` should be
    ///   invalidated. Don't try to detect whether a by-value coercion occurred (which would also
    ///   move the source `Data`) to decide whether old views of `Data` should be invalidated.
    ///
    /// ## Pathological Immutable Methods
    ///
    /// That last point deserves to be expanded on. One threat to the ability to lifetime-extend
    /// the `'stable` lifetime of views and continue accessing them (interspersed with moves,
    /// coercions, and shared or immutable access to the source `Data`), is the ability for wacky
    /// `&Data` methods to detect whether the source `Data` is moved or coerced and trigger
    /// pathological behavior.
    ///
    /// Note that we do not need to add more general safety conditions prohibiting `&Data` methods
    /// from mutating the pointees of shared pointers, accessing the pointees of exclusive pointers,
    /// or writing invalid data to their pointees; except in this edge case about manual move or
    /// coercion checks, such requirements are already required of all sound Rust code. If a `&Data`
    /// method were to invalidate a pointer in a view returned by this method (via mutating or
    /// accessing the pointee, or invalidating some transitive invariant of the pointee) when the
    /// source `Data` had not been moved, then entirely safe code could obtain a view, call the
    /// problematic `&Data` method, and trigger UB by using the old view.
    ///
    /// It would, however, be possible for a problematic `&Data` method to invalidate a previous
    /// view if safe Rust would not be able to access that view, which is the case after `Data` is
    /// moved. Some function on the view type could also take a `&Data` argument and choose to
    /// manually invalidate a view if its source `Data` was moved. Therefore, for the sake of being
    /// absolutely thorough, we explicitly forbid silly edge cases like that.
    ///
    /// ## `StableClone`
    ///
    /// Note that if `T: StableView` and `T` can be coerced to type `U`, then performing the three
    /// permitted kinds of operations on values of type `U` that had been coerced from type `T` must
    /// not invalidate views obtained from `<T as StableView>::view`, even if `U: !StableView`. The
    /// most common way for this to occur is likely `dyn Trait` erasure, which should not be able to
    /// cause any problems. It seems unlikely that any problems from coercions could occur
    /// accidentally (that is, without intentionally invalidating views as discussed above).
    ///
    /// # `transmute` in `view(_mut)` Implementation
    /// A common pattern in implementations of this method may be something like the following:
    ///
    /// ```ignore
    /// let stable_eq_a: CustomView<'a, 'a, 'data, Data, Self> = data;
    ///
    /// // SAFETY: See "`transmute` in `view(_mut)` Implementation" in the `StableView::view` docs.
    /// // <Explain why applying the three kinds of operations to the source `Data` value doesn't
    /// // invalidate the stable data in the returned view.>
    /// unsafe {
    ///     transmute::<
    ///         CustomView<'a, 'a, 'data, Data, Self>,
    ///         CustomView<'a, 'stable, 'data, Data, Self>,
    ///     >(stable_eq_a)
    /// }
    /// ```
    ///
    /// No further explanation of why the `transmute` is sound is necessary, since the caller of
    /// this method is responsible for soundly handling the `stable` lifetime; all you need to do is
    /// uphold the properties that the caller will assume of the source `Data` value and the
    /// `'stable` stable data.
    ///
    /// The same applies to [`StableViewMut::view_mut`], but with the three kinds of operations
    /// of [`StableViewMut`].
    ///
    /// [coercions]: https://doc.rust-lang.org/reference/type-coercions.html
    /// [`concepts_and_safety`]: crate::concepts_and_safety
    /// [`mem::drop`]: core::mem::drop
    /// [`mem::forget`]: core::mem::forget
    /// [`Deref`]: core::ops::Deref
    /// [`UnsafeCell`]: core::cell::UnsafeCell
    /// [`MaybeUninit`]: core::mem::MaybeUninit
    #[cfg_attr(feature = "alloc", doc = "[`AliasableBox`]: crate::aliasable::AliasableBox")]
    #[cfg_attr(not(feature = "alloc"), doc = "[`AliasableBox`]: https://docs.rs/stable-view/0/stable_view/struct.AliasableBox.html")]
    /// [`AliasableRefMut<'b, T>`]: crate::aliasable::AliasableRefMut
    /// [`StableViewer`]: crate::viewer::StableViewer
    /// [`ReferenceViewKind`]: crate::view_kinds::ReferenceViewKind
    /// [`DefaultViewKind`]: crate::view_kinds::DefaultViewKind
    #[must_use]
    unsafe fn view<'stable>(data: &'a Data) -> Varying<'stable, 'a, &'data (), Self::View>
    where
        'data: 'stable,
        'stable: 'a;
}

/// Get temporary mutable "views" whose `'stable` data is suitable for self-references to the
/// views' source `Data` values in self-referential structs.
///
/// The primary interface for using this trait is [`StableViewerMut`]; you should not need to
/// directly use this trait's method. Implementations of this trait are also provided by various
/// "view kinds", such as [`ReferenceViewKind`] and [`DefaultViewKind`], so you should not need
/// to implement this trait, either.
///
/// If you need a `'stable` reference to something not covered by existing implementations,
/// consider wrapping it in an [`AliasableBox`].
///
/// See the crate-level documentation for more.
///
#[cfg_attr(feature = "alloc", doc = "[`AliasableBox`]: crate::aliasable::AliasableBox")]
#[cfg_attr(not(feature = "alloc"), doc = "[`AliasableBox`]: https://docs.rs/stable-view/0/stable_view/struct.AliasableBox.html")]
/// [`StableViewerMut`]: crate::viewer::StableViewerMut
/// [`ReferenceViewKind`]: crate::view_kinds::ReferenceViewKind
/// [`DefaultViewKind`]: crate::view_kinds::DefaultViewKind
pub trait StableViewMut<
    'a, 'data, Data: ?Sized,
    __ImplyBound = &'a &'data (),
>: StableView<'a, 'data, Data, __ImplyBound> {
    /// A temporary mutable view of `Data` whose covariant `'stable` lifetime can be soundly
    /// lifetime-extended under specific conditions.
    type ViewMut: CovariantFamily<'a, &'data (), Is: Sized>;

    /// Get a temporary mutable view" whose `'stable` data is suitable for self-references to the
    /// given `Data` value in self-referential structs.
    ///
    /// The primary interface for using this trait is [`StableViewer`]; you should not need to
    /// directly use this method. Implementations of this trait are also provided by various
    /// "view kinds", such as [`ReferenceViewKind`] and [`DefaultViewKind`], so you should not need
    /// to implement this trait, either.
    ///
    /// If you need a `'stable` reference to something not covered by existing implementations,
    /// consider wrapping it in an [`AliasableBox`].
    ///
    /// See the crate-level documentation for more.
    ///
    /// The rest of this method's documentation is targeted at experience Rust programmers with a
    /// solid understanding of `unsafe`. Read [`concepts_and_safety`] before proceeding.
    ///
    /// # Safety
    ///
    /// Where the implementor's type is `Self` and the source data type is `Data`, while `'data`
    /// has not yet ended, any `'stable` data obtained via applying `Self`'s implementation of
    /// `StableViewMut<'a, 'data, Data>::view_mut::<'stable>` to a source `Data` value can be used
    /// at a given moment so long as, starting from when the view was created up to when it is used,
    /// only the following three kinds of operations (in any quantity and ordering) are applied to
    /// the source `Data` value:
    ///
    /// - moves, including any accompanying retag and other effects in the aliasing model, and
    /// - "permitted coercions", which are any type of [coercions] available in or before stable
    ///   Rust 1.85, except `Deref` and `DerefMut` coercions,
    /// - no-ops on the `Data` value (which don't access the `Data` value at all, though perhaps do
    ///   some work elsewhere).
    ///
    /// These constraints are the same as [`StableView::view`], except for not permitting immutable
    /// operations on the source `Data` value. See that method for full details on the meaning
    /// of this safety condition, as well as for advice on implementing this method soundly.
    ///
    /// [coercions]: https://doc.rust-lang.org/reference/type-coercions.html#r-coerce.types
    /// [`concepts_and_safety`]: crate::concepts_and_safety
    #[cfg_attr(feature = "alloc", doc = "[`AliasableBox`]: crate::aliasable::AliasableBox")]
    #[cfg_attr(not(feature = "alloc"), doc = "[`AliasableBox`]: https://docs.rs/stable-view/0/stable_view/struct.AliasableBox.html")]
    /// [`StableViewer`]: crate::viewer::StableViewer
    /// [`StableViewerMut`]: crate::viewer::StableViewerMut
    /// [`ReferenceViewKind`]: crate::view_kinds::ReferenceViewKind
    /// [`DefaultViewKind`]: crate::view_kinds::DefaultViewKind
    #[must_use]
    unsafe fn view_mut<'stable>(
        data: &'a mut Data,
    ) -> Varying<'stable, 'a, &'data (), Self::ViewMut>
    where
        'data: 'stable,
        'stable: 'a;
}

/// Extend the conditions under which the `'stable` data obtained via [`StableView`] views of this
/// type may be soundly used.
///
/// The safety requirements are closely modeled after the behavior of [`Rc`] and [`Arc`], though
/// other types like `()`, `&'b T` (where `'b: 'data`), and `Option<impl StableClone>` also
/// implement `StableClone`.
///
/// This trait's guarantees mainly impact the soundness of `unsafe` code, and implementing it
/// is also `unsafe`. As such, direct usage or implementation of this trait isn't part of the
/// primary interface of `stable-view`.
///
/// The rest of this method's documentation is targeted at experience Rust programmers with a
/// solid understanding of `unsafe`. Read [`concepts_and_safety`] before proceeding.
///
/// # Motivation
/// The restrictions on stable (non-long-lived) data in views returned by [`StableView`] are
/// sufficiently strict that, for [`StableView`] implementations of many types, any stable data in
/// the [`StableView`] impl's views **must** come from a specific, known part of the types.
///
/// (Note that `'a = 'stable = 'data` is possible in the [`StableView`] impl, in which case
/// temporary `'a` references to an arbitrary part of a type are the same as `'stable` references to
/// that part of the type. This scenario doesn't contradict the motivation, since those `'stable`
/// references live for at least `'data`, and are thus considered to be long-lived rather than
/// stable data.)
///
/// A `Data` implementor of this trait asserts, speaking roughly, that `Data` knows all possible
/// sources of stable data obtained from views of values of type `Data`, *and* that those sources
/// of stable data are either reference-counted (or similar) or remain valid for at least `'data`.
///
/// For example, stable data in views of `Rc<T>` must come from references to the `Rc<T>`'s `T`
/// pointee. (This includes the case where the directly-contained `T` provides a reference to some
/// deeper-nested `T`, for some suitable concrete `T` type.) That pointee is reference-counted,
/// so `Rc<T>` can implement `StableClone`.
///
/// The same holds of `Option<Rc<T>>`. Any stable data in a view of `Option<U>` must be stable data
/// of its `Some` variant, and any stable data of `Rc<T>` is reference counted. Additionally,
/// cloning a `Some` clones the wrapped value, increasing the `Rc<T>`'s reference count; all
/// stable data in an `Option<Rc<T>>` must be, speaking somewhat roughly, reference-counted by the
/// `Clone` impl of `Option<Rc<T>>`.
///
/// Conversely, stable data in views of `AliasableBox<Rc<T>>` *could* be references to the
/// reference-counted `T` pointee, *or* be `&'stable Rc<T>` references. The `Rc<T>` itself is
/// *not* reference-counted, so `AliasableBox<Rc<T>>` cannot implement `StableClone`.
///
/// `Vec<T>` can provide a non-reference-counted `&'stable [T]`, so it likewise does not
/// implement `StableClone`.
///
/// However, a `struct DoubleIndirection(AliasableBox<Rc<T>>)` which never publicly exposes
/// a reference to its inner `Rc<T>` *could* potentially implement `StableClone`.
///
/// # Safety
/// The exact notion of "reference-counting" needs to be formalized. The definition used by this
/// trait focuses on pools of values.
///
/// `StableClone` is slightly more restrictive than [`StableView`]. Instead of a single source
/// `Data` value, there is instead a conceptual *pool* of source values, and the `'stable` data of
/// a view must not be invalidated as long as the source pool is nonempty. ([`StableView`]'s
/// requirements can be seen as a special case where no operations are guaranteed to increase the
/// size of the conceptual pool. Note also that `StableView`'s `Data` parameter is
/// `StableClone`'s `Self`, which is why the implementor of this trait is here referred to as
/// `Data` rather than `Self`.)
///
/// Any consistent definition of a "conceptual pool" for a `Data` type which satisfies the below
/// four requirements can be used. The definition need not be documented, but it ideally should be,
/// such that third-party code could also add elements to the conceptual pool.
///
/// All the requirements only apply until `'data` ends.
///
/// ## Requirement 1
///
/// A source `Data` value is always in exactly one nonempty pool, containing at least
/// itself. (Note that non-`Data` values may be in arbitrarily many pools, under `Data`'s
/// definition of conceptual pools.)
///
/// ## Requirement 2
///
/// A clone of a `Data` value produced via `Data`'s implementations of [`Clone::clone`] or
/// [`Clone::clone_from`] **must** be added to the conceptual pool which the source `Data` value is
/// in (at the time the clone is produced), under the pool definition of `Data`.
///
/// ## Requirement 3
///
/// Applying the three kinds of operations listed by [`StableView::view`] (moves, permitted
/// coercions, and operations done through `&` references) in any quantity and ordering to a `Data`
/// value in the pool **must not** remove that value from the pool.
///
/// Other operations, such as mutating or running the destructor of a value in the pool, *may*
/// (but are not guaranteed to) remove a value from the conceptual pool. Likewise, the pool may
/// (but is not guaranteed to) be emptied after `'data` ends.
///
/// ## Requirement 4
///
/// For ***any*** view kind `V` and lifetimes lifetimes `'a`, `'stable`, and `'d` such that
/// `V: StableView<'a, 'd, Data>` and `'d: 'data`, the stable data of a value of type
/// [`CustomView<'a, 'stable, 'd, Data, V>`] obtained from applying `V`'s impl of
/// [`StableView<'a, 'd, Data>::view`] to some source `Data` value in the pool **must**
/// not be invalidated so long as its source pool is nonempty and `'data` has not ended.
///
/// Note that changing the conceptual pool to which the source `Data` value is associated (likely
/// by mutating it in some way) does not change the pool associated with the previously-produced
/// view, which is associated with the pool that its source `Data` value was in at the moment that
/// the view was obtained. A new view would be associated with that new pool, but the guaranteed
/// validity of a view is not solely tied to its original source value under [`StableClone`]'s
/// rules.
///
/// # Safety of Use
/// On the side of using `StableClone`, if `data_1` and `data_2` are in the same conceptual pool,
/// then (at your choice) you can soundly pretend (for the purposes of the rules of [`StableView`])
/// that `'stable` data obtained from views of `data_1` had actually been obtained from views of
/// `data_2` just after `data_2` entered the conceptual pool.
///
/// # `StableClone + StableViewMut`
/// For a pool of size at least 2, mutating one value in the pool and obtaining a mutable view
/// via [`StableViewMut`] (possibly reducing the size of the pool to 1) is not permitted to
/// invalidate the views associated with the pool. (For instance, mutating one `Rc` cannot
/// invalidate references to the refcounted data while a sibling `Rc` clone exists.)
///
/// However, a valid mutable reference cannot overlap with any other valid references; therefore,
/// the produced mutable view must not overlap with any of the still-valid immutable views in the
/// pool, possibly including past immutable views of the same source value.
///
/// A view kind could implement [`StableViewMut<'_, '_, Data>`] where `Data: StableClone`
/// perhaps by having two entirely different sets of data which are accessed by `'stable` data in
/// [`StableView`] and [`StableViewMut`], or via operations like [`Rc::get_mut`] and
/// [`Rc::make_mut`]. One of the two immutable/mutable view types might not contain any `'stable`
/// data at all.
///
/// As such, those two traits are not incompatible, even if they might at first seem to be.
///
/// # Soundness of relying on `Clone`
/// As seen during the stabilization of `dyn Allocator` in the standard library, a `dyn`-compatible
/// `unsafe` trait is generally incapable of placing constraints on how a different safe trait is
/// optionally implemented. See <https://github.com/rust-lang/rust/issues/156920> for details.
///
/// Users of `StableClone` can rely on its constraints on the `Clone` trait because the
/// `Data: Clone` bound is not optional:
/// <https://github.com/rust-lang/rust/issues/156920#issuecomment-4543098759>.
///
/// [coercions]: https://doc.rust-lang.org/reference/type-coercions.html
/// [`concepts_and_safety`]: crate::concepts_and_safety
/// [`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html
/// [`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
/// [`Rc::get_mut`]: https://doc.rust-lang.org/std/rc/struct.Rc.html#method.get_mut
/// [`Rc::make_mut`]: https://doc.rust-lang.org/std/rc/struct.Rc.html#method.make_mut
pub unsafe trait StableClone<'data>: Clone {}
