pub mod completion_code;
pub mod interp;
mod tcl_ffi;

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use {completion_code, interp};

    #[test]
    fn it_works() {
        let mut interp = interp::TclInterp::new().unwrap();
        assert_eq!(
            interp.eval(CString::new("expr {2 + 2}").unwrap()),
            completion_code::CompletionCode::Ok
        );
        assert_eq!(
            interp.get_string_result(),
            CString::new("4").unwrap().as_ref()
        );
    }
}
