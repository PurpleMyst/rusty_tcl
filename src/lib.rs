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

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use {completion_code, interp};

    macro_rules! cstr {
        ($s:expr) => {
            CString::new($s).unwrap()
        }
    }

    // TODO: Move these to `interp.rs`
    #[test]
    fn it_works() {
        let mut interp = interp::TclInterp::new().unwrap();

        macro_rules! tcl_assert_eq {
            ($cc:expr, $expected:expr) => {{
                let cc = $cc;
                if let completion_code::CompletionCode::Error(msg) = cc {
                    panic!("{}", msg.into_string().unwrap());
                }

                assert_eq!(interp.get_string_result(), cstr!($expected).as_ref());
            }}
        };

        tcl_assert_eq!(interp.eval(cstr!("expr {2 + 2}")), "4");

        assert!(interp.set_var(cstr!("x"), cstr!("5")).is_ok());
        tcl_assert_eq!(interp.eval(cstr!("return $x")), "5");

        interp.make_safe().panic_if_error();
        assert!(interp.is_safe());
    }
}
