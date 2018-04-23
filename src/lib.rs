//! A simple crate that allow to use the Tcl C library in a more rustic way.
#![warn(missing_docs)]
use std::{ffi::CString, env, sync::{Once, ONCE_INIT}};

extern crate rusty_tcl_sys;

pub mod completion_code;
pub mod interp;
pub mod obj;

pub use completion_code::CompletionCode;
pub use obj::TclObj;
pub use interp::TclInterp;

static TCL_INIT: Once = ONCE_INIT;

/// Initialize the Tcl environment.
///
/// # Notes
/// You should not need to call this normally. It's automagically called in [`TclObj::new`] and
/// [`TclInterp::new`].
pub fn init() {
    TCL_INIT.call_once(|| {
        let arg0 = CString::new(env::args().nth(0).unwrap()).unwrap();;
        unsafe {
        rusty_tcl_sys::Tcl_FindExecutable(arg0.as_ptr());
        assert!(!rusty_tcl_sys::Tcl_GetNameOfExecutable().is_null());
        }
        // TODO: Call Tcl_Init?
    });
}
