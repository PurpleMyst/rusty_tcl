//! A module that holds the [`TclInterp`](struct.TclInterp.html) struct.
use super::{error::TclError, obj::TclObj, rusty_tcl_sys};

use std::{ffi::{CStr, CString},
          os::raw::c_int,
          ptr::NonNull};

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
    ///     2. Initialization of the current interpreter via `Tcl_Init` fails.
    pub fn new() -> Result<Self, TclError> {
        super::init();
        let interp_ptr = NonNull::new(unsafe { rusty_tcl_sys::Tcl_CreateInterp() })
            .ok_or(TclError::NullPointer)?;

        let mut this = Self { interp_ptr };

        this.init()?;

        Ok(this)
    }

    // XXX: Can we refactor this out?
    fn init(&mut self) -> Result<(), TclError> {
        let cc = unsafe { rusty_tcl_sys::Tcl_Init(self.interp_ptr.as_ptr()) };

        self.cc_to_result(cc)
    }

    fn get_string_result(&self) -> Result<&str, TclError> {
        let c_str = unsafe { rusty_tcl_sys::Tcl_GetStringResult(self.interp_ptr.as_ptr()) };

        if c_str.is_null() {
            Err(TclError::NullPointer)
        } else {
            unsafe { CStr::from_ptr(c_str).to_str().map_err(TclError::from) }
        }
    }

    pub(crate) fn get_object_result(&self) -> Result<TclObj, TclError> {
        let c_obj = unsafe { rusty_tcl_sys::Tcl_GetObjResult(self.interp_ptr.as_ptr()) };

        TclObj::from_ptr(c_obj)
    }

    /// Makes this interpreter safe.
    ///
    /// # Errors
    /// This function returns an error when `Tcl_MakeSafe` fails.
    ///
    /// # Notes
    /// As noted in the `Tcl_MakeSafe` man page, a "safe" interpreter only removes **core**
    /// potentially-unsafe functions. It's **your** responsibility to make sure any extensions you
    /// use are safe.
    pub fn make_safe(&mut self) -> Result<(), TclError> {
        self.cc_to_result(unsafe {
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

    fn cc_to_result(&self, cc: c_int) -> Result<(), TclError> {
        if cc == (rusty_tcl_sys::TCL_ERROR as c_int) {
            Err(TclError::InternalError(self.get_string_result()?.to_owned()))
        } else {
            Ok(())
        }
    }

    fn eval(&mut self, code: &str) -> Result<(), TclError> {
        let c_code = CString::new(code).map_err(TclError::from)?;

        let cc =
            unsafe { rusty_tcl_sys::Tcl_Eval(self.interp_ptr.as_ptr(), c_code.as_ptr()) };

        self.cc_to_result(cc)
    }

    /// Evaluates code, returning the interpreter's string result.
    ///
    /// # Errors
    /// This function returns any errors that happen in the code as a
    /// `Err(TclError::InternalError(..))`, and it also returns any errors that happen when getting
    /// the interpreter's string result.
    pub fn eval_to_string(&mut self, code: &str) -> Result<&str, TclError> {
        match self.eval(code) {
            Ok(()) => self.get_string_result(),

            // NB: This is not the same as `err @ Err(_) => err`.
            Err(err) => Err(err),
        }
    }

    /// Evaluates code, returning the interpreter's object result.
    ///
    /// # Errors
    /// This function returns any errors that happen in the code as a
    /// `Err(TclError::InternalError(..))`, and it also returns any errors that happen when getting
    /// the interpreter's object result.
    pub fn eval_to_obj(&mut self, code: &str) -> Result<TclObj, TclError> {
        match self.eval(code) {
            Ok(()) => self.get_object_result(),

            // NB: This is not the same as `err @ Err(_) => err`.
            Err(err) => Err(err),
        }
    }

    /// Sets a variable with the given `name` to the given `value`.
    ///
    /// # Errors
    /// This function an error if either `name` or `value` contain NUL bytes, or if the pointer
    /// returned by `Tcl_SetVar` is NULL.
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
                CString::new(name)?.as_ptr(),
                CString::new(value)?.as_ptr(),
                flags,
            )
        };

        if result_ptr.is_null() {
            Err(TclError::InternalError(self.get_string_result()?.to_owned()))
        } else {
            Ok(unsafe { CStr::from_ptr(result_ptr) })
        }
    }
}

#[cfg(test)]
mod tests {
    use TclInterp;

    #[test]
    fn eval_to_string() {
        let mut interp = TclInterp::new().unwrap();

        assert_eq!(interp.eval_to_string("expr {2 + 2}").unwrap(), "4");
    }

    #[test]
    fn eval_to_object() {
        // TODO
    }

    #[test]
    fn set_var() {
        let mut interp = TclInterp::new().unwrap();

        interp.set_var("x", "5").unwrap();
        assert_eq!(interp.eval_to_string("return $x").unwrap(), "5");
    }

    #[test]
    fn safety() {
        let mut interp = TclInterp::new().unwrap();
        interp.make_safe().unwrap();
        assert!(interp.is_safe());
    }
}
