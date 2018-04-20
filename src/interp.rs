use super::{completion_code::CompletionCode, tcl_ffi};
use std::convert::AsRef;
use std::ffi::CStr;
use std::os::raw::c_int;
use std::ptr::NonNull;

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
    pub fn new() -> Option<Self> {
        let interp_ptr = unsafe { tcl_ffi::Tcl_CreateInterp() };

        Some(Self {
            interp_ptr: NonNull::new(interp_ptr)?,
        })
    }

    pub fn get_string_result<'a>(&self) -> &'a CStr {
        unsafe { CStr::from_ptr(tcl_ffi::Tcl_GetStringResult(self.interp_ptr.as_ptr())) }
    }

    fn completioncode_from_int(&self, raw_completion_code: c_int) -> CompletionCode {
        match raw_completion_code {
            tcl_ffi::TCL_OK => CompletionCode::Ok,
            tcl_ffi::TCL_ERROR => CompletionCode::Error(self.get_string_result()),
            tcl_ffi::TCL_RETURN => CompletionCode::Return,
            tcl_ffi::TCL_BREAK => CompletionCode::Break,
            tcl_ffi::TCL_CONTINUE => CompletionCode::Continue,

            _ => panic!("Invalid completion code {:?}", raw_completion_code),
        }
    }

    pub fn eval(&mut self, code: impl AsRef<CStr>) -> CompletionCode {
        let raw_completion_code =
            unsafe { tcl_ffi::Tcl_Eval(self.interp_ptr.as_ptr(), code.as_ref().as_ptr()) };

        self.completioncode_from_int(raw_completion_code)
    }
}
