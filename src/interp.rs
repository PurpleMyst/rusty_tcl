//! A module that holds the [`TclInterp`](struct.TclInterp.html) struct.
use super::{error::TclError, obj::TclObj, rusty_tcl_sys};

use std::{ffi::{CStr, CString},
          os::raw::c_int,
          ptr::NonNull};

macro_rules! option_to_err {
    ($option:expr) => {
        if let Some(err) = $option {
            Err(err)
        } else {
            Ok(())
        }
    };
}

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
    ///
    /// # Errors
    /// This function returns an [`Err`] value if:
    ///     1. The pointer returned by `Tcl_CreateInterp` is NULL.
    ///     2. [`TclError::from_completion_code`] returns an Err given the completion code from
    ///        `Tcl_Init`.
    pub fn new() -> Result<Self, TclError> {
        super::init();
        let interp_ptr = NonNull::new(unsafe { rusty_tcl_sys::Tcl_CreateInterp() })
            .ok_or(TclError::NullPointer)?;

        let mut this = Self { interp_ptr };

        this.app_init()?;

        Ok(this)
    }

    fn app_init<'a>(&'a mut self) -> Result<(), TclError> {
        let cc = unsafe { rusty_tcl_sys::Tcl_Init(self.interp_ptr.as_ptr()) };
        option_to_err!(TclError::from_completion_code(&self, cc))
    }

    /// Fetches the interpreter's internal string result.
    ///
    /// # Errors
    /// This function returns an error if the interpreter's internal string result contains bytes
    /// that are invalid UTF-8, which should never happen.
    ///
    /// It also returns an error when the interpreter's internal string result is NULL.
    pub fn get_string_result(&self) -> Result<&str, TclError> {
        let c_str = unsafe { rusty_tcl_sys::Tcl_GetStringResult(self.interp_ptr.as_ptr()) };

        if c_str.is_null() {
            Err(TclError::NullPointer)
        } else {
            unsafe { CStr::from_ptr(c_str).to_str().map_err(TclError::from) }
        }
    }

    /// Fetches the interpreter's internal object result.
    ///
    /// # Errors
    /// This functions returns an error when [`TclObj::from_ptr`] does.
    pub fn get_object_result(&self) -> Result<TclObj, TclError> {
        let c_obj = unsafe { rusty_tcl_sys::Tcl_GetObjResult(self.interp_ptr.as_ptr()) };

        TclObj::from_ptr(c_obj)
    }

    /// Make this interpreter safe.
    ///
    /// # Notes
    /// As noted in the `Tcl_MakeSafe` man page, a "safe" interpreter only removes **core**
    /// potentially-unsafe functions. It's **your** responsibility to make sure any extensions you
    /// use are safe.
    pub fn make_safe(&mut self) -> Result<(), TclError> {
        option_to_err!(TclError::from_completion_code(&self, unsafe {
            rusty_tcl_sys::Tcl_MakeSafe(self.interp_ptr.as_ptr())
        }))
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

    /// Evaluates a piece of Tcl code.
    ///
    /// # Notes
    /// This just returns an unit value or an error, to get the actual result you need
    /// [`TclInterp::get_string_result`] or [`TclInterp::get_object_result`].
    pub fn eval(&mut self, code: &str) -> Result<(), TclError> {
        let c_code = CString::new(code).map_err(TclError::from)?;

        let raw_completion_code =
            unsafe { rusty_tcl_sys::Tcl_Eval(self.interp_ptr.as_ptr(), c_code.as_ptr()) };

        option_to_err!(TclError::from_completion_code(&self, raw_completion_code))
    }

    /// Sets a variable with the given `name` to the given `value`.
    ///
    /// # Panics
    /// This function panics if `name` or `value` contain NUL bytes.
    ///
    /// # Notes
    /// This returns the value that `name` was set to, which may differ from `value` due to
    /// tracing.
    pub fn set_var(
        &mut self,
        name: impl Into<Vec<u8>>,
        value: impl Into<Vec<u8>>,
    ) -> Result<&CStr, TclError> {
        let flags: c_int = rusty_tcl_sys::TCL_LEAVE_ERR_MSG as c_int;

        let result_ptr = unsafe {
            rusty_tcl_sys::Tcl_SetVar(
                self.interp_ptr.as_ptr(),
                CString::new(name).unwrap().as_ptr(),
                CString::new(value).unwrap().as_ptr(),
                flags,
            )
        };

        if result_ptr.is_null() {
            Err(TclError::from_string_result(&self))
        } else {
            Ok(unsafe { CStr::from_ptr(result_ptr) })
        }
    }
}

#[cfg(test)]
mod tests {
    use TclInterp;

    #[test]
    fn it_works() {
        let mut interp = TclInterp::new().unwrap();

        macro_rules! tcl_assert_eq {
            ($cc:expr, $expected:expr) => {{
                assert_eq!($cc.unwrap(), ());
                assert_eq!(interp.get_string_result().unwrap(), $expected);
            }};
        };

        tcl_assert_eq!(interp.eval("expr {2 + 2}"), "4");

        assert!(interp.set_var("x", "5").is_ok());
        tcl_assert_eq!(interp.eval("return $x"), "5");

        interp.make_safe().unwrap();
        assert!(interp.is_safe());

        // TODO: Test `get_object_result`
    }
}
