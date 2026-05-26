//! The core interface of this crate.

#![expect(unsafe_code, reason = "Allow `unsafe` code to rely on implementations being correct")]

use variance_family::{CovariantFamily, Varying};


/// The [`StableView::View`] associated with some `Data` and view kind (such as [`PointerViewKind`]
/// or [`DefaultViewKind`]).
///
/// [`PointerViewKind`]: crate::view_kinds::PointerViewKind
/// [`DefaultViewKind`]: crate::view_kinds::DefaultViewKind
pub type CustomView<'a, 'stable, 'other_data, Data, V>
    = Varying<'stable, 'a, &'other_data (), <V as StableView<'a, 'other_data, Data>>::View>;

/// The [`StableViewMut::ViewMut`] associated with some `Data` and view kind (such as
/// [`PointerViewKind`] or [`DefaultViewKind`]).
///
/// [`PointerViewKind`]: crate::view_kinds::PointerViewKind
/// [`DefaultViewKind`]: crate::view_kinds::DefaultViewKind
pub type CustomViewMut<'a, 'stable, 'other_data, Data, V>
    = Varying<'stable, 'a, &'other_data (), <V as StableViewMut<'a, 'other_data, Data>>::ViewMut>;

/// A trait for types with temporary views that are somewhat stable.
///
/// The `'stable` lifetime of these views can be soundly lifetime-extended under specific
/// conditions.
///
/// This trait is intended to be useful for self-referential types, though it might serve minor
/// utility in some other data structures.
///
/// Generally, it is low-level glue; it is expected that most transitive users should not need to
/// touch this trait at all, while some users may need to create a custom view of a data structure
/// by implementing this trait, and even fewer should end up needing direct (and `unsafe`) usage
/// of implementations of this trait.
///
/// As such, this trait and its documentation are verbose and targeted at experienced Rust
/// programmers, who have a solid understanding of `unsafe` but perhaps not of obstacles specific
/// to self-referential structs. Here be dragons, so I believe that the soundness of using *and*
/// implementing this trait should be thoroughly understood before attempting to do either.
///
/// # Parameters
/// The implementor, `Self`, is a view kind, such as [`PointerViewKind`] or [`DefaultViewKind`].
///
/// `'a` represents a short lifetime with no extra guarantees beyond the languages's invariants
/// enforced by the borrow checker.
///
/// `'stable` represents the lifetime of data which can be accessed longer than usual; this lifetime
/// is the one which can be soundly (and `unsafe`ly) lifetime-extended in specific conditions.
///
/// `'other_data` represents a long lifetime; all views will stop being used before `'other_data`
/// ends.
///
/// `Data` is the type of the source data of the view. (It is a separate parameter, rather than
/// `Self`, in order to allow multiple kinds of views to be used on a single type, possibly
/// including custom user-written views.)
///
/// # Safety
/// Where the implementor's type is `Self` and the source data type is `Data`, while `'other_data`
/// has not yet ended, any `'stable` data (such as `&'stable` references) in a value of type
/// [`CustomView<'a, 'stable, 'other_data, Data, Self>`] obtained from applying `Self`'s
/// [`StableView::view`] impl to a source `Data` value must not be invalidated by applying the
/// following three operations (in any quantity and ordering) to the source `Data` value:
/// - moves,
/// - [coercions] (which may or may not involve moves, and may read arbitrary data thanks to
///   user-defined deref coercions),
/// - any (sound) operations which use data derived from the source `Data` value only through
///   shared/immutable `&` references to the relevant parts of `Data`. (These could be called
///   "immutable operations" on the source `Data` value, if not for internal mutability within
///   `Data`, which could escalate a `&` reference to part of `Data` to a `&mut` reference
///   to another part of `Data`.)
///
/// ## Elaboration
///
/// For example, this covers coercing the `Data` value to something else, performing an immutable
/// operation on the coerced value, coercing it to yet another type, and moving that doubly-coerced
/// value.
///
/// Actions with no effect on the source value of a view, including *not* running its destructor
/// (perhaps after moving it into `Box::leak`), are trivially permitted as no-ops. Technically,
/// they are vacuously permitted through the third case, as "all" of the data derived from the
/// source value which they (don't) use is used only through `&` references.
///
/// # Step-by-Step Safety Breakdown
///
/// The clause about `'other_data` ending ensures that, say, `&'b T` can implement this
/// trait with a `&'stable T` view where `'b: 'other_data`, even though the `&'stable T` may be
/// invalidated after lifetime `'b` ends (even if the source `Data = &'b T` value is only ever
/// moved, coerced, or accessed immutably, noting that `Copy` types are guaranteed to have no
/// destructor to be run).
///
/// "The following three operations (in any quantity and ordering)" covers, for example, coercing
/// the `Data` value to something else, performing an immutable operation on the coerced value,
/// coercing it to yet another type, and moving that doubly-coerced value.
///
/// The guarantee of the first operation prohibits returning `'stable` references to inline data
/// (for example, `Option<T>` cannot provide an `&'stable Option<T>` or `Option<&'stable T>` view
/// referencing its data) **or** to data behind `noalias` pointers, such as `&mut T` and `Box<T>`.
///
/// **WARNING**: This is delicate and warrants repeating: `&mut T` and `Box<T>` cannot provide
/// `&'stable T` references to their directly-referenced `T` data. Moving a `&mut T` or `Box<T>`
/// value has roughly the same effect as directly moving the `T`. (See below for more details.)
///
/// The guarantee of the second operation is arguably covered by the other two cases, but it's
/// included for the sake of caution.
///
/// The guarantee of the third operation primarily restricts how `Data` uses internal mutability;
/// for example, `Mutex<Vec<u8>>` certainly wouldn't be able to provide a `&'stable [u8]` view, even
/// though `Vec<u8>` can. A hypothetical
/// `struct InvalidateOnMove(UnsafeCell<Vec<u8>>, Cell<usize>)` wouldn't be able to
/// provide a `&'stable [u8]` view if it had the following method, even if moves and coercions
/// alone or `&self` methods alone would not invalidate the view:
///
/// ```ignore
/// // Suppose this method is called at the start of any method which accesses `self.0`, strictly
/// // before accessing `self.0`. (Since `Self`'s own code is trusted by this impl, this `fn`
/// // need not be `unsafe`.)
/// pub fn invalidate_on_move(&self) {
///     let this: *Self = self;
///     let this_addr = this.addr();
///     if self.1.replace(this_addr) != this_addr {
///         // SAFETY: Since
///         // - `Self: !Send`,
///         // - we are called at the start of any method which accesses `self.0` (strictly before
///         //   `self.0` is accessed),
///         // - our address has changed since the last time we were called,
///         // it follows that any previous borrow of `self.0` has ended (else, `self` could not
///         // have been moved). We can therefore obtain (short-lived) exclusive access over it.
///         let buf: &mut Vec<u8> = unsafe { &mut *self.0.get() };
///         *buf = Vec::new();
///     }
/// }
/// ```
///
/// # Implications of Safety Requirements for Users
/// ## Sound usage of a view
/// A returned view can be used at a given moment so long as, starting from when the view
/// was created up to when it is used, only those three operations are performed.
///
/// In particular, a returned view may be soundly lifetime-transmuted from
/// `CustomView<'a, 'stable, 'other_data, Data, Self>` to
/// `CustomView<'a, 's, 'other_data, Data, Self>` for any lifetime `'s` such that only those
/// three operations are performed on the source `Data` during that lifetime `'s`, and the
/// resulting `CustomView<'_, 's, '_, Data, Self>` value can be soundly exposed to arbitrary
/// (sound) code, as the view would remain valid during its entire lifetime.
///
/// Extending `'stable` to a fake lifetime like `'static` may be sound if you are careful to expose
/// that view only to code aware that the lifetime annotation is a lie; in that case, the view
/// must only be accessed when it can be proven that the view was not invalidated.
///
/// Note that the `view` function itself may need to perform a lifetime transmute to the `'stable`
/// lifetime, thus why it is `unsafe` and requires that these rules of sound usage be manually
/// enforced; the borrow checker will not help. An *additional* unsafe lifetime transmute after
/// using `view` is likely unnecessary.
///
/// ## Dangers of lifetime-transmuting a view
/// Functions that take owned `Data` arguments or exclusively-borrowed `&mut Data` arguments
/// (or which can transitively access an owned `Data` or `&mut Data`), including [`Drop::drop`],
/// [`mem::drop`], and [`StableViewMut::view_mut`], are (in general) allowed to invalidate
/// previously-returned views of those `Data` values (or to enable safe code to later invalidate
/// previously-returned views). Some functions, such as `Box::new` (*when it does not unwind after
/// OOM*) and [`mem::forget`], may be known to only perform permitted operations (possibly only
/// under certain conditions), but be cautious.
///
/// (Note that [`mem::forget`] does invalidate the location of a `Data` value, but a sound
/// implementation of this type cannot hand out views which reference data stored inline in the
/// source `Data`; otherwise, moving a `Data` value could invalidate references in its views.
/// [`mem::forget`] could perhaps be seen as semantically moving the `Data` value to some location
/// that can never be accessed again.)
///
/// As views may have nontrivial destructors, dropping an unsafely lifetime-extended view may
/// count as a usage of that view; if a view is not known to have no drop glue, be careful not to
/// perform any operation that could invalidate a view before dropping it. In particular, drop (or
/// leak) views before dropping the `Data` source of those views.
///
/// For example, when working with panicky functions which only invalidate the `Data` source on
/// error (perhaps by dropping the `Data` value during unwinding), such as `Box::new(data)`, one
/// sound approach is to wrap views in `ManuallyDrop` before calling the panicky function and only
/// unwrap the views after the function's successful return; this ensures that views are not
/// improperly accessed in their destructors during unwinding. A leak is far preferable to UB.
/// (Using `Box::new_uninit()` to avoid unexpectedly dropping `data` is also possible.)
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
/// # More Details for Implementors
/// Below is an analytical approach to describing this trait, though looking at the source
/// code of this crate ([`alloc_impls.rs`] in particular) may be more helpful.
///
/// Let `CustomView<'a, 'stable, 'other_data, Data, Self>` be abbreviated as `Self::View<'stable>`.
///
/// To elaborate on what is meant by the prohibition against certain operations "invalidating"
/// views, it must be sound to lifetime-extend the `'stable` lifetime of a `Self::View<'stable>`
/// value and continue using it as long as operations on its source `Data` value are limited to the
/// three stated cases (and as long as `'other_data` does not end). It suffices to ensure that:
/// - The pointees of pointers in the `Self::View<'stable>` view which are assumed to be valid for
///   shared access during `'stable`, such as `&'stable T` or [`cell::Ref<'stable, T>`], are not
///   mutated (except inside [`UnsafeCell`]) or otherwise exclusively accessed by moves or coercions
///   of values of type `Data` (or of formerly-`Data` values coerced to a different type).
///   (This essentially implies that the pointees cannot be stored inline in `Data`; they must
///   either be in static memory, on the heap, in some part of the stack that outlives `Data`, or
///   similar.)
/// - The pointees of pointers in the `Self::View<'stable>` view which are assumed to be valid for
///   exclusive access during `'stable`, such as `&'stable mut T` or [`cell::RefMut<'stable, T>`],
///   are not accessed by moves or coercions of values of type `Data` (or of formerly-`Data` values
///   coerced to a different type). (This again implies that the pointees cannot be stored inline
///   in `Data`.)
/// - Exclusive access is not asserted by any of the three operations over the pointees of pointers
///   in the view assumed to be valid for accesses during `'stable`.
/// - No creative shenanigans are performed in functions that access `&Data` that cause UB when a
///   value of type `Data` is moved or coerced but old views continue to be used. (See above
///   `InvalidateOnMove` example.)
///
/// The first three requirements are a matter of pointer [provenance] and permissions, and ensures
/// that the permissions of any pointers or references derived from pointers with a `'stable`
/// lifetime in a `Self::View<'stable>` view are not shortened, reduced, or removed when the source
/// `Data` is moved or coerced. (The Rust Abstract Machine knows nothing about the stack, heap, or
/// static memory, so they are most pedantically expressed in terms of mutation and accesses, but in
/// practice the first two requirements are about where the pointees are stored.)
/// The fourth requirement ensures that manual checks of invariants that would hold of safe Rust,
/// but not of `unsafe` code utilizing the aliasing guarantees provided by `StableView`, are not
/// allowed to trigger undefined behavior.
///
/// The following rough guidelines should be sufficient:
/// - Returning `'stable` references to data guaranteed to be on the heap is sound, *except* for
///   data behind `&mut T` or `Box<T>`. Those two types currently assert exclusive access over
///   their pointees when moved. Moving a `&mut T` or `Box<T>` value has roughly the same effect
///   as directly moving the `T`.
///
///   Note that `CString` internally uses `Box<[u8]>`, but most similar `std` types (including,
///   for instance, `String`, `OsString`, and `PathBuf`) internally use `Vec<u8>`, which does
///   *not* currently have stringent aliasing requirements. It would be best to avoid
///   `Cow<'a, CStr>` and `CString`, though I am not certain that using them would trigger UB.
// TODO: Miri on Rust Playground doesn't see UB on `CString`. That may just be because the
// playground doesn't have Miri recursively retag fields. This should be further explored.
///
///   For further reading, see "Yoke vs noalias", especially this comment:
///   <https://github.com/unicode-org/icu4x/issues/2095#issuecomment-1200095048>
///
///   Types in `std` guaranteed to store data on the heap (or in a sufficiently-long-lived part
///   of the stack) include `Vec<T>`, `String`, `Cow<'b, [T]>`, and `Cow<'b, str>` where
///   `'b: 'other_data`.
///
///   Many Rust types like `HashMap` currently do not explicitly guarantee that they store all
///   their keys and values on the heap. See
///   <https://internals.rust-lang.org/t/could-collections-hypothetically-store-keys-and-values-inline/24195>.
/// - Avoid returning references to data that may be on the stack, except for references valid
///   for at least `'other_data` (which refer to data in long-lived stack frames).
///
///   For instance, where `'b: 'other_data`, if `Data` is similar to `&'b T` or
///   [`AliasableRefMut<'b, T>`], then views of `Data` can soundly contain references of lifetime
///   `'b` (or other pointers guaranteed to be valid for at least lifetime `'other_data`) to that
///   `T` referenced by `Data`, which can be soundly shorted to `'stable` references.
/// - Don't check the address of `&Data` to decide whether old views of `Data` should be
///   invalidated. Don't try to detect whether a by-value coercion occurred (which would also
///   move the source `Data`) to decide whether old views of `Data` should be invalidated.
///
/// ## Justification
/// The first three requirements are sufficient to imply that moving or coercing values of type
/// `Data` does not invalidate pointers that are required by this trait to remain valid.
/// In particular, the first two requirements ensure that the pointees of pointers in views are
/// not stored inline in the `Data` value; otherwise, a `Data` and its views stored in local
/// variables on the stack could be returned from a function, causing the views to reference data
/// in a deallocated stack frame. (Such a scenario would assert exclusive access over the pointees
/// and/or be considered to write uninit data to the relevant pointers' pointees; therefore, such
/// a situation is prohibited by the first two requirements.) The third requirement ensures that
/// retags introduced by moving a `&'b mut T` (and, currently, `Box<T>`, among other types) do not
/// invalidate the provenance or permissions of views.
///
/// The last remaining threat to the ability to lifetime-extend views and continue accessing them
/// (interspersed with moves, coercions, and shared or immutable access to the source `Data`), is
/// the ability for wacky `&Data` methods to detect whether the source `Data` is moved or coerced
/// and trigger pathological behavior. Note that we do not need to add more general safety
/// conditions prohibiting `&Data` methods from mutating the pointees of shared pointers,
/// accessing the pointees of exclusive pointers, or writing invalid data to their pointees; except
/// in this edge case about manual move or coercion checks, such requirements are already required
/// of all sound Rust code. If a `&Data` method were to invalidate a pointer in a view returned by
/// [`StableView::view`] (via mutating or accessing the pointee, or invalidating some transitive
/// invariant of the pointee) when the source `Data` had not been moved, then entirely safe code
/// could obtain a view, call the problematic `&Data` method, and trigger UB by using the old view.
/// It would, however, be sound for a problematic `&Data` method to invalidate a previous view if
/// safe Rust would not be able to access that view, which is the case after `Data` is moved.
/// Some function on the view type could also take a `&Data` argument and choose to manually
/// invalidate a view if its source `Data` was moved. Therefore, for the sake of being absolutely
/// thorough, we explicitly forbid silly edge cases like that.
///
/// Additionally, if `T: StableView` and `T` can be coerced to type `U`, then performing the
/// three permitted operations on values of type `U` that had been coerced from type `T` must not
/// invalidate views obtained from `<T as StableView>::view`, even if `U: !StableView`. The
/// most common way for this to occur is likely `dyn Trait` erasure, which should not be able to
/// cause any problems. It seems unlikely that any problems from coercions could occur accidentally
/// (that is, without intentionally invalidating views as discussed above).
///
/// # `transmute` in `view` Implementation
/// A common pattern in implementations of [`StableView::view`] may be something like the following:
///
/// ```ignore
/// let stable_eq_a: CustomView<'a, 'a, 'other_data, Data, Self> = data;
///
/// // SAFETY: See above the safety comment of the `unsafe impl` of `StableView`. Additionally,
/// // the caller of `view` unsafely asserts that the returned view is only used when the source
/// // data has only been moved or coerced (or had no-ops occur) from just after this function
/// // returns (and, therefore, also starting from now, since we have a `&` borrow of the source
/// // data) until the time of use, and that `'other_data` has not ended when it's used. By the same
/// // reasoning that enables the `unsafe` trait impl, we know that those uses do not invalidate
/// // `'stable` data and that lifetime extension of the `'stable` lifetime parameter is sound. Any
/// // further soundness concerns are the responsibility of the caller of `view`.
/// unsafe {
///     transmute::<
///         CustomView<'a, 'a, 'other_data, Data, Self>,
///         CustomView<'a, 'stable, 'other_data, Data, Self>,
///     >(stable_eq_a)
/// }
/// ```
///
/// Instead of needing to write out a long explanation each time, just say:
///
/// ```ignore
/// let stable_eq_a: CustomView<'a, 'a, 'other_data, Data, Self> = data;
///
/// // SAFETY: See the "`transmute` in `view` Implementation" section of the `StableView` docs.
/// unsafe {
///     transmute::<
///         CustomView<'a, 'a, 'other_data, Data, Self>,
///         CustomView<'a, 'stable, 'other_data, Data, Self>,
///     >(stable_eq_a)
/// }
/// ```
///
/// # Prior Art
///
/// This trait is similar to [`AliasableDeref`], but supporting an arbitrary lifetime-infected
/// type rather than the [`Deref`] trait's `&'_ Self::Target` return type; its intended use case is
/// also similar to that of [`StableDeref`], but the requirement that repeatedly calling `deref`
/// (or `view` in this case) returns the same value is unnecessary for soundness of
/// self-referential types. (Moreover, `StableDeref`'s implementation for `&mut T` [is unsound],
/// and its implementation for `Box<T>` is debatably unsound.)
///
/// [coercions]: https://doc.rust-lang.org/reference/type-coercions.html
/// [provenance]: https://doc.rust-lang.org/std/ptr/index.html#provenance
/// [`mem::drop`]: core::mem::drop
/// [`mem::forget`]: core::mem::forget
/// [`ManuallyDrop::new`]: core::mem::ManuallyDrop::new
/// [`Deref`]: core::ops::Deref
/// [`cell::Ref<'stable, T>`]: core::cell::Ref
/// [`cell::RefMut<'stable, T>`]: core::cell::RefMut
/// [`UnsafeCell`]: core::cell::UnsafeCell
/// [`MaybeUninit`]: core::mem::MaybeUninit
/// [`AliasableRefMut<'b, T>`]: crate::aliasable::AliasableRefMut
/// [`DefaultViewKind`]: crate::view_kinds::DefaultViewKind
/// [`PointerViewKind`]: crate::view_kinds::PointerViewKind
/// [`Self::View`]: StableView::View
/// [`alloc_impls.rs`]: https://github.com/robofinch/lifetime-foundry/blob/main/crates/stable-view/src/alloc_impls.rs
/// [`AliasableDeref`]: https://docs.rs/aliasable_deref_trait/1.0.0/aliasable_deref_trait/trait.AliasableDeref.html
/// [`StableDeref`]: https://docs.rs/stable_deref_trait/1.2.1/stable_deref_trait/trait.StableDeref.html
/// [is unsound]: https://github.com/Storyyeller/stable_deref_trait/issues/15#issuecomment-3714995546
pub unsafe trait StableView<'a, 'other_data, Data: ?Sized> {
    /// A temporary (but somewhat stable) view of the implementing type.
    type View: CovariantFamily<'a, &'other_data (), Is: Sized>;

    /// Get a temporary (but somewhat stable) view of this type.
    ///
    /// # Safety
    /// The returned view must only be used under the conditions described by the below robust
    /// guarantee. Any lifetime can be used for `'stable` between `'a` and `'other_data`, but
    /// if `'stable` is chosen to be too long, then exposing the view to arbitrary safe Rust code
    /// could cause undefined behavior.
    ///
    /// `'stable = 'a` is guaranteed to be sound.
    ///
    /// **WARNING**: Generally, the borrow checker will not automatically check that `'stable` is
    /// reasonable.
    ///
    /// # Robust Guarantee
    /// The `'stable` lifetime of the returned view can be soundly transmuted to any lifetime
    /// between `'a` and `'other_data`, though *using* the view might trigger undefined behavior
    /// if `'stable` is too long.
    ///
    /// While `'other_data` has not yet ended, the returned view can be used at a given moment so
    /// long as, starting from when the view is returned from this function up to when it is used,
    /// only the following three operations are performed on the source `Data` value (in any
    /// quantity and ordering):
    /// - moves,
    /// - [coercions] that may or may not involve moves,
    /// - any (sound) operations which use data derived from the source `Data` value only through
    ///   shared/immutable `&` references to the relevant parts of `Data`. (These could be called
    ///   "immutable operations" on the source `Data` value, if not for internal mutability within
    ///   `Data`, which could escalate a `&` reference to part of `Data` to a `&mut` reference
    ///   to another part of `Data`.)
    ///
    /// See the [trait-level documentation] for more about how returned views may be used, including
    /// **vital** warnings.
    ///
    /// [`MaybeUninit`]: core::mem::MaybeUninit
    /// [coercions]: https://doc.rust-lang.org/reference/type-coercions.html
    /// [trait-level documentation]: StableView#implications-of-safety-requirements-for-users
    #[must_use]
    unsafe fn view<'stable>(data: &'a Data) -> CustomView<'a, 'stable, 'other_data, Data, Self>
    where
        'other_data: 'stable,
        'stable: 'a;
}

/// A trait for types with temporary mutable views that are somewhat stable.
///
/// The `'stable` lifetime of these mutable views can be soundly lifetime-extended under specific
/// conditions.
///
/// This trait is intended to be useful for self-referential types, though it might serve minor
/// utility in some other data structures.
///
/// Generally, it is low-level glue; it is expected that most transitive users should not need to
/// touch this trait at all, while some users may need to create a custom view of a data structure
/// by implementing this trait, and even fewer should end up needing direct (and `unsafe`) usage
/// of implementations of this trait.
///
/// As this trait is an extension of [`StableView`], refer to its documentation for full details.
///
/// # Safety
/// Where the implementor's type is `Self` and the source data type is `Data`, while `'other_data`
/// has not yet ended, any `'stable` data (such as `&'stable` references) in a value of type
/// [`CustomViewMut<'a, 'stable, 'other_data, Data, Self>`] obtained from applying `Self`'s
/// [`StableViewMut::view_mut`] impl to a source `Data` value must not be invalidated by applying
/// the following three operations (in any quantity and ordering) to the source `Data` value:
/// - moves,
/// - non-deref [coercions] (which may or may not involve moves, and may read inline data),
/// - no-ops which don't access data derived from the source `Data` value (except data derived
///   from the returned mutable view).
///
/// ## Elaboration
///
/// For example, this covers coercing the `Data` value to something else, coercing it to yet
/// another type, and moving that doubly-coerced value.
///
/// The guarantee of the second operation is arguably covered by the first case, but it's
/// included for the sake of caution.
///
/// The third operation category notably includes *not* running the `Data` value's destructor
/// (perhaps after moving it into `Box::leak`). The third category is trivial, but it is included
/// to explicitly grant users that right.
///
/// # Implications of Safety Requirements for Users
/// Refer to the relevant section of [`StableView`]'s documentation. The only differing details are
/// mutability and exactly which three operations are guaranteed.
///
/// # More Details for Implementors
/// The relevant section of [`StableView`]'s documentation is only marginally more complicated
/// than what is necessary for [`StableViewMut`]: the fourth requirement, about pathological
/// cases like `InvalidateOnMove`, does not apply here.
///
/// Note that allowing non-deref coercions essentially comes for free from the requirement that
/// moves do not invalidate `'stable` data. Moves necessarily include a read of the data inline in
/// a `Data` allocation, so the first guarantee implies that reading inline data does not invalidate
/// `'stable` data. All coercions other than deref coercions, even unsizing coercions from
/// `Data = Rc<T>` to `Rc<dyn Trait>`, only read data inline in `Data`; they cannot read something
/// in `Data` and invalidate a `&'stable mut U` reference. Therefore, when implementing this trait,
/// you need only worry about the first and third operations, in addition to the details
/// of [`StableView`]. The soundness of the third operation, of course, is trivial, leaving only
/// the first operation for serious consideration.
///
/// # `transmute` in `view_mut` Implementation
/// A common pattern in implementations of [`StableViewMut::view_mut`] may be something like the
/// following:
///
/// ```ignore
/// let stable_eq_a: CustomViewMut<'a, 'a, 'other_data, Data, Self> = data;
///
/// // SAFETY: See above the safety comment of the `unsafe impl` of `StableViewMut`.
/// // Additionally, the caller of `view_mut` unsafely asserts that the returned view is only
/// // used when the source data has only been moved or coerced (or had no-ops occur) from just
/// // after this function returns (and, therefore, also starting from now, since we have a
/// // `&mut` borrow of the source data) until the time of use, and that `'other_data` has not
/// // ended when it's used. By the same reasoning that enables the `unsafe` trait impl, we know
/// // that those uses do not invalidate `'stable` data and that lifetime extension of the
/// // `'stable` lifetime parameter is sound. Any further soundness concerns are the
/// // responsibility of the caller of `view_mut`.
/// unsafe {
///     transmute::<
///         CustomViewMut<'a, 'a, 'other_data, Data, Self>,
///         CustomViewMut<'a, 'stable, 'other_data, Data, Self>,
///     >(stable_eq_a)
/// }
/// ```
///
/// Instead of needing to write out a long explanation each time, just say:
///
/// ```ignore
/// let stable_eq_a: CustomViewMut<'a, 'a, 'other_data, Data, Self> = data;
///
/// // SAFETY: See the "`transmute` in `view_mut` Implementation" section of the
/// // `StableViewMut` docs.
/// unsafe {
///     transmute::<
///         CustomViewMut<'a, 'a, 'other_data, Data, Self>,
///         CustomViewMut<'a, 'stable, 'other_data, Data, Self>,
///     >(stable_eq_a)
/// }
/// ```
///
/// [coercions]: https://doc.rust-lang.org/reference/type-coercions.html
pub unsafe trait StableViewMut<'a, 'other_data, Data: ?Sized>: StableView<'a, 'other_data, Data> {
    /// A temporary (but somewhat stable) mutable view of the implementing type.
    type ViewMut: CovariantFamily<'a, &'other_data (), Is: Sized>;

    /// Get a temporary (but somewhat stable) mutable view of this type.
    ///
    /// # Safety
    /// The returned mutable view must only be used under the conditions described by the below
    /// robust guarantee. Any lifetime can be used for `'stable` between `'a` and `'other_data`, but
    /// if `'stable` is chosen to be too long, then exposing the view to arbitrary safe Rust code
    /// could cause undefined behavior.
    ///
    /// `'stable = 'a` is guaranteed to be sound.
    ///
    /// **WARNING**: Generally, the borrow checker will not automatically check that `'stable` is
    /// reasonable.
    ///
    /// # Robust Guarantee
    /// The `'stable` lifetime of the returned mutable view can be soundly transmuted to any
    /// lifetime between `'a` and `'other_data`, though *using* the view might trigger undefined
    /// behavior if `'stable` is too long.
    ///
    /// While `'other_data` has not yet ended, the returned mutable view can be used at a given
    /// moment so long as, starting from when the mutable view is returned from this function up to
    /// when it is used, only the following three operations are performed on the source `Data`
    /// value (in any quantity and ordering):
    /// - moves,
    /// - non-deref [coercions] (which may or may not involve moves, and may read inline data),
    /// - no-ops which don't access data derived from the source `Data` value (except data derived
    ///   from the returned mutable view).
    ///
    /// See the [trait-level documentation] for more about how returned views may be used, including
    /// **vital** warnings.
    ///
    /// [`MaybeUninit`]: core::mem::MaybeUninit
    /// [coercions]: https://doc.rust-lang.org/reference/type-coercions.html
    /// [trait-level documentation]: StableViewMut#implications-of-safety-requirements-for-users
    #[must_use]
    unsafe fn view_mut<'stable>(
        data: &'a mut Data,
    ) -> CustomViewMut<'a, 'stable, 'other_data, Data, Self>
    where
        'other_data: 'stable,
        'stable: 'a;
}

/// Extend the conditions under which temporary views of this type may be soundly lifetime-extended
/// (or, in the case of raw pointers, continue to be soundly accessed).
///
/// This trait is intended to be useful for self-referential types, and it is generally intended
/// to be implemented for types that are reference-counted *or* provide owned "views" that are
/// never invalidated when the source `Data` is dropped.
///
/// The safety requirements are closely modeled after the behavior of [`Rc`] and [`Arc`].
///
/// # Safety
/// This trait is slightly more restrictive than [`StableView`]. Instead of a single source
/// `Data` value, there is instead a conceptual *pool* of source values, and the `'stable` data of
/// a view must not be invalidated as long as the source pool is nonempty. ([`StableView`]'s
/// requirements can be seen as a special case where no operations are guaranteed to increase the
/// size of the conceptual pool.)
///
/// Any consistent definition of a "conceptual pool" for a type which allows these requirements
/// to be satisfied can be used. The definition can vary with `Self` and `Data`. The definition need
/// not be documented, but it ideally should be, such that third-party code could also add elements
/// to the conceptual pool. Note that a data value must be in exactly one pool at all times, and a
/// view must be associated with exactly one pool at all times (although that pool may be empty in
/// the case of invalidated views).
///
/// ## Requirement 1
///
/// A source `Data` value is always in exactly one nonempty pool, containing at least itself.
/// (Note that *other* non-`Data` values may be in zero, one, two, or more pools.)
/// If `Self` implements [`StableClone<'_, '_, Data>`], then a clone of a `Data` value produced
/// via `Data`'s implementations of [`Clone::clone`] or [`Clone::clone_from`] **must** be added to
/// the conceptual pool which the source `Data` value is in (at the time the clone is produced),
/// under the pool definition of `Self` and `Data`.
///
/// ## Requirement 2
///
/// Applying the three operations listed by [`StableView::view`] (moves, [coercions], and operations
/// done through `&` references) in any quantity and ordering to a data value in the pool
/// **must not** remove that value from the pool.
///
/// Other operations, such as mutating or running the destructor of a value in the pool, *may*
/// (but are not guaranteed to) remove a value from the conceptual pool. Likewise, the pool may
/// (but is not guaranteed to) be emptied after `'other_data` ends.
///
/// ## Requirement 3
///
/// The `'stable` data of a value of type [`CustomView<'a, 'stable, 'other_data, Data, Self>`]
/// obtained from applying `Self`'s [`StableView::view`] impl to some source `Data` value in the
/// pool **must** not be invalidated so long as its source pool is nonempty.
///
/// Note that changing the conceptual pool to which the source `Data` value is associated (likely
/// by mutating it in some way) does not change the pool associated with the previously-produced
/// view. A new view would be associated with that new pool, but the guaranteed validity of a view
/// is not solely tied to its original source value under [`StableClone`]'s rules.
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
/// A type can implement both [`StableClone`] and [`StableViewMut`] perhaps by having
/// two entirely different sets of data which are accessed by `'stable` data in [`StableView`] and
/// [`StableViewMut`], or via operations like [`Rc::get_mut`] and [`Rc::make_mut`] which do not
/// necessarily separate the data. One sort of view might not contain any `'stable` data at all.
///
/// As such, those two `unsafe` traits are not incompatible.
///
/// # Soundness of relying on `Clone`
/// As seen during the stabilization of `dyn Allocator` in the standard library, a `dyn`-compatible
/// `unsafe` trait is generally incapable of placing constraints on how a different safe trait is
/// (optionally) implemented. See <https://github.com/rust-lang/rust/issues/156920> for details.
///
/// Users of `StableClone` can rely on its constraints on `Data`'s implementations of the safe
/// `Clone` trait because `StableClone` is not `dyn`-compatible. The fact that the `Data: Clone`
/// bound is not optional might also suffice:
/// <https://github.com/rust-lang/rust/issues/156920#issuecomment-4543098759>.
///
/// [coercions]: https://doc.rust-lang.org/reference/type-coercions.html
/// [`mem::forget`]: core::mem::forget
/// [`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html
/// [`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
/// [`Rc::get_mut`]: https://doc.rust-lang.org/std/rc/struct.Rc.html#method.get_mut
/// [`Rc::make_mut`]: https://doc.rust-lang.org/std/rc/struct.Rc.html#method.make_mut
pub unsafe trait StableClone<'a, 'other_data, Data: Clone>: StableView<'a, 'other_data, Data> {}
