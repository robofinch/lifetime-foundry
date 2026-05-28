//! A [`recursive_view`] macro for reducing the safety requirements of this crate's three
//! `unsafe` traits to a simpler list of requirements, applicable to common sorts of
//! [`RecursiveViewKind`] implementations.
//!
//! [`recursive_view`]: crate::recursive_view
//! [`RecursiveViewKind`]: crate::view_kinds::RecursiveViewKind

#![expect(
    unsafe_code,
    reason = "create a simpler set of `unsafe` requirements for implementing other unsafe traits",
)]


/// Utility for implementing [`StableView`], [`StableViewMut`], and [`StableClone`]
/// for [`RecursiveViewKind`] in cases similar to `Option<T>`, `Box<T>`, `[T; N]`, or
/// `(T1, .., Tn)`.
///
/// Optionally, [`DefaultViewKind`] is set to use [`RecursiveViewKind`] for the data type.
///
/// # Motivation
/// The intent is to implement [`StableView`], [`StableViewMut`], and [`StableClone`] by
/// simply deferring to generic type parameters' implementations of those traits, such that
/// [`CustomView<'_, '_, '_, Self, RecursiveViewKind<(V1, .., Vn)>`] is
/// `SelfWithoutParams<.., CustomView<'_, '_, '_, T1, V1>, .., CustomView<'_, '_, '_, Tn, Vn>>`
/// (and likewise for [`CustomViewMut`]).
///
/// # Robust Guarantee about Conceptual Pools
/// Suppose `Data = SelfWithoutParams<.., T1, .., Tn>`.
///
/// When `Data: Clone` and each `Vi: StableClone<'_, '_, Ti>` (in addition to any other
/// where-bounds provided by you), this macro implements `StableClone<'_, '_, Data>` for
/// `RecursiveViewKind<(V1, .., Vn)>`, and sets the definition of the conceptual pool associated
/// with `RecursiveViewKind<(V1, .., Vn)>` and a `data: Data` value as the set of all values of
/// some `SelfWithoutParams<.., U1, .., Un>` type (or any other type reachable via coercions of
/// `SelfWithoutParams<.., U1, .., Un>` values) whose view components' set of conceptual pools is
/// equal to the set of conceptual pools of `data`'s view components.
/// (See below for what "view components" are.)
///
/// Put more simply: the conceptual pool is the intersection of `data`'s view components'
/// conceptual pools. Two `Data` values are in the same conceptual pool if their view components
/// are in the same conceptual pools (in any order, with any multiplicity). This definition ensures
/// that `data` is in one (and exactly one) conceptual pool at any given time. A view's associated
/// pool is the (fixed) conceptual pool which its source `Data` value was in at the time of its
/// creation.
///
/// This definition of conceptual pool is also referenced by [`RecursiveViewKind`] as the
/// *standard* definition of conceptual pool for that view kind. Users of this macro *robustly*
/// guarantee that this standard definition is used, such that `unsafe` code can rely on it.
///
/// # Terminology
/// Where `Self` is the implementing type, let `SelfWithoutParams` be the type path of `Self`,
/// such that `Self` is something like `SelfWithoutParams<'static, Params, Here, .., T1, .., Tn>`.
///
/// Let some tail of `Self`'s generic parameters `T1, .., Tn` be considered
/// "view component parameters", and define "view components" of `Self` as the values of type
/// `T1`, .., or `Tn` stored in `Self`. (As indicated by the notation, `T1, .., Tn` are required
/// to be generic type parameters, not const generics or generic lifetime parameters.)
///
/// ## Notes
/// Any part of `Self` which is not considered a view component must not **ever** contain a
/// non-`'static` reference. Remember that generic type parameters can usually refer to types that
/// contain references.
///
/// However, note (for example) that `Foo<'a, B, T1>` can still use this macro while having
/// `'a` and `B` *not* be involved with the view components; for example, "`Self`" could be
/// `Foo<'static, B, T1>` with a `B: 'static` bound included in the impl.
///
/// Next, `T1, .., Tn` must be the last generics of the type. If necessary, you can use a type
/// alias to change the order of the generics of the implementing type (and possibly substitute
/// in `'static` data for some generics to avoid repetition in the impl), such as:
/// ```ignore
/// type ConvenientOrdering<const FOO: u8, V, T1, T2> = path::to::Type<'static, T1, FOO, T2, V>;
/// ```
///
/// # Safety
/// You will need to read the `Syntax` section for full context.
///
/// - All view components in `SelfWithoutParams<.., T1, .., Tn>` must be values of type `T1`, ..,
///   or `Tn` values stored with no internally mutable wrappers around them within `Self`.
///
///   Wrapping a view component in `UnsafeCell` *could* be sound so long as you never actually
///   *use* internal mutability on that view component, but it is simpler to just forbid this case
///   entirely for the purposes of this macro. Wrapping an *entire* `Self` in an internally mutable
///   type is fine, thus the final "within `Self`" qualifier. (If `Self` has a public safe
///   constructor, then the caller of this macro cannot prevent someone from creating a
///   `Mutex<Self>` anyway, for example.)
/// - Cloning a value of type `SelfWithoutParams<.., T1, .., Tn>` must result in a new value whose
///   view components are the clones of the source value's view components. "cloning" (and "clones")
///   refers to the application of (and values produced from) [`Clone::clone`] and
///   [`Clone::clone_from`]. To be clear, each source view component **must** have at least one
///   clone in the new value, and each view component in the new value must be a clone of some
///   view component in the source value.
/// - Any view components in the `SelfWithoutParams<.., T1, .., Tn>` value returned by your `map` or
///   `map_mut` implementation must be values returned by some `map_i` applied to a view
///   component of `this` of type `Ti`. (This safety condition doesn't forbid you from naming
///   the `map_i` functions whatever you want in `Variadics`, there just isn't a more
///   clear way to refer to "`map_i`" here.)
///
/// # Syntax
/// Pretend that there exists some trait as follows (noting that a similar trait was actually
/// the prototype for this macro):
///
/// ```ignore
/// /// *More-or-less this macro's documentation.*
/// ///
/// /// # Safety
/// /// *More-or-less this macro's safety requirements,* but with the additional constraint that
/// /// `Self::WithParams<U1, .., Un>` must be `SelfWithoutParams<.., U1, .., Un>` and that
/// /// `T1, .., Tn` are the view component parameters of `Self`.
/// unsafe trait MapView<
///     'a, 'stable, 'data,
///     __ImpliedBounds = &'a &'stable &'data (),
/// > {
///     type T1: ?Sized;
///     ..;
///     type Tn: ?Sized;
///
///     type WithParams<U1, .., Un>;
///     // Must implement `CovariantFamily<'a, 'data>` such that
///     // `Varying<'stable, 'a, 'data, Self::WithParamsFamily<U1, ..>>`
///     // is `Self::WithParams<Varying<'stable, 'a, 'data, U1>, ..>`.
///     // Else, the code fails to compile.
///     type WithParamsFamily<U1, .., Un>: /* complicated trait bound */;
///
///     const SET_DEFAULT_TO_RECURSIVE_VIEW_KIND: bool;
///
///     /// Apply some mapping to `this`'s view components, obtaining a new value with the mapped
///     /// view components.
///     ///
///     /// See the trait-level documentation (or, macro-level as it were)
///     /// for implementation safety requirements.
///     fn map<'a, M1, .., Mn, U1, .., Un>(
///         this: &'a Self,
///         map_1: M1,
///         ..,
///         map_n: Mn,
///     ) -> Self::WithParams<U1, .., Un>
///     where
///         M1: Fn(&'a Self::T1) -> U1,
///         ..,
///         Mn: Fn(&'a Self::Tn) -> Un,
///         Self::T1: 'a,
///         ..,
///         Self::Tn: 'a;
///
///     /// Apply some mapping to `this`'s view components, obtaining a new value with the mapped
///     /// view components.
///     ///
///     /// See the trait-level documentation (or, macro-level as it were)
///     /// for implementation safety requirements.
///     fn map_mut<'a, M1, .., Mn, U1, .., Un>(
///         this: &'a mut Self,
///         map_1: M1,
///         ..,
///         map_n: Mn,
///     ) -> Self::WithParams<U1, .., Tn>
///     where
///         M1: Fn(&'a mut Self::T1) -> U1,
///         ..,
///         Mn: Fn(&'a mut Self::Tn) -> Un,
///         Self::T1: 'a,
///         ..,
///         Self::Tn: 'a;
/// }
/// ```
///
/// Your responsibility is to write an `impl` of this trait for your type, with some abbreviated
/// syntax.
///
/// ## Example
///
/// ```
/// use stable_view::recursive_view;
/// mod path { mod to { mod your {
///     struct Type<'a, const FOO: u8, V, T1, T2: ?Sized + Debug>(&'a u8, T1, V, T2);
/// }}}
///
/// recursive_view! {
///     // Information used to automatically fill in various parts of the "trait implementation",
///     // such as signatures, function parameters, where-bounds, and so on.
///     // Those components are denoted as `..` or `_` in the rest of the syntax.
///     Variadics = [
///         // The first parameter is used to define `Self` as `SelfWithoutParams<.., T1, .., Tn>`.
///         //
///         // The second parameter is used as the name of a view parameter for
///         // `RecursiveViewKind<(V1, .., Vn)>`. They *are* in scope for for your where-bounds,
///         // `map`, and `map_mut` implementations, though you can probably just ignore them.
///         //
///         // The third parameter is the name of the corresponding `map_i` mapping function,
///         // which is in-scope for your `map` or `map_mut` implementation.
///         // Equivalents of `U1, .., Un` and `M1, .., Mn` are not exposed.
///         (T1, V1, map_1),
///
///         // After each `Vi` parameter, you can optionally include where-bounds on
///         // `<Vi as StableView<'a, 'stable, 'data, Ti>>::{View, ViewMut}`.
///         //
///         // The lifetime parameters are (intentionally) not exposed to you, and you cannot access
///         // them due to macro hygiene. Therefore, this macro must expose these separate
///         // means to bound those types.
///         // Note that all `View`s and `ViewMut`s implement `Sized`.
///         (T2, V2 (View: Debug) (ViewMut: Debug), map_2),
///     ];
///
///     // Either `Default = true;` or `Default = false;`. Determines whether
///     // `SetDefaultView` and `SetDefaultViewMut` are implemented (to choose `DefaultViewKind`).
///     Default = true;
///
///     // `Upper: UpperBound, T1, .., Tn` impl parameters are included automatically.
///     // To place bounds on any of the `Ti` parameters, use where-bounds.
///     // `Ti: ?Sized` bounds are *not* included by default.
///     //
///     // Remember, if the type contains values of type `V` which are not considered view
///     // components, then you **must** ensure that `V: 'static` so that non-view components
///     // do not have non-`'static` references.
///     unsafe impl<.., {const FOO: u8, V: 'static + Clone}> MapView<..>
///     // Start by listing any type parameters not among `T1, .., Tn`. Those last parameters
///     // are included for you. Each parameter goes in its own set of braces.
///     for path::to::your::Type<{'static}, {FOO}, {V}, ..>
///     // `where {...}` is optional.
///     where {
///         // Any additional where-bounds for the "trait impl" of `MapView` must go here.
///         T2: ?Sized + Debug,
///     }
///     {
///         // The `T1, .., Tn` and `WithParams` associated types are not included, since the
///         // information you provide above is sufficient to define them.
///         // You can optionally provide `WithParamsFamily`; else, it defaults to the same
///         // as `WithParams`, which is equivalent to this:
///         type WithParamsFamily<..> = path::to::your::Type<{'static}, {FOO}, {V}, ..>;
///
///         fn map<..>(this: &Self, ..) -> _ where .. {
///             // You fill in this implementation for your type.
///             // Warning: due to how this is desugared, early `return`s will not work
///             // (except in edge cases). Use the following trick instead, if needed:
///             'early_return: {
///                 // Example:
///                 break 'early_return path::to::your::Type(
///                     this.0,
///                     map_1(&this.1),
///                     this.2.clone(),
///                     map_2(&this.3),
///                 );
///             }
///         }
///
///         fn map_mut<..>(this: &mut Self, ..) -> _ where .. {
///             // You fill in this implementation for your type.
///
///             // Example:
///             path::to::your::Type(this.0, map_1(&mut this.1), this.2.clone(), map_2(&mut this.3))
///         }
///     }
/// }
/// ```
///
/// [`StableView`]: crate::traits::StableView
/// [`StableViewMut`]: crate::traits::StableViewMut
/// [`StableClone`]: crate::traits::StableClone
/// [`CustomView<'_, '_, '_, Self, RecursiveViewKind<(V1, .., Vn)>`]: crate::traits::CustomView
/// [`CustomViewMut`]: crate::traits::CustomViewMut
/// [`DefaultViewKind`]: crate::view_kinds::DefaultViewKind
/// [`RecursiveViewKind`]: crate::view_kinds::RecursiveViewKind
#[macro_export]
macro_rules! recursive_view {
    // NOTE: `view_bounds`, `view_mut_bounds`, `impl_params`, `generics`, `where_bounds`,
    // `map_impl`, and `map_mut_impl` can contain arbitrary Rust code. We need to prevent
    // code injection from causing unexpected effects in this macro.
    {
        Variadics = [
            $((
                $t:ident,
                $v:ident $((View: $($view_bounds:tt)*) (ViewMut: $($view_mut_bounds:tt)*))?,
                $map:ident $(,)?
            )),* $(,)?
        ];

        Default = $set_default_view_kind:ident;

        unsafe impl<..$(, {$($impl_params:tt)*})?> MapView<..>
        for $($name:ident)::+<$({$($generics:tt)*},)* ..>
        $(where {$($where_bounds:tt)*})?
        {
            $(type WithParamsFamily<..> = $($fam_name:ident)::+<$({$($fam_generics:tt)*},)* ..>;)?

            fn map<..>($this_ref:ident: &Self, ..) -> _ where .. {
                $($map_impl:tt)*
            }

            fn map_mut<..>($this_mut:ident: &mut Self, ..) -> _ where .. {
                $($map_mut_impl:tt)*
            }
        }
    } => {
        // SAFETY: Asserted by user of this macro.
        const _: () = unsafe { $crate::__macro::unsafe_recursive_view() };

        #[expect(
            unused_lifetimes,
            reason = "if a bound is unsatisfiable, the for<'maybe_unsat> lifetime binder \
                      means that the trait will simply never be implemented, \
                      instead of the impossible bound potentially causing a compilation error",
        )]
        #[expect(
            unsafe_code,
            reason = "lint is moved to `unsafe_recursive_view` for a clearer error message",
        )]
        const _: () = {
            // SAFETY:
            // Moves, coercions, or immutable operations (in any quantity and order) on `data`
            // cannot invalidate any `'static` data, so we need only care about the potentially
            // non-`'static` data in `data.view()`, which come solely from view components produced
            // from calling `.view()` on view components of `data`.
            // (This fact is a mixture of reasoning about the below impl and about the safety
            // requirements of this macro.)
            // Moves (or coercions, or immutable operations on) `data` (in any quantity and order)
            // can move (or coerce, or perform immutable operations on) its view components,
            // but since the view kinds are here required to implement `StableView` for those
            // view components (and since code injection is prevented), that does not invalidate
            // their views.
            // (Note that we use the fact that performing immutable operations on `data` cannot
            // perform mutable operations on its view components; this is why we forbade internal
            // mutability.)
            //
            // Thus, moves, coercions, and immutable operations (in any quantity and order) on
            // `data` do not invalidate the value returned by `data.view()`, and since we
            // also enforce the `'data` upper bound, this implementation is sound.
            unsafe impl<
                'a, 'data,
                $(
                    $t,
                    $v: $crate::StableView<'a, 'data, $t, $(View: $($view_bounds)*)?>,
                )*
                $($($impl_params)*)?
            > $crate::StableView<'a, 'data, $($name)::+<$($($generics)*,)* $($t),*>>
            for $crate::RecursiveViewKind<($($v,)*)>
            where
                $($($where_bounds)*)?
            {
                type View = $crate::__emit_if_nonempty!(if {$($($fam_name)+)?} {
                    $($($fam_name)::+)?<
                        $($($($fam_generics)*,)*)?
                        $(<$v as $crate::StableView<'a, 'data, $t>>::View),*
                    >
                } else {
                    $($name)::+<
                        $($($generics)*,)*
                        $(<$v as $crate::StableView<'a, 'data, $t>>::View),*
                    >
                });

                #[inline]
                unsafe fn view<'stable>(
                    data: &'a $($name)::+<$($($generics)*,)* $($t),*>,
                ) -> $crate::CustomView<
                    'a, 'stable, 'data,
                    $($name)::+<$($($generics)*,)* $($t),*>, Self,
                >
                where
                    'data: 'stable,
                    // We define `'a` before expanding untrusted `tt` tokens, so by macro hygiene,
                    // this mention of `'a` prevents any of them from switching out this unsafe
                    // impl.
                    'stable: 'a
                {
                    let $this_ref = data;
                    $(
                        // SAFETY: `'stable = 'a` is guaranteed to be sound.
                        let $map = |view_component| unsafe {
                            <$v as $crate::StableView<'a, 'data, $t>>::view::<'a>(
                                view_component,
                            )
                        };
                    )*

                    // Rust has paired `{` and `}` delimiters, so `map_impl` can't do anything
                    // too crazy.
                    let stable_eq_a: $crate::CustomView<
                        'a, 'a, 'data,
                        $($name)::+<$($($generics)*,)* $($t),*>, Self,
                    > = {
                        $($map_impl)*
                    };

                    #[allow(
                        clippy::useless_transmute,
                        reason = "if `'stable` is unused, this is a no-op",
                    )]
                    // SAFETY: See the "`transmute` in `view` Implementation" section of the
                    // `StableView` docs.
                    unsafe {
                        ::core::mem::transmute::<
                            $crate::CustomView<
                                'a, 'a, 'data,
                                $($name)::+<$($($generics)*,)* $($t),*>, Self,
                            >,
                            $crate::CustomView<
                                'a, 'stable, 'data,
                                $($name)::+<$($($generics)*,)* $($t),*>, Self,
                            >,
                        >(stable_eq_a)
                    }
                }
            }

            // SAFETY:
            // Moves, non-deref coercions (in any quantity and order), or no-ops of `data` cannot
            // invalidate any `'static` data, so we need only care about the potentially
            // non-`'static` data in `data.view_mut()`, which come solely from view components
            // produced from calling `.view_mut()` on view components of `data`.
            // (This fact is a mixture of reasoning about the below impl and about the safety
            // requirements of this macro.)
            // Moving (or coercing, or performing no-ops on) `data` (in any quantity and order) can
            // move (or coerce, or perform no-ops on) its view components, but since they are here
            // required to implement `StableViewMut` (and since code injection is prevented), that
            // does not invalidate their views.
            //
            // Thus, moving, coercing, or performing no-ops on `data` (in any quantity and order)
            // does not invalidate the value returned by `data.view_mut()`, and since we
            // also enforce the `'data` upper bound, this implementation is sound.
            unsafe impl<
                'a, 'data,
                $(
                    $t,
                    $v: $crate::StableViewMut<
                        'a, 'data, $t,
                        $(View: $($view_bounds)*, ViewMut: $($view_bounds)*)?
                    >,
                )*
                $($($impl_params)*)?
            > $crate::StableViewMut<
                'a, 'data,
                $($name)::+<$($($generics)*,)* $($t),*>,
            >
            for $crate::RecursiveViewKind<($($v,)*)>
            where
                $($($where_bounds)*)?
            {
                type ViewMut = $crate::__emit_if_nonempty!(if {$($($fam_name)+)?} {
                    $($($fam_name)::+)?<
                        $($($($fam_generics)*,)*)?
                        $(<$v as $crate::StableViewMut<'a, 'data, $t>>::ViewMut),*
                    >
                } else {
                    $($name)::+<
                        $($($generics)*,)*
                        $(<$v as $crate::StableViewMut<'a, 'data, $t>>::ViewMut),*
                    >
                });

                #[inline]
                unsafe fn view_mut<'stable>(
                    data: &'a mut $($name)::+<$($($generics)*,)* $($t),*>,
                ) -> $crate::CustomViewMut<
                    'a, 'stable, 'data,
                    $($name)::+<$($($generics)*,)* $($t),*>, Self,
                >
                where
                    'data: 'stable,
                    // We define `'a` before expanding untrusted `tt` tokens, so by macro hygiene,
                    // this mention of `'a` prevents any of them from switching out this unsafe
                    // impl.
                    'stable: 'a
                {
                    let $this_mut = data;
                    $(
                        // SAFETY: `'stable = 'a` is guaranteed to be sound.
                        let $map = |view_component| unsafe {
                            <$v as $crate::StableViewMut<'a, 'data, $t>>::view_mut::<'a>(
                                view_component,
                            )
                        };
                    )*

                    // Rust has paired `{` and `}` delimiters, so `map_mut_impl` can't do anything
                    // too crazy.
                    let stable_eq_a: $crate::CustomViewMut<
                        'a, 'a, 'data,
                        $($name)::+<$($($generics)*,)* $($t),*>, Self,
                    > = {
                        $($map_mut_impl)*
                    };

                    #[allow(
                        clippy::useless_transmute,
                        reason = "if `'stable` is unused, this is a no-op",
                    )]
                    // SAFETY: See the "`transmute` in `view_mut` Implementation" section of the
                    // `StableViewMut` docs.
                    unsafe {
                        ::core::mem::transmute::<
                            $crate::CustomViewMut<
                                'a, 'a, 'data,
                                $($name)::+<$($($generics)*,)* $($t),*>, Self,
                            >,
                            $crate::CustomViewMut<
                                'a, 'stable, 'data,
                                $($name)::+<$($($generics)*,)* $($t),*>, Self,
                            >,
                        >(stable_eq_a)
                    }
                }
            }

            /// # Robust Guarantee
            /// The conceptual pool definition given by [`stable_view::recursive_view`] is used.
            ///
            /// [`stable_view::recursive_view`]: https://docs.rs/stable-view/0/stable_view/macro.recursive_view.html
            #[allow(single_use_lifetimes, reason = "it's used once iff `*` repeats zero times")]
            // SAFETY:
            // Let `Data` refer to `$($name)::+<$($($generics)*,)* $($t),*>`, a.k.a.
            // `SelfWithoutParams<.., T1, .., Tn>`.
            // We can define the conceptual pool associated with a `data: Data` value as the set of
            // all values of some `SelfWithoutParams<.., U1, .., Un>` type (or any other type
            // reachable via coercions of `SelfWithoutParams<.., U1, .., Un>` values) whose view
            // components' set of conceptual pools is equal to the set of conceptual pools of
            // `data`'s view components.
            //
            // ...That definition is long. Put more simply: we're basically taking the intersection
            // of `data`'s view components' conceptual pools. Two `Data` values are in the same
            // conceptual pool if their view components are in the same conceptual pools (in
            // any order, with any multiplicity). This definition ensures that `data` is in one
            // (and exactly one) conceptual pool at any given time. A view's associated pool is
            // the (fixed) conceptual pool which its source `Data` value was in at the time of its
            // creation.
            //
            // Requirement 1:
            // The caller of this macro guarantees that cloning `data` results in a new `Data`
            // value whose view components are clones of `data`'s view components and which contain
            // a clone of each of `data`'s view components. Therefore, the set of conceptual
            // `StableClone` pools of the view components of `data` is the same as the set
            // of conceptual pools of the view components of the clone of `data`. By definition,
            // the clone is in the same pool as `data`, satisfying requirement 1.
            //
            // Requirement 2:
            // Only the view components of `data` determine which pool it's in, so we need only
            // care about the impact of moves, coercions, and immutable operations on `data`'s
            // view components. Moves (or coercions, or immutable operations on) `data`
            // (in any quantity and order) can move (or coerce, or perform immutable operations on)
            // its view components, and since the view kinds are here required to implement
            // `StableClone` for the view components (and since code injection is prevented), that
            // does not change which pools they are in.
            // (Note that we use the fact that performing immutable operations on `data` cannot
            // perform mutable operations on its view components; this is why we forbade internal
            // mutability.)
            //
            // Thus, moves, coercions, and immutable operations (in any quantity and order) on
            // `data` do not change which pool `data` is in.
            //
            // Requirement 3:
            // If the conceptual pool associated with a view returned by `data.view()` is nonempty,
            // then there exists some value of type `Data` (or formerly of type `Data` before
            // coercions) whose view components are in the same pools as the view components
            // which were in `data` at the time the view was made. Each of the view
            // components in `data.view()` came from view components in `data` at the time the view
            // was made, so the associated pools of the view components of `data.view()` are all
            // therefore nonempty. Since the `'static` data in the view cannot be invalidated
            // and the only non-`'static` data is in its view components, we thus have that the
            // view has not been invalidated.
            unsafe impl<
                'a, 'data,
                $(
                    $t: ::core::clone::Clone,
                    $v: $crate::StableClone<'a, 'data, $t, $(View: $($view_bounds)*)?>,
                )*
                $($($impl_params)*)?
            > $crate::StableClone<'a, 'data, $($name)::+<$($($generics)*,)* $($t),*>>
            for $crate::RecursiveViewKind<($($v,)*)>
            where
                for<'maybe_unsat> $($name)::+<$($($generics)*,)* $($t),*>: ::core::clone::Clone,
                // We define `'a` before expanding untrusted `tt` tokens, so by macro hygiene,
                // this mention of `'a` prevents any of them from switching out this unsafe
                // impl. There is still `where_bounds` below, but those bounds can't do any damage
                // to the soundness of this code.
                &'a ():,
                $($($where_bounds)*)?
            {}

            // This isn't an `unsafe impl`, so `tt` code injection does not matter.
            $crate::__maybe_emit!(if $set_default_view_kind {
                #[allow(single_use_lifetimes, reason = "it's used once iff `*` repeats zero times")]
                impl<
                    'a, 'data: 'a,
                    $(
                        $t,
                    )*
                    $($($impl_params)*)?
                > $crate::SetDefaultView<'a, 'data>
                for $($name)::+<$($($generics)*,)* $($t),*>
                where
                    $(
                        $crate::DefaultViewKind: $crate::StableView<
                            'a, 'data, $t,
                            $(View: $($view_bounds)*)?
                        >,
                    )*
                    $($($where_bounds)*)?
                {
                    type Default = $crate::RecursiveViewKind<($(
                        $crate::__replace!({$v}, {$crate::DefaultViewKind}),
                    )*)>;
                }
            });
        };
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __maybe_emit {
    (if true { $($tokens:tt)* }) => {
        $($tokens)*
    };

    (if false { $($tokens:tt)* }) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __emit_if_nonempty {
    (if {$($nonempty:tt)+} {$($then:tt)*} else {$($else:tt)*}) => {
        $($then)*
    };

    (if {} {$($then:tt)*} else {$($else:tt)*}) => {
        $($else)*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __replace {
    ({$($tokens:tt)*}, {$($with:tt)*} $(,)?) => {
        $($with)*
    };
}

/// Denotes that [`recursive_view`] was used.
///
/// Used to trigger the `unsafe_code` lint.
///
/// # Safety
/// This function itself has no safety preconditions, but though the safety requirements for
/// using [`recursive_view`] must be upheld.
///
/// [`recursive_view`]: crate::recursive_view
pub const unsafe fn unsafe_recursive_view() {}
