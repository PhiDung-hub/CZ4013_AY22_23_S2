use core::fmt::{self, Display};
use serde::{Deserialize, Serialize};
use std::io;

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum APIError {
    MalformedRequest,
    RecordNotFound,
    ParametersOutOfBounds,
    IOFailed,
    TimeOutError
}

impl Display for APIError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            APIError::MalformedRequest => formatter.write_str("Request deserialization error"),
            APIError::RecordNotFound => formatter.write_str("Requested resource does not exists"),
            APIError::ParametersOutOfBounds => {
                formatter.write_str("Request parameters out of bounds")
            }
            APIError::IOFailed => formatter.write_str("Failed to get response from server"),
            APIError::TimeOutError => formatter.write_str("Request time out, message might be loss"),
        }
    }
}
impl std::error::Error for APIError {}

pub type Result<T> = core::result::Result<T, APIError>;

impl From<serde::Error> for APIError {
    fn from(_: serde::Error) -> Self {
        APIError::MalformedRequest
    }
}

impl From<io::Error> for APIError {
    fn from(_: io::Error) -> Self {
        APIError::MalformedRequest
    }
}


