//! View kinds whose implementations are *all* provided by this crate, for all `Data` types.

#![expect(unsafe_code, reason = "defer to other unsafe impls, and trivial unsafe impls")]

use variance_family::Unvarying;

use crate::{
    traits::{StableView, StableViewMut},
    viewer::{StableViewer, StableViewerMut},
    viewer_families::{VaryingStableViewer, VaryingStableViewerMut},
};


/// A trivial view kind (or mutable view kind) whose returned view is always `()`.
#[derive(Debug, Default, Clone, Copy)]
pub struct UnitViewKind;

impl<'a, Data: ?Sized> StableView<'a, '_, Data> for UnitViewKind {
    type View = ();

    #[inline]
    unsafe fn view<'stable: 'stable>(_data: &'a Data) {}
}

impl<'a, Data: ?Sized> StableViewMut<'a, '_, Data> for UnitViewKind {
    type ViewMut = ();

    #[inline]
    unsafe fn view_mut<'stable: 'stable>(_data: &'a mut Data) {}
}

/// A trivial view kind (or mutable view kind) whose returned views have no `'stable` references.
///
/// Its view methods are no-ops.
#[derive(Debug, Default, Clone, Copy)]
pub struct UnstableViewKind;

impl<'a, Data: ?Sized + 'a> StableView<'a, '_, Data> for UnstableViewKind {
    type View = &'a Unvarying<Data>;

    #[inline]
    unsafe fn view<'stable: 'stable>(data: &'a Data) -> &'a Data {
        data
    }
}

impl<'a, Data: ?Sized + 'a> StableViewMut<'a, '_, Data> for UnstableViewKind {
    type ViewMut = &'a mut Unvarying<Data>;

    #[inline]
    unsafe fn view_mut<'stable: 'stable>(data: &'a mut Data) -> &'a mut Data {
        data
    }
}

/// Implement [`StableView`] and [`StableViewMut`] by returning a [`StableViewer`] or
/// [`StableViewerMut`].
#[derive(Debug, Default, Clone, Copy)]
pub struct ViewerViewKind;

impl<'a, 'data, Data: ?Sized + 'a> StableView<'a, 'data, Data> for ViewerViewKind {
    type View = VaryingStableViewer<'a, 'data, Data>;

    #[inline]
    unsafe fn view<'stable>(data: &'a Data) -> StableViewer<'a, 'stable, 'data, Data>
    where
        'data: 'stable,
        'stable: 'a
    {
        // SAFETY: The safety conditions of `StableViewer::new` are precisely the conditions
        // necessary to *safely* use the view returned by `StableView::view`.
        //
        // If the caller chooses to *unsafely* use the returned view, with an overly-long `stable`
        // or lifetime extension, then they are responsible for not exposing the view to untrusted
        // code with an overly-long lifetime. Such a caller is responsible for both the `'stable`
        // lifetime of the returned viewer *and* the `'stable` data obtained via the returned viewer
        // (since using that `'stable` data is still considered to be using the `'stable` data of
        // the returned viewer -- in this case, phantom data, which still counts).
        //
        // Therefore, the safety conditions are asserted by the caller.
        unsafe { StableViewer::new(data) }
    }
}

impl<'a, 'data, Data: ?Sized + 'a> StableViewMut<'a, 'data, Data> for ViewerViewKind {
    type ViewMut = VaryingStableViewerMut<'a, 'data, Data>;

    #[inline]
    unsafe fn view_mut<'stable>(data: &'a mut Data) -> StableViewerMut<'a, 'stable, 'data, Data>
    where
        'data: 'stable,
        'stable: 'a
    {
        // SAFETY: The safety conditions of `StableViewerMut::new` are precisely the conditions
        // necessary to *safely* use the view returned by `StableViewMut::view_mut`.
        //
        // As above, if the caller chooses to *unsafely* use the returned view, the documentation
        // of `view_mut` doesn't grant the caller overly strong power.
        //
        // Therefore, the safety conditions are asserted by the caller.
        unsafe { StableViewerMut::new(data) }
    }
}
