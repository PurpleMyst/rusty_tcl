//! A module that contains the [`TclObj`](struct.TclObj.html).
use super::rusty_tcl_sys;

use std::ptr::NonNull;

/// A struct representing the `Tcl_Obj` type.
#[derive(PartialEq, Eq, Debug)]
pub struct TclObj {
    obj_ptr: NonNull<rusty_tcl_sys::Tcl_Obj>,
    is_alive: bool,
}

impl Clone for TclObj {
    fn clone(&self) -> Self {
        let mut child = Self {
            obj_ptr: self.obj_ptr,
            is_alive: self.is_alive,
        };
        if self.is_alive {
            child.incr_ref_count();
        }
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
    /// This returns `None` when the pointer retruned by `Tcl_NewObj` is NULL.
    pub fn new() -> Option<Self> {
        super::init();
        let obj_ptr = unsafe { rusty_tcl_sys::Tcl_NewObj() };
        Self::from_ptr(obj_ptr)
    }

    pub(crate) fn from_ptr(obj_ptr: *mut rusty_tcl_sys::Tcl_Obj) -> Option<Self> {
        super::init();
        let mut this = Self {
            obj_ptr: NonNull::new(obj_ptr)?,
            is_alive: true,
        };
        this.incr_ref_count();
        Some(this)
    }

    /// Increments this object's reference count.
    // TODO: Use the `Tcl_IncrRefCount` macro here instead.
    pub fn incr_ref_count(&mut self) {
        assert!(self.is_alive);

        unsafe {
            let obj_ptr = self.obj_ptr.as_ptr();
            // TODO: Do we need ::std::ptr::read_unaligned?
            let mut tcl_obj = ::std::ptr::read_unaligned(obj_ptr);
            tcl_obj.refCount += 1;
            ::std::ptr::write_unaligned(obj_ptr, tcl_obj);
        }
    }

    /// Decrements this object's reference count and frees this struct's memory if the reference
    /// count is now zero.
    ///
    /// Returns `true` if the underlying struct was freed.
    pub fn decr_ref_count(&mut self) -> bool {
        assert!(self.is_alive);

        let should_free = unsafe {
            let obj_ptr = self.obj_ptr.as_ptr();
            let mut tcl_obj = ::std::ptr::read_unaligned(obj_ptr);
            tcl_obj.refCount -= 1;
            ::std::ptr::write_unaligned(obj_ptr, tcl_obj);
            tcl_obj.refCount <= 0
        };

        if should_free {
            unsafe {
                rusty_tcl_sys::TclFreeObj(self.obj_ptr.as_ptr());
            }
            self.is_alive = false;
            true
        } else {
            false
        }
    }

    /// Returns `true` if there are multiple [`TclObj`]s that point to the same underlying struct.
    pub fn is_shared(&self) -> bool {
        assert!(self.is_alive);
        let obj_struct = unsafe { *(self.obj_ptr.as_ptr()) };
        obj_struct.refCount > 1
    }
}

#[cfg(test)]
mod tests {
    use obj::TclObj;

    #[test]
    fn should_share_on_clone() {
        let obj1 = TclObj::new().unwrap();

        {
            let obj2 = obj1.clone();
            assert!(obj1.is_shared());
            assert!(obj2.is_shared());
            assert_eq!(obj1.obj_ptr, obj2.obj_ptr);
        }

        assert!(!obj1.is_shared());
    }
}
