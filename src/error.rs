//! A module that contains the [`TclError`](enum.TclError.html) enum.
use super::completion_code::CompletionCode;

use std::{ffi::NulError, str::Utf8Error};

#[derive(Debug, Fail)]
/// Represents different classes of Tcl Errors.
pub enum TclError {
    /// Represents a pointer being NULL when it shouldn't have been. This usually happens in
    /// functions like [`TclObj::new`].
    #[fail(display = "A pointer was NULL when it shouldn't have been.")]
    NullPointer,

    #[fail(display = "The string {:?} contained NUL bytes when it shouldn't.", _0)]
    /// Represents a string containing NUL bytes when it shouldn't have. This can happen in pretty
    /// much any function that takes a [`String`].
    NulBytes(String),

    #[fail(display = "A string returned by the interpreter contained NUL bytes when it shouldn't.")]
    /// Represents a C string containing bytes that are invalid UTF-8.
    InvalidUtf8,

    /// Represents an error returned by Tcl itself.
    #[fail(display = "Tcl returned completion code {:?}", _0)]
    InternalError(CompletionCode<'static>),
}

impl TclError {
    // Creates a `Result<(), TclError>` from a completion code.
    //
    // This returns Ok(()) if the argument is anything but a [`CompletionCode::Error`].
    pub(crate) fn from_completion_code(cc: CompletionCode<'static>) -> Result<(), TclError> {
        if let CompletionCode::Error(_) = cc {
            Err(TclError::InternalError(cc))
        } else {
            Ok(())
        }
    }
}

impl From<Utf8Error> for TclError {
    fn from(_err: Utf8Error) -> TclError {
        TclError::InvalidUtf8
    }
}

impl From<NulError> for TclError {
    fn from(err: NulError) -> TclError {
        match String::from_utf8(err.into_vec()) {
            Ok(s) => TclError::NulBytes(s),
            Err(_) => TclError::InvalidUtf8,
        }
    }
}
