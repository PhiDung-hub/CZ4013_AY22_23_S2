//! Export types from std to use for proc macros derives.
//! Alias `str` and `usize`
pub use std::borrow::Cow;
pub use std::boxed::Box;
pub use std::string::String;
pub use core::option::Option::{self, None, Some};
pub use core::result::Result::{Err, Ok};

pub use self::help::Str as str;
pub use self::help::Usize as usize;

mod help {
    pub type Str = str;
    pub type Usize = usize;
}
