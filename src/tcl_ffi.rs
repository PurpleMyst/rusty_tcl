use std::os::raw::{c_char, c_int};

pub(crate) const TCL_OK: c_int = 0;
pub(crate) const TCL_ERROR: c_int = 1;
pub(crate) const TCL_RETURN: c_int = 2;
pub(crate) const TCL_BREAK: c_int = 3;
pub(crate) const TCL_CONTINUE: c_int = 4;

pub(crate) const TCL_LEAVE_ERR_MSG: c_int = 0x0200;

#[allow(non_camel_case_types)]
pub(crate) type Tcl_FreeProc = extern "C" fn(*mut c_char);

#[allow(non_snake_case, non_camel_case_types)]
#[repr(C)]
pub(crate) struct Tcl_Interp {
    result: *mut c_char,
    freeProc: *mut Tcl_FreeProc,
    errorLine: c_int,
}

#[allow(non_snake_case, non_camel_case_types)]
#[link(name = "tcl8.5")]
extern "C" {
    pub(crate) fn Tcl_CreateInterp() -> *mut Tcl_Interp;
    pub(crate) fn Tcl_DeleteInterp(interp: *mut Tcl_Interp);

    pub(crate) fn Tcl_Eval(interp: *mut Tcl_Interp, script: *const c_char) -> c_int;

    pub(crate) fn Tcl_GetStringResult(interp: *mut Tcl_Interp) -> *const c_char;

    pub(crate) fn Tcl_SetVar(interp: *mut Tcl_Interp, varName: *const c_char, newValue: *const c_char, flags: c_int) -> *const c_char;

    pub(crate) fn Tcl_Init(interp: *mut Tcl_Interp) -> c_int;
}
