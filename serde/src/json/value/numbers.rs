use crate::de::{Deserialize, Visitor};
use crate::ser::{Fragment, Serialize};
use crate::{Place, Result};

use core::fmt::{self, Display};

/// A JSON number represented by some Rust primitive.
#[derive(Clone, Debug)]
pub enum Number {
    U64(u64),
    I64(i64),
    F64(f64),
}

impl Display for Number {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Number::U64(n) => formatter.write_str(&n.to_string()),
            Number::I64(i) => formatter.write_str(&i.to_string()),
            Number::F64(f) => formatter.write_str(&f.to_string()),
        }
    }
}

impl Serialize for Number {
    fn begin(&self) -> Fragment {
        match self {
            Number::U64(n) => Fragment::U64(*n),
            Number::I64(i) => Fragment::I64(*i),
            Number::F64(f) => Fragment::F64(*f),
        }
    }
}

impl Deserialize for Number {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl Visitor for Place<Number> {
            fn negative(&mut self, i: i64) -> Result<()> {
                self.out = Some(Number::I64(i));
                Ok(())
            }

            fn nonnegative(&mut self, n: u64) -> Result<()> {
                self.out = Some(Number::U64(n));
                Ok(())
            }

            fn float(&mut self, f: f64) -> Result<()> {
                self.out = Some(Number::F64(f));
                Ok(())
            }
        }

        Place::new(out)
    }
}
