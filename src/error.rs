//! A module that contains the [`TclError`](enum.TclError.html) enum.
use super::{interp::TclInterp, rusty_tcl_sys};

use std::{ffi::NulError,
          os::raw::{c_int, c_uint},
          str::Utf8Error};

#[derive(Debug, Fail)]
/// Represents different classes of Tcl Errors.
pub enum TclError {
    /// Represents a pointer being NULL when it shouldn't have been. This usually happens in
    /// functions like [`TclObj::new`].
    #[fail(display = "A pointer was NULL when it shouldn't have been.")]
    NullPointer,

    /// Represents a string containing NUL bytes when it shouldn't have. This can happen in pretty
    /// much any function that takes a [`String`].
    #[fail(display = "The string {:?} contained NUL bytes when it shouldn't.", _0)]
    NulBytes(String),

    /// Represents a C string containing bytes that are invalid UTF-8.
    #[fail(display = "A string returned by the interpreter contained NUL bytes when it shouldn't.")]
    InvalidUtf8,

    /// Represents an error returned by Tcl itself.
    #[fail(display = "Tcl returned error{:?}", _0)]
    InternalError(String),
}

impl TclError {
    pub(crate) fn from_string_result(interp: &TclInterp) -> TclError {
        match interp.get_string_result() {
            Ok(msg) => TclError::InternalError(msg.to_owned()),

            // XXX: What should we do here?
            Err(e) => e,
        }
    }

    pub(crate) fn from_completion_code(interp: &TclInterp, cc: c_int) -> Option<TclError> {
        if let rusty_tcl_sys::TCL_ERROR = cc as c_uint {
            Some(Self::from_string_result(&interp))
        } else {
            None
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
