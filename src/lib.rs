pub mod completion_code;
pub mod interp;
mod tcl_ffi;

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use {completion_code, interp};

    macro_rules! cstr {
        ($s:expr) => {
            CString::new($s).unwrap()
        }
    }

    #[test]
    fn it_works() {
        let mut interp = interp::TclInterp::new().unwrap();

        macro_rules! tcl_assert_eq {
            ($cc:expr, $expected:expr) => {{
                assert_eq!($cc, completion_code::CompletionCode::Ok);
                assert_eq!(interp.get_string_result(), cstr!($expected).as_ref());
            }}
        };

        tcl_assert_eq!(interp.eval(cstr!("expr {2 + 2}")), "4");
    }
}
