//! Small module for custom error types of this crate.

use core::error::Error;
use core::fmt::{Debug, Display, Formatter, Result as FmtResult};


/// The error returned by various `try_attach*` methods.
#[derive(Debug, Clone, Copy)]
pub struct TryAttachError<D, E> {
    /// The source data, to which a self-reference failed to be constructed.
    pub data:  D,
    /// The error reported by a callback constructing a self-reference.
    pub error: E,
}

impl<D, E: Display> Display for TryAttachError<D, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.error, f)
    }
}

impl<D: Debug, E: Error> Error for TryAttachError<D, E> {}
