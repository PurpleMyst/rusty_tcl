//! A module that holds the [`TclInterp`](struct.TclInterp.html) struct.
use super::{completion_code::CompletionCode, rusty_tcl_sys};

use std::convert::AsRef;
use std::ffi::CStr;
use std::os::raw::{c_int, c_uint};
use std::ptr::NonNull;

/// Interpreter struct that holds the Tcl interpreter itself.
// TODO: mark this not thread safe
pub struct TclInterp {
    interp_ptr: NonNull<rusty_tcl_sys::Tcl_Interp>,
}

impl Drop for TclInterp {
    fn drop(&mut self) {
        unsafe { rusty_tcl_sys::Tcl_DeleteInterp(self.interp_ptr.as_ptr()) }
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
        super::init();
        // TODO: Use `Option::ok_or_else` here.
        let interp_ptr = NonNull::new(unsafe { rusty_tcl_sys::Tcl_CreateInterp() }).unwrap();

        let mut this = Self { interp_ptr };

        if let err @ CompletionCode::Error(_) = this.app_init() {
            return Err(err);
        }

        Ok(this)
    }

    fn app_init<'a>(&'a mut self) -> CompletionCode {
        self.completioncode_from_int(unsafe { rusty_tcl_sys::Tcl_Init(self.interp_ptr.as_ptr()) })
    }

    /// Fetches the interpreter's internal string result.
    pub fn get_string_result<'a>(&self) -> &'a CStr {
        unsafe { CStr::from_ptr(rusty_tcl_sys::Tcl_GetStringResult(self.interp_ptr.as_ptr())) }
    }

    /// Make this interpreter safe.
    ///
    /// # Notes
    /// As noted in the `Tcl_MakeSafe` man page, a "safe" interpreter only removes **core**
    /// potentially-unsafe functions. It's **your** responsibility to make sure any extensions you
    /// use are safe.
    pub fn make_safe(&mut self) -> CompletionCode {
        self.completioncode_from_int(unsafe {
            rusty_tcl_sys::Tcl_MakeSafe(self.interp_ptr.as_ptr())
        })
    }

    /// Returns `true` if the current interpreter is safe.
    pub fn is_safe(&self) -> bool {
        let bad_bool = unsafe { rusty_tcl_sys::Tcl_IsSafe(self.interp_ptr.as_ptr()) };

        match bad_bool {
            0 => false,
            1 => true,
            _ => unreachable!(),
        }
    }

    fn completioncode_from_int(&self, raw_completion_code: c_int) -> CompletionCode {
        match raw_completion_code as c_uint {
            rusty_tcl_sys::TCL_OK => CompletionCode::Ok,
            rusty_tcl_sys::TCL_ERROR => CompletionCode::Error(self.get_string_result().to_owned()),
            rusty_tcl_sys::TCL_RETURN => CompletionCode::Return,
            rusty_tcl_sys::TCL_BREAK => CompletionCode::Break,
            rusty_tcl_sys::TCL_CONTINUE => CompletionCode::Continue,

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
            unsafe { rusty_tcl_sys::Tcl_Eval(self.interp_ptr.as_ptr(), code.as_ref().as_ptr()) };

        self.completioncode_from_int(raw_completion_code)
    }

    /// Sets a variable with the given `name` to the given `value`.
    ///
    /// # Notes
    /// This returns the value that `name` was set to, which may differ from `value` due to
    /// tracing.
    pub fn set_var<'a>(
        &mut self,
        name: impl AsRef<CStr>,
        value: impl AsRef<CStr>,
    ) -> Result<&'a CStr, CompletionCode> {
        let flags: c_int = rusty_tcl_sys::TCL_LEAVE_ERR_MSG as c_int;

        let result_ptr = unsafe {
            rusty_tcl_sys::Tcl_SetVar(
                self.interp_ptr.as_ptr(),
                name.as_ref().as_ptr(),
                value.as_ref().as_ptr(),
                flags,
            )
        };

        if result_ptr.is_null() {
            Err(CompletionCode::Error(self.get_string_result().to_owned()))
        } else {
            Ok(unsafe { CStr::from_ptr(result_ptr) })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use {CompletionCode, TclInterp};

    macro_rules! cstr {
        ($s:expr) => {
            CString::new($s).unwrap()
        };
    }

    #[test]
    fn it_works() {
        let mut interp = TclInterp::new().unwrap();

        macro_rules! tcl_assert_eq {
            ($cc:expr, $expected:expr) => {{
                let cc = $cc;
                if let CompletionCode::Error(msg) = cc {
                    panic!("{}", msg.into_string().unwrap());
                }

                assert_eq!(interp.get_string_result(), cstr!($expected).as_ref());
            }};
        };

        tcl_assert_eq!(interp.eval(cstr!("expr {2 + 2}")), "4");

        assert!(interp.set_var(cstr!("x"), cstr!("5")).is_ok());
        tcl_assert_eq!(interp.eval(cstr!("return $x")), "5");

        interp.make_safe().panic_if_error();
        assert!(interp.is_safe());
    }
}
