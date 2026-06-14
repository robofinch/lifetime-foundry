//! Planned module for `stable-view` implementations for `AttachableRefFull`.

// AttachableRefFull should be viewable through `RecursiveView<(VN, VR, VM, VD)>`,
// or `RecursiveView<(VN, VR, VM)>`.
// AttachableRefFull<.., N, R, Infallible, Data> should be viewable through RecursiveView<(VD,)>

// Default view: no default.
// AttachableRef and AttachedRef should have defaults, though.
// AttachableMut and AttachedMut could also have defaults, I guess.
