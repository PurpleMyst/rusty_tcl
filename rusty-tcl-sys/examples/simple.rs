extern crate rusty_tcl_sys;

use std::os::raw::c_int;

fn main() {
    unsafe {
        let interp_ptr = rusty_tcl_sys::Tcl_CreateInterp();
        assert_eq!(
            rusty_tcl_sys::Tcl_MakeSafe(interp_ptr),
            rusty_tcl_sys::TCL_OK as c_int
        );
        assert_eq!(
            rusty_tcl_sys::Tcl_IsSafe(interp_ptr),
            rusty_tcl_sys::TCL_OK as c_int
        );
        rusty_tcl_sys::Tcl_DeleteInterp(interp_ptr);
    }
}
