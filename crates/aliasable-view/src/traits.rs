#![expect(unsafe_code, reason = "Allow `unsafe` code to rely on implementations being correct")]

use variance_family::{Lend, LendFamily, MaxUpperBound, UpperBound};


/// Shorthand for applying a `'a` lifetime parameter to the `T::View` lifetime family.
///
/// This requires that `T: 'a` (and `T: AliasableView`).
pub type View<'a, T, Upper = MaxUpperBound>
    = Lend<'a, Upper, <T as AliasableView<Upper>>::View>;

/// Shorthand for applying a `'a` lifetime parameter to the `T::ViewMut` lifetime family.
///
/// This requires that `T: 'a` (and `T: AliasableViewMut`).
pub type ViewMut<'a, T, Upper = MaxUpperBound>
    = Lend<'a, Upper, <T as AliasableViewMut<Upper>>::ViewMut>;

/// A trait for types with temporary views that are somewhat stable.
///
/// These views can be soundly lifetime-extended (or, in the case of raw pointers, continue to be
/// soundly accessed) under specific conditions.
///
/// This trait is intended to be useful for self-referential types.
///
/// # Safety
/// Where the implementor's type is `Self`, the following operations must not invalidate any value
/// of type [`View<'a, Self>`] obtained from applying [`AliasableView::view`] to a value in the
/// below bullet points:
/// - moving a value of type `Self` (or a value formerly of type `Self` that was coerced to a
///   different type),
/// - performing [coercions] on a value of type `Self` that may or may not involve moves,
/// - performing any (sound) operation on a value of type `&Self` (which includes arbitrary
///   operations on data transitively reachable via the `&Self` value).
///
/// Actions with no effect on the source value of a view, including *not* running its destructor
/// (perhaps after moving it into `Box::leak`), are trivially permitted as no-ops. The bullet point
/// for coercions is arguably covered by the bullet point for moves (and this note about no-ops),
/// but it's listed for the sake of caution.
///
/// ## Implications for Users
/// ### Sound usage of a view
/// A returned view can be used at a given moment so long as, starting from when the view
/// was created up to when it is used, only those three operations are performed.
///
/// In particular, a returned view may be soundly lifetime-transmuted from `View<'_, Self>`
/// to `View<'b, Self>` for any lifetime `'b` such that only those three operations are performed
/// on the source `Self` during that lifetime `'b`, and the resulting `View<'b, Self>` can be
/// soundly exposed to arbitrary (sound) code, as the view would remain valid during its entire
/// lifetime.
///
/// Extending a view to a fake lifetime like `'static` may be sound if you are careful to expose
/// that view only to code aware that the lifetime annotation is a lie; in that case, the view
/// should only be accessed when it can be proven that the view was not invalidated.
///
/// Note that the borrow checker would normally force a returned view to remain unused after the
/// source `Self` value is moved. As a result, safe code cannot directly make use of this guarantee;
/// it's most relevant for `unsafe` code. Purely safe Rust can (correctly) assume that a
/// returned `View<'_, Self>` view remains valid during its `'_` lifetime.
///
/// ### Dangers of lifetime-transmuting a view
/// Functions that take owned `Self` arguments or exclusively-borrowed `&mut Self` arguments
/// (or which can transitively access an owned `Self` or `&mut Self`), including [`Drop::drop`],
/// [`mem::drop`], and [`AliasableViewMut::view_mut`], are (in general) allowed to invalidate
/// previously-returned views of those `Self` values (or to enable safe code to later invalidate
/// previously-returned views). Some functions, such as `Box::new` (*when it does not unwind after
/// OOM*) and [`mem::forget`], may be known to only perform permitted operations (possibly only
/// under certain conditions), but be cautious.
///
/// (Note that [`mem::forget`] does deallocate the location of a `Self` value, but a sound
/// implementation of this type cannot hand out views which reference data stored inline in the
/// source `Self`; otherwise, moving a `Self` value could invalidate references in its views.
/// [`mem::forget`] could perhaps be seen as semantically moving the `Self` value to some location
/// that can never be accessed again.)
///
/// As views may have nontrivial destructors, dropping an unsafely lifetime-extended view may
/// count as a usage of that view; if a view is not known to have no drop glue, be careful not to
/// perform any operation that could invalidate a view before dropping it. In particular, drop (or
/// leak) views before dropping the `Self` source of those views.
///
/// For example, when working with panicky functions which only invalidate the `Self` source on
/// error (perhaps by dropping the `Self` value during unwinding), such as `Box::new(self)`, one
/// sound approach is to wrap views in `ManuallyDrop` before calling the panicky function and only
/// unwrap the views after the function's successful return; this ensures that views are not
/// improperly accessed in their destructors during unwinding. A leak is far preferable to UB.
///
/// ## More details for Implementors
/// To elaborate on what is meant by the prohibition against certain operations "invalidating"
/// views, it must be sound to lifetime-extend a `View<'a, Self>` view and continue using it as
/// long as operations on its source `self` value are limited to the three stated cases. It
/// suffices to ensure that (where `'a` is the varying lifetime parameter of [`Self::View`]):
/// - The pointees of pointers in the `View<'a, Self>` view which are assumed to be valid for shared
///   access during `'a`, such as `&'a T` or [`cell::Ref<'a, T>`], are not mutated (except inside
///   [`UnsafeCell`]) or otherwise exclusively accessed by moves or coercions of values of type
///   `Self` (or of formerly-`Self` values coerced to a different type).
///   (This essentially implies that the pointees cannot be stored inline in `Self`; they must
///   either be in static memory, on the heap, in some part of the stack that outlives `Self`, or
///   similar.)
/// - The pointees of pointers in the `View<'a, Self>` view which are assumed to be valid for
///   exclusive access during `'a`, such as `&'a mut T` or [`cell::RefMut<'a, T>`], are not accessed
///   by moves or coercions of values of type `Self` (or of formerly-`Self` values coerced to a
///   different type). (This again implies that the pointees cannot be stored inline in `Self`.)
/// - Exclusive access is not asserted over the pointees of pointers in the view assumed
///   to be valid for accesses during `'a` are not asserted by any of the three operations.
/// - No creative shenanigans are performed in functions that access `&Self` that cause UB when a
///   value of type `Self` is moved or coerced but old views continue to be used.
///
/// The first three requirements are a matter of pointer [provenance], and ensures that the
/// provenance of any pointers or references derived from pointers with a `'a` lifetime in a
/// `View<'a, Self>` view are not shortened, reduced, or removed when the source `Self` is moved
/// or coerced. (The Rust Abstract Machine knows nothing about the stack, heap, or static memory,
/// so they are most pedantically expressed in terms of mutation and accesses, but in practice the
/// first two requirements are about where the pointees are stored.)
/// The fourth requirement ensures that manual checks of invariants that would hold of safe Rust,
/// but not of `unsafe` code utilizing the aliasing guarantees provided by `AliasableView`, are not
/// allowed to trigger undefined behavior.
///
/// The following rough guidelines should be sufficient:
/// - Returning references to data on the heap is sound, *except* for data behind `&mut T` or
///   `Box<T>`. Those two types currently assert exclusive access over their pointees when moved.
///
///   Note that `CString` internally uses `Box<[u8]>`, but most similar `std` types (including,
///   for instance, `String`, `OsString`, and `PathBuf`) internally use `Vec<u8>`, which does
///   *not* currently have stringent aliasing requirements. It would probably be best to avoid
///   `Cow<'a, CStr>` and `CString`, though I am not certain that using them would trigger UB.
// TODO: Miri on Rust Playground doesn't see UB. That may just be because it doesn't recursively
// retag fields. This should be further explored.
/// - Avoid returning references to data that may be on the stack, except for data behind pointers
///   known to outlive `Self`.
///
///   For instance, if `Self` is similar to `&'a T` or [`AliasableRefMut<'a, T>`], then views of
///   `Self` can soundly contain references of lifetime `'a` (or other pointers guaranteed to be
///   valid for lifetime `'a`) to that `T` referenced by `Self`.
/// - Don't check the address of `&Self` to decide whether old views of `Self` should be
///   invalidated. Don't try to detect whether a by-value coercion occurred (which would also
///   move the source `Self`) to decide whether old views of `Self` should be invalidated.
///
/// ### Justification
/// The first three requirements are sufficient to imply that moving or coercing values of type
/// `Self` does not invalidate pointers that are required by this trait to remain valid.
/// In particular, the first two requirements ensure that the pointees of pointers in views are
/// not stored inline in the `Self` value; otherwise, a `Self` and its views stored in local
/// variables on the stack could be returned from a function, causing the views to reference data
/// in a deallocated stack frame. (Such a scenario would assert exclusive access over the pointees
/// and/or be considered to write uninit data to the relevant pointers' pointees; therefore, such
/// a situation is prohibited by the first two requirements.) The third requirement ensures that
/// retags introduced by moving a `&'a mut T` (and, currently, `Box<T>`, among other types) do not
/// invalidate the provenance of views.
///
/// The last remaining threat to the ability to lifetime-extend views and continue accessing them
/// (interspersed with moves, coercions, and shared or immutable access to the source `Self`), is
/// the ability for wacky `&Self` methods to detect whether the source `Self` is moved or coerced
/// and trigger pathological behavior. Note that we do not need to add more general safety
/// conditions prohibiting `&Self` methods from mutating the pointees of shared pointers,
/// accessing the pointees of exclusive pointers, or writing invalid data to their pointees; except
/// in this edge case about manual move or coercion checks, such requirements are already required
/// of all sound Rust code. If a `&Self` method were to invalidate a pointer in a view returned by
/// [`AliasableView::view`] (via mutating or accessing the pointee, or invalidating some transitive
/// invariant of the pointee) when the source `Self` had not been moved, then entirely safe code
/// could obtain a view, call the problematic `&Self` method, and trigger UB by using the old view.
/// It would, however, be sound for a problematic `&Self` method to invalidate a previous view if
/// safe Rust would not be able to access that view, which is the case after `Self` is moved.
/// Some function on the view type could also take a `&Self` argument and choose to manually
/// invalidate a view if its source `Self` was moved. Therefore, for the sake of being absolutely
/// thorough, we explicitly forbid silly edge cases like that.
///
/// Additionally, if `T: AliasableView` and `T` can be coerced to type `U`, then performing the
/// three permitted operations on values of type `U` that had been coerced from type `T` must not
/// invalidate views obtained from `<T as AliasableView>::view`, even if `U: !AliasableView`. The
/// most common way for this to occur is likely `dyn Trait` erasure, which should not be able to
/// cause any problems. It seems unlikely that any problems from coercions could occur accidentally
/// (that is, without intentionally invalidating views as discussed above).
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
/// [`cell::Ref<'a, T>`]: core::cell::Ref
/// [`cell::RefMut<'a, T>`]: core::cell::RefMut
/// [`UnsafeCell`]: core::cell::UnsafeCell
/// [`AliasableRefMut<'a, T>`]: crate::data_source::aliasable::AliasableRefMut
/// [`Self::View`]: AliasableView::View
/// [`AliasableDeref`]: https://docs.rs/aliasable_deref_trait/1.0.0/aliasable_deref_trait/trait.AliasableDeref.html
/// [`StableDeref`]: https://docs.rs/stable_deref_trait/1.2.1/stable_deref_trait/trait.StableDeref.html
/// [is unsound]: https://github.com/Storyyeller/stable_deref_trait/issues/15#issuecomment-3714995546
pub unsafe trait AliasableView<Upper: UpperBound = MaxUpperBound> {
    /// A temporary (but somewhat stable) view of the implementing type.
    type View: LendFamily<Upper>;

    /// Get a temporary (but somewhat stable) view of this type.
    ///
    /// # Guarantees for Unsafe Code
    /// The returned view can be used at a given moment so long as, starting from when the view
    /// is returned from this function up to when it is used, only the following three operations
    /// are performed on the source `Self`:
    /// - moves of the source `Self` value (which may have been coerced to a different type),
    /// - performing [coercions] on a value of type `Self` that may or may not involve moves,
    /// - any (sound) operation on a value of type `&Self` (which includes arbitrary operations on
    ///   data transitively reachable via the `&Self` value).
    ///
    /// Actions with no effect on the source value of a view, including *not* running its destructor
    /// (perhaps after moving it into `Box::leak`), are trivially permitted as no-ops. The bullet
    /// point for coercions is arguably covered by the bullet point for moves (and this note about
    /// no-ops), but it's listed for the sake of caution.
    ///
    /// This guarantee implies that some lifetime transmutes of the returned view are sound.
    /// See the [trait-level documentation] for more about how returned views may be used.
    ///
    /// [coercions]: https://doc.rust-lang.org/reference/type-coercions.html
    /// [trait-level documentation]: AliasableView#implications-for-users
    #[must_use]
    fn view(&self) -> View<'_, Self, Upper>;
}

/// A trait for types with temporary mutable views that are somewhat stable.
///
/// These mutable views can be soundly lifetime-extended (or, in the case of raw pointers, continue
/// to be soundly accessed) under specific conditions.
///
/// This trait is intended to be useful for self-referential types.
///
/// # Safety
/// Where the implementor's type is `Self`, the following operations must not invalidate any value
/// of type [`ViewMut<'a, Self>`] obtained from applying [`AliasableViewMut::view_mut`] to
/// a value in the below bullet points:
/// - moving a value of type `Self` (or a value formerly of type `Self` that was coerced to a
///   different type),
/// - performing [coercions] on a value of type `Self` that may or may not involve moves.
///
/// Actions with no effect on the source value of a view, including *not* running its destructor
/// (perhaps after moving it into `Box::leak`), are trivially permitted as no-ops. The bullet point
/// for coercions is arguably covered by the bullet point for moves (and this note about no-ops),
/// but it's listed for the sake of caution.
///
/// ## Implications for Users
/// ### Sound usage of a view
/// A returned view can be used at a given moment so long as, starting from when the view
/// was created up to when it is used, only those two operations are performed.
///
/// In particular, a returned view may be soundly lifetime-transmuted from `ViewMut<'_, Self>`
/// to `ViewMut<'b, Self>` for any lifetime `'b` such that only those two operations are performed
/// on the source `Self` during that lifetime `'b`, and the resulting `ViewMut<'b, Self>` can be
/// soundly exposed to arbitrary (sound) code, as the view would remain valid during its entire
/// lifetime.
///
/// Extending a view to a fake lifetime like `'static` may be sound if you are careful to expose
/// that view only to code aware that the lifetime annotation is a lie; in that case, the view
/// should only be accessed when it can be proven that the view was not invalidated.
///
/// Note that the borrow checker would normally force a returned view to remain unused after the
/// source `Self` value is moved. As a result, safe code cannot directly make use of this guarantee;
/// it's most relevant for `unsafe` code. Purely safe Rust can (correctly) assume that a
/// returned `ViewMut<'_, Self>` view remains valid during its `'_` lifetime.
///
/// ### Dangers of lifetime-transmuting a view
/// Running arbitrary functions on the source `Self` (whether they take owned `Self` arguments,
/// borrowed `&Self` or `&mut Self` arguments, or can transitively access a `Self` value in some
/// way) is, in general, capable of invalidating previously-returned views of that source `Self`
/// (or to enable safe code to later invalidate previously-returned views). Such problematic
/// functions include [`Drop::drop`], [`mem::drop`], [`Debug::fmt`], [`AliasableView::view`], and
/// [`AliasableViewMut::view_mut`]. Some functions, such as `Box::new` (*when it does not
/// unwind after OOM*) and [`mem::forget`], may be known to only perform permitted operations
/// (possibly only under certain conditions), but be cautious.
///
/// (Note that [`mem::forget`] does deallocate the location of a `Self` value, but a sound
/// implementation of this type cannot hand out views which reference data stored inline in the
/// source `Self`; otherwise, moving a `Self` value could invalidate references in its views.
/// [`mem::forget`] could perhaps be seen as semantically moving the `Self` value to some location
/// that can never be accessed again.)
///
/// As views may have nontrivial destructors, dropping an unsafely lifetime-extended view may
/// count as a usage of that view; if a view is not known to have no drop glue, be careful not to
/// perform any operation that could invalidate a view before dropping it. In particular, drop (or
/// leak) views before dropping the `Self` source of those views.
///
/// For example, when working with panicky functions which only invalidate the `Self` source on
/// error (perhaps by dropping the `Self` value during unwinding), such as `Box::new(self)`, one
/// sound approach is to wrap views in `ManuallyDrop` before calling the panicky function and only
/// unwrap the views after the function's successful return; this ensures that views are not
/// improperly accessed in their destructors during unwinding. A leak is far preferable to UB.
///
/// ## More details for Implementors
/// To elaborate on what is meant by the prohibition against certain operations "invalidating"
/// views, it must be sound to lifetime-extend a `ViewMut<'a, Self>` view and continue using it as
/// long as operations on its source `self` value are limited to the two stated cases. It
/// suffices to ensure that (where `'a` is the varying lifetime parameter of [`Self::ViewMut`]):
/// - the pointees of pointers in the `ViewMut<'a, Self>` view which are assumed to be valid for
///   shared access during `'a`, such as `&'a T` or [`cell::Ref<'a, T>`], are not mutated (except
///   inside [`UnsafeCell`]) or otherwise exclusively accessed by moves or coercions of type `Self`
///   (which essentially implies that the pointees cannot be stored inline in `Self`; they must
///   either be in static memory, on the heap, in some part of the stack that outlives `Self`, or
///   similar),
/// - the pointees of pointers in the `ViewMut<'a, Self>` view which are assumed to be valid for
///   exclusive access during `'a`, such as `&'a mut T` or [`cell::RefMut<'a, T>`], are not accessed
///   by moves or coercions of type `Self` (which again implies that the pointees cannot be stored
///   inline in `Self`), and
/// - exclusive access is not asserted over the pointees of pointers in the view assumed
///   to be valid for accesses during `'a` are not asserted by either of the two operations.
///
/// The three requirements are a matter of pointer [provenance], and ensure that the provenance of
/// any pointers or references derived from pointers with a `'a` lifetime in a `ViewMut<'a, Self>`
/// view are not shortened, reduced, or removed when the source `Self` is moved or coerced. (The
/// Rust Abstract Machine knows nothing about the stack, heap, or static memory, so they are most
/// pedantically expressed in terms of mutation and accesses, but in practice the first two
/// requirements are about where the pointees are stored.)
///
/// The following rough guidelines should be sufficient:
/// - Returning references to data on the heap is sound, *except* for data behind `&mut T` or
///   `Box<T>`. Those two types currently assert exclusive access over their pointees when moved.
///
///   Note that `CString` internally uses `Box<[u8]>`, but most similar `std` types (including,
///   for instance, `String`, `OsString`, and `PathBuf`) internally use `Vec<u8>`, which does
///   *not* currently have stringent aliasing requirements. It would probably be best to avoid
///   `Cow<'a, CStr>` and `CString`, though I am not certain that using them would trigger UB.
/// - Avoid returning references to data that may be on the stack, except for data behind pointers
///   known to outlive `Self`.
///
///   For instance, if `Self` is similar to `&'a T` or [`AliasableRefMut<'a, T>`], then views of
///   `Self` can soundly contain references of lifetime `'a` (or other pointers guaranteed to be
///   valid for lifetime `'a`) to that `T` referenced by `Self`.
///
/// ### Justification
/// The three requirements are sufficient to imply that moving or coercing values of type
/// `Self` does not invalidate pointers that are required by this trait to remain valid.
/// In particular, the first two requirements ensure that the pointees of pointers in views are
/// not stored inline in the `Self` value; otherwise, a `Self` and its views stored in local
/// variables on the stack could be returned from a function, causing the views to reference data
/// in a deallocated stack frame. (Such a scenario would assert exclusive access over the pointees
/// and/or be considered to write uninit data to the relevant pointers' pointees; therefore, such
/// a situation is prohibited by the first two requirements.) The third requirement ensures that
/// retags introduced by moving a `&'a mut T` (and, currently, `Box<T>`, among other types) do not
/// invalidate the provenance of views.
///
/// Additionally, if `T: AliasableViewMut` and `T` can be coerced to type `U`, then performing the
/// two permitted operations on values of type `U` that had been coerced from type `T` must not
/// invalidate views obtained from `<T as AliasableViewMut>::view_mut`, even if
/// `U: !AliasableViewMut`. As coercions and moves do not execute arbitrary code, this constraint
/// should not add anything on top of the other constraints placed on `T: AliasableViewMut`.
///
/// [coercions]: https://doc.rust-lang.org/reference/type-coercions.html
/// [provenance]: https://doc.rust-lang.org/std/ptr/index.html#provenance
/// [`mem::drop`]: core::mem::drop
/// [`mem::forget`]: core::mem::forget
/// [`ManuallyDrop::new`]: core::mem::ManuallyDrop::new
/// [`Deref`]: core::ops::Deref
/// [`cell::Ref<'a, T>`]: core::cell::Ref
/// [`cell::RefMut<'a, T>`]: core::cell::RefMut
/// [`UnsafeCell`]: core::cell::UnsafeCell
/// [`AliasableRefMut<'a, T>`]: crate::data_source::aliasable::AliasableRefMut
/// [`Self::ViewMut`]: AliasableViewMut::ViewMut
pub unsafe trait AliasableViewMut<Upper: UpperBound = MaxUpperBound>: AliasableView<Upper> {
    /// A temporary (but somewhat stable) mutable view of the implementing type.
    type ViewMut: LendFamily<Upper>;

    /// Get a temporary (but somewhat stable) mutable view of this type.
    ///
    /// # Guarantees for Unsafe Code
    /// The returned view can be used at a given moment so long as, starting from when the view
    /// is returned from this function up to when it is used, only the following two operations
    /// are performed on the source `Self`:
    /// - moves of the source `Self` value (which may have been coerced to a different type),
    /// - performing [coercions] on a value of type `Self` that may or may not involve moves.
    ///
    /// Actions with no effect on the source value of a view, including *not* running its destructor
    /// (perhaps after moving it into `Box::leak`), are trivially permitted as no-ops. The bullet
    /// point for coercions is arguably covered by the bullet point for moves (and this note about
    /// no-ops), but it's listed for the sake of caution.
    ///
    /// This guarantee implies that some lifetime transmutes of the returned view are sound.
    /// See the [trait-level documentation] for more about how returned views may be used.
    ///
    /// [coercions]: https://doc.rust-lang.org/reference/type-coercions.html
    /// [trait-level documentation]: AliasableViewMut#implications-for-users
    #[must_use]
    fn view_mut(&mut self) -> ViewMut<'_, Self, Upper>;
}

/// Extend the conditions under which temporary views of this type may be soundly lifetime-extended
/// (or, in the case of raw pointers, continue to be soundly accessed).
///
/// This trait is intended to be useful for self-referential types, and it is generally intended
/// to be implemented for types that are reference-counted *or* provide owned "views" that are
/// never invalidated when the source `Self` is dropped.
///
/// # Safety
/// Where the implementor's type is `Self`, in addition to the operations prohibited by
/// [`AliasableView`] from invalidating any value of type [`View<'a, Self>`] obtained
/// from applying [`AliasableView::view`] to a source `Self` value, the following operation on a
/// `Self` value must not invalidate its views obtained via [`AliasableView::view`]:
/// - Dropping (that is, running the destructor of) the value of `Self` when at least one other
///   sibling clone of that `Self` value has not been dropped.
///
/// If `Self` also implements [`AliasableViewMut`], then in addition to the operations prohibited
/// by [`AliasableViewMut`] from invalidating any value of type [`ViewMut<'a, Self>`] obtained
/// from applying [`AliasableViewMut::view_mut`] to a source `Self` value, the following operation
/// on a `Self` value must not invalidate its views obtained via [`AliasableViewMut::view_mut`]:
/// - Dropping (that is, running the destructor of) the value of `Self` when at least one other
///   sibling clone of that `Self` value has not been dropped.
///
/// A "sibling clone" of a `val` value here means any `sibling` value which satisfies one or
/// more of the following:
/// - `sibling` is a clone of `val` (that is, `sibling` was constructed via [`Clone::clone`]
///   or [`Clone::clone_from`] applied to a reference to `val`),
/// - `val` is a clone of `sibling`,
/// - `val` is a sibling of a sibling of `sibling`.
///
/// Note in particular that if some `sibling` was forgotten via [`mem::forget`], then even though
/// the location of the `sibling` itself may be deallocated or otherwise invalidated, it does
/// count as a sibling clone which has not and will never be dropped (unless, for example, that
/// `sibling` is unsafely recovered from some raw form and is later dropped).
///
/// ## `AliasableClone + AliasableViewMut`
/// Note that mutating one sibling clone is not permitted to invalidate views of other siblings
/// (as that would be unsound, even for entirely safe Rust not making use of the lifetime
/// transmutes that this trait guarantees are sound). Therefore, types that implement both
/// [`AliasableClone`] and [`AliasableViewMut`] presumably provide views in a trivial way.
/// (Perhaps the "mutable views" only contain shared / immutable references, are obtained from
/// `Box::leak`, or are simply plain-old `Copy + 'static` data, for example). Nevertheless,
/// implementing both traits can be sound.
///
/// [`mem::forget`]: core::mem::forget
pub unsafe trait AliasableClone: AliasableView + Clone {}

/// A trait for types with a canonical aliasable version (which at least implements
/// [`AliasableView`]).
///
/// This trait is intended to improve ergonomics for types similar to `Box<T>` or `&mut T`
/// in circumstances where an aliasable type is needed.
///
/// For a type which is *already* aliasable, the canonical aliasable version is itself.
pub trait IntoAliasable<Upper: UpperBound = MaxUpperBound> {
    /// The canonical aliasable version of this type.
    type IntoAliasable: AliasableView<Upper>;

    /// Get the canonical aliasable version of this type.
    ///
    /// This conversion should be cheap.
    #[must_use]
    fn into_aliasable(self) -> Self::IntoAliasable;
}

impl<T: AliasableView<Upper>, Upper: UpperBound> IntoAliasable<Upper> for T {
    type IntoAliasable = Self;

    #[inline]
    fn into_aliasable(self) -> Self::IntoAliasable {
        self
    }
}

/// A trait for types with a canonical aliasable version (which at least implements
/// [`AliasableView`] and [`AliasableView`]).
///
/// This trait is intended to improve ergonomics for types similar to `Box<T>` or `&mut T`
/// in circumstances where an aliasable type is needed.
///
/// For a type which is *already* aliasable, the canonical aliasable version is itself.
pub trait IntoAliasableMut<Upper: UpperBound = MaxUpperBound>:
    IntoAliasable<Upper, IntoAliasable: AliasableViewMut<Upper>>
{}

impl<T, Upper> IntoAliasableMut<Upper> for T
where
    T: IntoAliasable<Upper, IntoAliasable: AliasableViewMut<Upper>>,
    Upper: UpperBound,
{}
