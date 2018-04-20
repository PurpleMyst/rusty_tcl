use std::ffi::CStr;

#[derive(Debug, PartialEq, Eq)]
pub enum CompletionCode<'a> {
    Ok,
    Error(&'a CStr),
    Return,
    Break,
    Continue,
}
