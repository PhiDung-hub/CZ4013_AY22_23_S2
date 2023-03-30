use core::fmt::{self, Display};

#[derive(Copy, Clone, Debug)]
pub struct Error;

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("serde error")
    }
}

impl std::error::Error for Error {}

/// Result type returned by deserialization functions.
pub type Result<T> = core::result::Result<T, Error>;

/// Define a "Place" type, wrapper around deserialized type.
pub struct Place<T> {
    pub out: Option<T>,
}

impl<T> Place<T> {
    pub fn new(out: &mut Option<T>) -> &mut Self {
        unsafe { &mut *(out as *mut Option<T> as *mut Place<T>) }
    }
}
