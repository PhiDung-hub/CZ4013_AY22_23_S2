pub mod de;
pub mod json;
pub mod ser;

mod ignore;
mod non_unique_box;
mod types;

#[path = "macro_export.rs"]
pub mod __private;

pub struct StreamSerializer;

pub mod lifetime;

pub use derive_macro::*;

pub use crate::de::Deserialize;
pub use crate::ser::Serialize;
pub use crate::types::{Error, Result, Place};

