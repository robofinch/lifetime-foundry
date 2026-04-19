#![expect(
    unsafe_code,
    reason = "create a simpler set of `unsafe` requirements for implementing other unsafe traits",
)]

/// Utility for implementing [`AliasableView`], [`AliasableViewMut`], and [`AliasableClone`]
/// in cases similar to `Option<T>`, `[T; N]`, or `(T1, .., Tn)`.
///
/// # Motivation
/// The intent is to implement [`AliasableView`], [`AliasableViewMut`], and [`AliasableClone`] by
/// simply deferring to generic type parameters' implementations of those traits, such that
/// [`View<'_, Self, _>`] is
/// `SelfWithoutParams<.., View<'_, T1, _>, .., View<'_, Tn, _>>` (and likewise for [`ViewMut`]).
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
/// # Correctness
/// Whenever `T1, .., Tn` all implement `LendFamily<Upper>` for some `Upper: UpperBound`,
/// it must hold that `SelfWithoutParams<.., T1, .., Tn>: LendFamily<Upper>`.
/// Otherwise, your invocation of this macro will fail to compile.
///
/// # Safety
/// You will need to read the `Syntax` section for full context.
///
/// - All view components in `SelfWithoutParams<.., T1, .., Tn>` must be values of type `T1`, ..,
///   or `Tn` values stored *inline* (that is, within the allocation of `Self`, rather than in a
///   separate allocation) with no internally mutable wrappers around them.
///
///   The "inline" requirement does not forbid something like
///   `struct Foo<T>(Option<Result<T, ()>>)`; the `T` value is still in the same allocation, even
///   if the relevant field isn't syntactically inline in the definition of `Foo`.
///
///   (Wrapping a view component in `UnsafeCell` *could* be sound so long as you never actually
///   *use* internal mutability on that view component, but it is simpler to just forbid this case
///   entirely for the purposes of this macro.)
/// - Cloning a value of type `SelfWithoutParams<.., T1, .., Tn>` must result in a new value whose
///   view components are the clones of the source value's view components. "cloning" (and "clones")
///   refers to the application of (and values produced from) [`Clone::clone`] and
///   [`Clone::clone_from`]. To be clear, each source view component **must** have at least one
///   clone in the new value, and each view component in the new value must be a clone of some
///   view component in the source value.
/// - Any view components in the `SelfWithoutParams<.., T1, .., Tn>` value returned by your `map` or
///   `map_mut` implementation must be values returned by some `map_i` applied to a view
///   component of `self` of type `Ti`. (This safety condition doesn't forbid you from naming
///   the `map_i` functions whatever you want in `Variadics`, there just isn't a more
///   clear way to refer to "`map_i`" here.)
///
/// # Syntax
/// Pretend that there exists some trait as follows (noting that a similar trait was actually
/// the prototype for this macro):
///
/// ```ignore
/// /// *More-or-less this macro's documentation*
/// /// # Safety
/// /// *More-or-less this macro's safety requirements*, but with the additional constraint that
/// /// `Self::WithParams<U1, .., Un>` must be `SelfWithoutParams<.., U1, .., Un>` and that
/// /// `T1, .., Tn` are the view component parameters of `Self`.
/// unsafe trait MapAliasable<Upper: UpperBound> {
///     type T1: ?Sized;
///     ..;
///     type Tn: ?Sized;
///
///     type WithParams<
///         U1: LendFamily<Upper>,
///         ..,
///         Un: LendFamily<Upper>,
///     >: LendFamily<Upper>;
///
///     /// Apply some mapping to `self`'s view components, obtaining a new value with the mapped
///     /// view components.
///     ///
///     /// See the trait-level documentation (or, macro-level as it were)
///     /// for implementation safety requirements.
///     fn map<'a, M1, .., Mn, U1, .., Un>(
///         &'a self,
///         map_1: M1,
///         ..,
///         map_n: Mn,
///     ) -> Lend<'a, Upper, Self::WithParams<U1, .., Tn>>
///     where
///         M1: Fn(&'a Self::T1) -> Lend<'a, Upper, U1>,
///         ..,
///         Mn: Fn(&'a Self::Tn) -> Lend<'a, Upper, Un>,
///         Self::T1: 'a,
///         ..,
///         Self::Tn: 'a,
///         U1: LendFamily<Upper>,
///         ..,
///         Un: LendFamily<Upper>,
///         Upper: 'a;
///
///     /// Apply some mapping to `self`'s view components, obtaining a new value with the mapped
///     /// view components.
///     ///
///     /// See the trait-level documentation (or, macro-level as it were)
///     /// for implementation safety requirements.
///     fn map_mut<'a, M1, .., Mn, U1, .., Un>(
///         &'a mut self,
///         map_1: M1,
///         ..,
///         map_n: Mn,
///     ) -> Lend<'a, Upper, Self::WithParams<U1, .., Tn>>
///     where
///         M1: Fn(&'a mut Self::T1) -> Lend<'a, Upper, U1>,
///         ..,
///         Mn: Fn(&'a mut Self::Tn) -> Lend<'a, Upper, Un>,
///         Self::T1: 'a,
///         ..,
///         Self::Tn: 'a,
///         U1: LendFamily<Upper>,
///         ..,
///         Un: LendFamily<Upper>,
///         Upper: 'a;
/// }
/// ```
///
/// Your responsibility is to write an `impl` of this trait for your type, with some abbreviated
/// syntax.
///
/// ## Example
///
/// ```
/// use aliasable_view::map_aliasable;
/// mod path { mod to { mod your {
///     struct Type<'a, const FOO: u8, V, T1, T2: ?Sized + Debug>(&'a u8, T1, V, T2);
/// }}}
///
/// map_aliasable! {
///     // Information used to automatically fill in various parts of the "trait implementation",
///     // such as signatures, function parameters, where-bounds, and so on.
///     // Those components are denoted as `..` or `_` in the rest of the syntax.
///     Variadics = [
///         // The first parameter is used to define `Self` as `SelfWithoutParams<.., T1, .., Tn>`.
///         // The second parameter is the name of the corresponding `map_i` mapping function,
///         // which is in-scope for your `map` or `map_mut` implementation.
///         // Equivalents of `U1, .., Un` and `M1, .., Mn` are not exposed.
///         (T1, map_1),
///
///         // After each `Ti` parameter, you can optionally include where-bounds on
///         // `View<'_, Upper, Ti>` and `ViewMut<'_, Upper, Ti>`.
///         //
///         // The `Upper` parameter is (intentionally) not exposed to you, and you cannot access
///         // it due to macro hygiene. Therefore, this macro must expose these separate
///         // means to bound those types.
///         // Note that all `View`s and `ViewMut`s implement `Sized`.
///         (T2 (View: Debug) (ViewMut: Debug), map_2),
///     ];
///
///     // `Upper: UpperBound, T1, .., Tn` impl parameters are included automatically.
///     // To place bounds on any of the `Ti` parameters, use where-bounds.
///     // `Ti: ?Sized` bounds are *not* included by default.
///     //
///     // Remember, if the type contains values of type `V` which are not considered view
///     // components, then you **must** ensure that `V: 'static` so that non-view components
///     // do not have non-`'static` references.
///     unsafe impl<.., {const FOO: u8, V: 'static}> MapAliasable<_>
///     // Start by listing any type parameters not among `T1, .., Tn`. Those last parameters
///     // are included for you. Each parameter goes in its own set of braces.
///     for path::to::your::Type<{'static}, {FOO}, {V}, ..>
///     // `where {...}` is optional.
///     where {
///         // Any additional where-bounds for the "trait impl" of `MapAliasable` must go here.
///         T2: ?Sized,
///     }
///     {
///         // The `T1, .., Tn` and `WithParams` associated types are not included, since the
///         // information you provide above is sufficient to define them.
///
///         fn map<..>(&self, ..) -> _ where .. {
///             // You fill in this implementation for your type.
///
///             // Example:
///             path::to::your::Type(map_1(&self.1), map_2(&self.3))
///         }
///
///         fn map_mut<..>(&mut self, ..) -> _ where .. {
///             // You fill in this implementation for your type.
///
///             // Example:
///             path::to::your::Type(map_1(&mut self.1), map_2(&mut self.3))
///         }
///     }
/// }
/// ```
///
/// [`AliasableView`]: crate::traits::AliasableView
/// [`AliasableViewMut`]: crate::traits::AliasableViewMut
/// [`AliasableClone`]: crate::traits::AliasableClone
/// [`View<'_, Self, _>`]: crate::traits::View
/// [`ViewMut`]: crate::traits::ViewMut
#[macro_export]
macro_rules! map_aliasable {
    // Note: the "inline" requirement is not strictly needed; it's just sufficient for all current
    // usage of this macro, and, in my opinion, simplifies reasoning about the macro.

    // NOTE: `view_bounds`, `view_mut_bounds`, `impl_params`, `generics`, `where_bounds`,
    // `map_impl`, and `map_mut_impl` can contain arbitrary Rust code. We need to prevent
    // code injection from causing unexpected effects in this macro.
    {
        Variadics = [
            $((
                $t:ident $((View: $($view_bounds:tt)*) (ViewMut: $($view_mut_bounds:tt)*))?,
                $map:ident $(,)?
            )),* $(,)?
        ];

        unsafe impl<..$(, {$($impl_params:tt)*})?> MapAliasable<_>
        for $($name:ident)::+<$({$($generics:tt)*},)* ..>
        $(where {$($where_bounds:tt)*})?
        {
            fn map<..>(&$self_ref:ident, ..) -> _ where .. {
                $($map_impl:tt)*
            }

            fn map_mut<..>(&mut $self_mut:ident, ..) -> _ where .. {
                $($map_mut_impl:tt)*
            }
        }
    } => {
        // SAFETY: Asserted by user of this macro.
        const _: () = unsafe { $crate::__macro::unsafe_map_aliasable() };

        #[expect(
            unused_lifetimes,
            reason = "if a bound is unsatisfiable, the for<'maybe_unsat> lifetime binder \
                      means that the trait will simply never be implemented, \
                      instead of the impossible bound causing a compilation error",
        )]
        #[expect(
            unsafe_code,
            reason = "lint is moved to `unsafe_map_aliasable` for a clearer error message",
        )]
        const _: () = {
            // SAFETY:
            // Moves, coercions, or immutable operations (in any quantity and order) on `self`
            // cannot invalidate any `'static` data, so we need only care about the potentially
            // non-`'static` data in `self.view()`, which come solely from view components produced
            // from calling `.view()` on view components of `self`.
            // (This fact is a mixture of reasoning about the below impl and about the safety
            // requirements of this macro.)
            // Moves (or coercions, or immutable operations on) `self` (in any quantity and order)
            // can move (or coerce, or perform immutable operations on) its view components,
            // but since they are here required to implement `AliasableView`
            // (and since code injection is prevented), that does not invalidate their views.
            // (Note that we use the fact that performing immutable operations on `self` cannot
            // perform mutable operations on its view components; this is why we forbade internal
            // mutability.)
            //
            // Thus, moves, coercions, and immutable operations (in any quantity and order) on
            // `self` do not invalidate the value returned by `self.view()`, so this
            // implementation is sound.
            unsafe impl<
                Upper: $crate::__macro::variance_family::UpperBound,
                $(
                    $t: $crate::AliasableView<Upper, $(View: $($view_bounds)*)?>,
                )*
                $($($impl_params)*)?
            > $crate::AliasableView<Upper>
            for $($name)::+<$($($generics)*,)* $($t),*>
            where
                $($($where_bounds)*)?
            {
                type View = $($name)::+<
                    $($($generics)*,)*
                    $(<$t as $crate::AliasableView<Upper>>::View),*
                >;

                // We define `Upper` before expanding untrusted `tt` tokens, so by macro hygiene,
                // this mention of `Upper` prevents any of them from switching out this unsafe
                // impl.
                #[inline]
                fn view(&$self_ref) -> $crate::View<'_, Self, Upper> {
                    $(
                        let $map = <$t as $crate::AliasableView<Upper>>::view;
                    )*

                    // Rust has paired `{` and `}` delimiters, so `map_impl` can't do anything
                    // too crazy.
                    $($map_impl)*
                }
            }

            impl<
                Upper: $crate::__macro::variance_family::UpperBound,
                $(
                    $t: $crate::AliasableView<Upper, $(View: $($view_bounds)*)?>,
                )*
                $($($impl_params)*)?
            > $crate::IntoAliasable<Upper>
            for $($name)::+<$($($generics)*,)* $($t),*>
            where
                for<'maybe_sat> Self: Sized,
                $($($where_bounds)*)?
            {
                type IntoAliasable = Self;

                #[inline]
                fn into_aliasable(self) -> Self {
                    self
                }
            }

            // SAFETY:
            // Moves or coercions (in any quantity and order) of `self` cannot invalidate any
            // `'static` data, so we need only care about the potentially non-`'static` data in
            // `self.view_mut()`, which come solely from view components produced from calling
            // `.view_mut()` on view components of `self`.
            // (This fact is a mixture of reasoning about the below impl and about the safety
            // requirements of this macro.)
            // Moving (or coercing) `self` (in any quantity and order) can move (or coerce) its
            // view components, but since they are here required to implement `AliasableViewMut`
            // (and since code injection is prevented), that does not invalidate their views.
            //
            // Thus, moving or coercing `self` (in any quantity and order) does not
            // invalidate the value returned by `self.view_mut()`, so this implementation is sound.
            unsafe impl<
                Upper: $crate::__macro::variance_family::UpperBound,
                $(
                    $t: $crate::AliasableViewMut<
                        Upper,
                        $(View: $($view_bounds)*, ViewMut: $($view_bounds)*)?
                    >,
                )*
                $($($impl_params)*)?
            > $crate::AliasableViewMut<Upper>
            for $($name)::+<$($($generics)*,)* $($t),*>
            where
                $($($where_bounds)*)?
            {
                type ViewMut = $($name)::+<
                    $($($generics)*,)*
                    $(<$t as $crate::AliasableViewMut<Upper>>::ViewMut),*
                >;

                // We define `Upper` before expanding untrusted `tt` tokens, so by macro hygiene,
                // this mention of `Upper` prevents any of them from switching out this unsafe
                // impl.
                #[inline]
                fn view_mut(&mut $self_mut) -> $crate::ViewMut<'_, Self, Upper> {
                    $(
                        let $map = <$t as $crate::AliasableViewMut<Upper>>::view_mut;
                    )*

                    // Rust has paired `{` and `}` delimiters, so `map_mut_impl` can't do anything
                    // too crazy.
                    $($map_mut_impl)*
                }
            }

            impl<
                Upper: $crate::__macro::variance_family::UpperBound,
                $(
                    $t: $crate::AliasableViewMut<
                        Upper,
                        $(View: $($view_bounds)*, ViewMut: $($view_bounds)*)?
                    >,
                )*
                $($($impl_params)*)?
            > $crate::IntoAliasableMut<Upper>
            for $($name)::+<$($($generics)*,)* $($t),*>
            where
                for<'maybe_sat> Self: Sized,
                $($($where_bounds)*)?
            {}

            // SAFETY:
            // We can define the conceptual pool associated with a `self: Self` value as the set of
            // all values of some `SelfWithoutBounds<.., U1, .., Un>` type (or any other type
            // reachable via coercions of `SelfWithoutBounds<.., U1, .., Un>` values) whose view
            // components' set of conceptual pools is equal to the set of conceptual pools of
            // `self`'s view components.
            //
            // ...That definition is long. Put more simply: we're basically taking the intersection
            // of `self`'s view components' conceptual pools. Two `self` values are in the same
            // conceptual pool if their view components are in the same conceptual pools (in
            // any order, with any multiplicity). This definition ensures that `self` is in one
            // (and exactly one) conceptual pool at any given time. A view's associated pool is
            // the (fixed) conceptual pool which its source `Self` value was in at the time of its
            // creation.
            //
            // Requirement 1:
            // The caller of this macro guarantees that cloning `self` results in a new `Self`
            // value whose view components are clones of `self`'s view components and which contain
            // a clone of each of `self`'s view components. Therefore, the set of conceptual
            // `AliasableClone` pools of the view components of `self` is the same as the set
            // of conceptual pools of the view components of the clone of `self`. By definition,
            // the clone is in the same pool as `self`, satisfying requirement 1.
            //
            // Requirement 2:
            // Only the view components of `self` determine which pool it's in, so we need only
            // care about the impact of moves, coercions, and immutable operations on `self`'s
            // view components. Moves (or coercions, or immutable operations on) `self`
            // (in any quantity and order) can move (or coerce, or perform immutable operations on)
            // its view components, and since they are here required to implement `AliasableClone`
            // (and since code injection is prevented), that does not change which pools they are
            // in.
            // (Note that we use the fact that performing immutable operations on `self` cannot
            // perform mutable operations on its view components; this is why we forbade internal
            // mutability.)
            //
            // Thus, moves, coercions, and immutable operations (in any quantity and order) on
            // `self` do not change which pool `self` is in.
            //
            // Requirement 3:
            // If the conceptual pool associated with a view returned by `self.view()` is nonempty,
            // then there exists some value of type `Self` (or formerly of type `Self` before
            // coercions) whose view components are in the same pools as the view components
            // which were in `self` at the time the view was made. Each of the view
            // components in `self.view()` came from view components in `self` at the time the view
            // was made, so the associated pools of the view components of `self.view()` are all
            // therefore nonempty. Since the `'static` data in the view cannot be invalidated
            // and the only non-`'static` data is in its view components, we thus have that the
            // view has not been invalidated.
            unsafe impl<
                Upper: $crate::__macro::variance_family::UpperBound,
                $($t: $crate::AliasableClone<Upper>,)*
                $($($impl_params)*)?
            > $crate::AliasableClone<Upper>
            for $($name)::+<$($($generics)*,)* $($t),*>
            where
                for<'maybe_sat> Self: ::core::clone::Clone,
                // We define `Upper` before expanding untrusted `tt` tokens, so by macro hygiene,
                // this mention of `Upper` prevents any of them from switching out this unsafe
                // impl. There is still `where_bounds` below, but it can't do any damage.
                Upper:,
                $($($where_bounds)*)?
            {}

        };
    };
}

/// Denotes that [`map_aliasable`] was used.
///
/// Used to trigger the `unsafe_code` lint.
///
/// # Safety
/// This function itself has no safety preconditions, but though the safety requirements for
/// using [`map_aliasable`] must be upheld.
pub const unsafe fn unsafe_map_aliasable() {}
