//! A simple crate that allow to use the Tcl C library in a more rustic way.
#![warn(missing_docs)]
use std::{env,
          ffi::CString,
          sync::{Once, ONCE_INIT}};

extern crate failure;

#[macro_use]
extern crate failure_derive;

extern crate rusty_tcl_sys;

pub mod error;
pub mod interp;
pub mod obj;

pub use error::TclError;
pub use interp::TclInterp;
pub use obj::TclObj;

static TCL_INIT: Once = ONCE_INIT;

pub(crate) fn init() {
    TCL_INIT.call_once(|| {
        let arg0 = CString::new(env::args().nth(0).unwrap()).unwrap();
        unsafe {
            rusty_tcl_sys::Tcl_FindExecutable(arg0.as_ptr());
            assert!(!rusty_tcl_sys::Tcl_GetNameOfExecutable().is_null());
        }
    });
}
