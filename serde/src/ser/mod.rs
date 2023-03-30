mod impls;

use std::borrow::Cow;
use std::boxed::Box;

pub enum Fragment<'a> {
    Null,
    Bool(bool),
    Str(Cow<'a, str>),
    U64(u64),
    I64(i64),
    F64(f64),
    Seq(Box<dyn Seq + 'a>),
    Map(Box<dyn Map + 'a>),
}

/// Trait for data structures that can be serialized to a JSON string.
pub trait Serialize {
    fn begin(&self) -> Fragment;
}

/// Trait that can iterate elements of a sequence.
pub trait Seq {
    fn next(&mut self) -> Option<&dyn Serialize>;
}

/// Trait that can iterate key-value entries of a map or struct.
pub trait Map {
    fn next(&mut self) -> Option<(Cow<str>, &dyn Serialize)>;
}

