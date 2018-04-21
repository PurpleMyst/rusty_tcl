//! A module that holds the [`TclInterp`](struct.TclInterp.html) struct.
use super::{completion_code::CompletionCode, tcl_ffi};

use std::convert::AsRef;
use std::ffi::CStr;
use std::os::raw::c_int;
use std::ptr::NonNull;

/// Interpreter struct that holds the Tcl interpreter itself.
// TODO: mark this not thread safe
pub struct TclInterp {
    interp_ptr: NonNull<tcl_ffi::Tcl_Interp>,
}

impl Drop for TclInterp {
    fn drop(&mut self) {
        unsafe { tcl_ffi::Tcl_DeleteInterp(self.interp_ptr.as_ptr()) }
    }
}

impl TclInterp {
    /// Creates a new interpreter.
    ///
    /// # Panics
    /// This function panics if the pointer returned by [`Tcl_CreateInterp`](https://www.tcl.tk/man/tcl/TclLib/CrtInterp.htm) is NULL.
    /// This will change in the future so this method will return an [`Err`] when this happens.
    ///
    /// # Errors
    /// This function returns an [`Err`] value with a [`CompletionCode::Error`] if [`Tcl_AppInit`](https://tcl.tk/man/tcl/TclLib/AppInit.htm) returns an error completion code.
    pub fn new() -> Result<Self, CompletionCode> {
        // TODO: Use `Option::ok_or_else` here.
        let interp_ptr = NonNull::new(unsafe { tcl_ffi::Tcl_CreateInterp() }).unwrap();

        let mut this = Self { interp_ptr, };

        if let err @ CompletionCode::Error(_) = this.app_init() {
            return Err(err)
        }

        Ok(this)
    }

    fn app_init<'a>(&'a mut self) -> CompletionCode {
        self.completioncode_from_int(unsafe {
            tcl_ffi::Tcl_Init(self.interp_ptr.as_ptr())
        })
    }

    /// Fetches the interpreter's internal string result.
    pub fn get_string_result<'a>(&self) -> &'a CStr {
        unsafe { CStr::from_ptr(tcl_ffi::Tcl_GetStringResult(self.interp_ptr.as_ptr())) }
    }

    fn completioncode_from_int(&self, raw_completion_code: c_int) -> CompletionCode {
        match raw_completion_code {
            tcl_ffi::TCL_OK => CompletionCode::Ok,
            tcl_ffi::TCL_ERROR => CompletionCode::Error(self.get_string_result().to_owned()),
            tcl_ffi::TCL_RETURN => CompletionCode::Return,
            tcl_ffi::TCL_BREAK => CompletionCode::Break,
            tcl_ffi::TCL_CONTINUE => CompletionCode::Continue,

            _ => panic!("Invalid completion code {:?}", raw_completion_code),
        }
    }

    /// Evaluates a piece of Tcl code.
    ///
    /// # Notes
    /// This just returns a [`CompletionCode`], to get the code's result you need to use
    /// [`TclInterp::get_string_result`].
    pub fn eval(&mut self, code: impl AsRef<CStr>) -> CompletionCode {
        let raw_completion_code =
            unsafe { tcl_ffi::Tcl_Eval(self.interp_ptr.as_ptr(), code.as_ref().as_ptr()) };

        self.completioncode_from_int(raw_completion_code)
    }
}
