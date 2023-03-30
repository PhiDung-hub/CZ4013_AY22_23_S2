use super::{Deserialize, Map, Seq, Visitor};
use crate::Place;
use crate::{Error, Result};

use core::mem;
use core::str;
use std::boxed::Box;

impl Deserialize for () {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl Visitor for Place<()> {
            fn null(&mut self) -> Result<()> {
                self.out = Some(());
                Ok(())
            }
        }
        Place::new(out)
    }
}

impl Deserialize for bool {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl Visitor for Place<bool> {
            fn boolean(&mut self, b: bool) -> Result<()> {
                self.out = Some(b);
                Ok(())
            }
        }
        Place::new(out)
    }
}

impl Deserialize for String {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl Visitor for Place<String> {
            fn string(&mut self, s: &str) -> Result<()> {
                self.out = Some(s.to_owned());
                Ok(())
            }
        }
        Place::new(out)
    }
}

macro_rules! signed {
    ($ty:ident) => {
        impl Deserialize for $ty {
            fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
                impl Visitor for Place<$ty> {
                    fn negative(&mut self, n: i64) -> Result<()> {
                        if n >= $ty::min_value() as i64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(Error)
                        }
                    }

                    fn nonnegative(&mut self, n: u64) -> Result<()> {
                        if n <= $ty::max_value() as u64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(Error)
                        }
                    }
                }
                Place::new(out)
            }
        }
    };
}
signed!(i8);
signed!(i16);
signed!(i32);
signed!(i64);
signed!(isize);

macro_rules! unsigned {
    ($ty:ident) => {
        impl Deserialize for $ty {
            fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
                impl Visitor for Place<$ty> {
                    fn nonnegative(&mut self, n: u64) -> Result<()> {
                        if n <= $ty::max_value() as u64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(Error)
                        }
                    }
                }
                Place::new(out)
            }
        }
    };
}
unsigned!(u8);
unsigned!(u16);
unsigned!(u32);
unsigned!(u64);
unsigned!(usize);

macro_rules! float {
    ($ty:ident) => {
        impl Deserialize for $ty {
            fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
                impl Visitor for Place<$ty> {
                    fn negative(&mut self, n: i64) -> Result<()> {
                        self.out = Some(n as $ty);
                        Ok(())
                    }

                    fn nonnegative(&mut self, n: u64) -> Result<()> {
                        self.out = Some(n as $ty);
                        Ok(())
                    }

                    fn float(&mut self, n: f64) -> Result<()> {
                        self.out = Some(n as $ty);
                        Ok(())
                    }
                }
                Place::new(out)
            }
        }
    };
}
float!(f32);
float!(f64);

impl<T: Deserialize> Deserialize for Option<T> {
    #[inline]
    fn default() -> Option<Self> {
        Some(None)
    }
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl<T: Deserialize> Visitor for Place<Option<T>> {
            fn null(&mut self) -> Result<()> {
                self.out = Some(None);
                Ok(())
            }

            fn boolean(&mut self, b: bool) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).boolean(b)
            }

            fn string(&mut self, s: &str) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).string(s)
            }

            fn negative(&mut self, n: i64) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).negative(n)
            }

            fn nonnegative(&mut self, n: u64) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).nonnegative(n)
            }

            fn float(&mut self, n: f64) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).float(n)
            }

            fn seq(&mut self) -> Result<Box<dyn Seq + '_>> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).seq()
            }

            fn map(&mut self) -> Result<Box<dyn Map + '_>> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).map()
            }
        }

        Place::new(out)
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl<T: Deserialize> Visitor for Place<Vec<T>> {
            fn seq(&mut self) -> Result<Box<dyn Seq + '_>> {
                Ok(Box::new(VecBuilder {
                    out: &mut self.out,
                    vec: Vec::new(),
                    element: None,
                }))
            }
        }

        struct VecBuilder<'a, T: 'a> {
            out: &'a mut Option<Vec<T>>,
            vec: Vec<T>,
            element: Option<T>,
        }

        impl<'a, T> VecBuilder<'a, T> {
            fn shift(&mut self) {
                if let Some(e) = self.element.take() {
                    self.vec.push(e);
                }
            }
        }

        impl<'a, T: Deserialize> Seq for VecBuilder<'a, T> {
            fn element(&mut self) -> Result<&mut dyn Visitor> {
                self.shift();
                Ok(Deserialize::begin(&mut self.element))
            }

            fn finish(&mut self) -> Result<()> {
                self.shift();
                *self.out = Some(mem::take(&mut self.vec));
                Ok(())
            }
        }

        Place::new(out)
    }
}
