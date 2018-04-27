//! A module that contains the [`TclObj`](struct.TclObj.html).
use super::{error::TclError, rusty_tcl_sys};

use std::{ffi::CStr,
          ptr::{self, NonNull}};

/// A struct representing the `Tcl_Obj` type.
#[derive(PartialEq, Eq, Debug)]
pub struct TclObj {
    obj_ptr: NonNull<rusty_tcl_sys::Tcl_Obj>,
}

impl Clone for TclObj {
    fn clone(&self) -> Self {
        let mut child = Self {
            obj_ptr: self.obj_ptr,
        };
        child.incr_ref_count();
        child
    }
}

impl Drop for TclObj {
    fn drop(&mut self) {
        self.decr_ref_count();
    }
}

impl TclObj {
    /// Creates a new, empty, Tcl object.
    ///
    /// # Errors
    /// This returns `Err(TclError::NullPointer)` when the pointer returned by `Tcl_NewObj` is NULL.
    ///
    /// # Panics
    /// This function panics if TCL itself fails to initialize.
    pub fn new() -> Result<Self, TclError> {
        super::init();
        let obj_ptr = unsafe { rusty_tcl_sys::Tcl_NewObj() };
        Self::from_ptr(obj_ptr)
    }

    pub(crate) fn from_ptr(obj_ptr: *mut rusty_tcl_sys::Tcl_Obj) -> Result<Self, TclError> {
        super::init();
        let mut this = Self {
            obj_ptr: NonNull::new(obj_ptr).ok_or(TclError::NullPointer)?,
        };
        this.incr_ref_count();
        Ok(this)
    }

    // TODO: Use the `Tcl_IncrRefCount` macro here instead.
    // XXX: Do we need `ptr::read_unaligned`?
    fn incr_ref_count(&mut self) {
        unsafe {
            let obj_ptr = self.obj_ptr.as_ptr();
            let mut tcl_obj = ptr::read_unaligned(obj_ptr);
            tcl_obj.refCount += 1;
            ptr::write_unaligned(obj_ptr, tcl_obj);
        }
    }

    fn decr_ref_count(&mut self) -> bool {
        let should_free = unsafe {
            let obj_ptr = self.obj_ptr.as_ptr();
            let mut tcl_obj = ptr::read_unaligned(obj_ptr);
            tcl_obj.refCount -= 1;
            ptr::write_unaligned(obj_ptr, tcl_obj);
            tcl_obj.refCount <= 0
        };

        if should_free {
            unsafe {
                rusty_tcl_sys::TclFreeObj(self.obj_ptr.as_ptr());
            }
            true
        } else {
            false
        }
    }

    /// Returns `true` if there are multiple [`TclObj`]s that point to the same underlying struct.
    pub fn is_shared(&self) -> bool {
        let obj_struct = unsafe { *(self.obj_ptr.as_ptr()) };
        obj_struct.refCount > 1
    }
}

impl ::std::fmt::Display for TclObj {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let string_repr = unsafe {
            let c_str = rusty_tcl_sys::Tcl_GetStringFromObj(self.obj_ptr.as_ptr(), ptr::null_mut());

            // XXX: Use `CStr::from_bytes_with_nul`?
            CStr::from_ptr(c_str).to_str().unwrap()
        };

        f.write_str(string_repr)
    }
}

// `ToString` is free!

#[cfg(test)]
mod tests {
    use obj::TclObj;

    #[test]
    fn should_share_on_clone() {
        let obj1 = TclObj::new().unwrap();
        assert!(!obj1.is_shared());

        {
            let obj2 = obj1.clone();
            assert!(obj1.is_shared());
            assert!(obj2.is_shared());
            assert_eq!(obj1.obj_ptr, obj2.obj_ptr);
        }

        assert!(!obj1.is_shared());
    }
}
