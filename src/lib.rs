//! A simple crate that allow to use the Tcl C library in a more rustic way.
#![warn(missing_docs)]

extern crate rusty_tcl_sys;

pub mod completion_code;
pub mod interp;

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
    }
}
