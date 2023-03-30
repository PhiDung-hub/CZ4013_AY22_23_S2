use serde::Result;
use serde::{Deserialize, Serialize};

/// this trait decodes `body` from type Vec<u8> to rust types.
pub trait DecodeBody {
    fn decode_body<D: Deserialize>(&self) -> Result<D>;
}

/// this trait encodes an body into bytes.
// #[async_trait(?Send)]
pub trait EncodeBody {
    fn encode_body<S: Serialize>(&mut self, body: S);
}
