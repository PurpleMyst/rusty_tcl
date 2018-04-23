//! A module that contains the [`CompletionCode`](enum.CompletionCode.html).

use std::borrow::Cow;

/// Enum that represents Tcl's usually-ints completion codes in a more rustic manner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompletionCode<'interp> {
    /// Represents that everything went fine.
    Ok,

    /// Represents that there was an error.
    /// Contains the error message left as the result.
    Error(Cow<'interp, str>),

    /// Represents that the last line executed was a result.
    Return,

    /// Represents that the last line executed was a break.
    Break,

    /// Represents that the last line executed was a continue.
    Continue,
}

impl<'interp> CompletionCode<'interp> {
    /// Panics if the `self` is a [`CompletionCode::Error`], else does nothing.
    #[inline]
    pub fn panic_if_error(self) {
        if let CompletionCode::Error(msg) = self {
            panic!("{}", msg);
        }
    }
}
