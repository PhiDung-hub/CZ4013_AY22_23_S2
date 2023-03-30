pub mod impls;

use super::{Error, Result};
use std::boxed::Box;

pub trait Deserialize: Sized {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor;

    /// For Option<T>
    #[inline]
    fn default() -> Option<Self> {
        None
    }
}

/// Trait that write data into an output place
pub trait Visitor {
    fn null(&mut self) -> Result<()> {
        Err(Error)
    }

    fn boolean(&mut self, _: bool) -> Result<()> {
        Err(Error)
    }

    fn string(&mut self, _: &str) -> Result<()> {
        Err(Error)
    }

    fn negative(&mut self, _: i64) -> Result<()> {
        Err(Error)
    }

    fn nonnegative(&mut self, _: u64) -> Result<()> {
        Err(Error)
    }

    fn float(&mut self, _: f64) -> Result<()> {
        Err(Error)
    }

    fn seq(&mut self) -> Result<Box<dyn Seq + '_>> {
        Err(Error)
    }

    fn map(&mut self) -> Result<Box<dyn Map + '_>> {
        Err(Error)
    }
}

/// Trait that can hand out places to write sequence elements.
pub trait Seq {
    fn element(&mut self) -> Result<&mut dyn Visitor>;
    fn finish(&mut self) -> Result<()>;
}

/// Trait that can hand out places to write values of a map.
pub trait Map {
    fn key(&mut self, k: &str) -> Result<&mut dyn Visitor>;
    fn finish(&mut self) -> Result<()>;
}
