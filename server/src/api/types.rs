use core::fmt::{self, Display};
use serde::{Serialize, Deserialize};
use std::io;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum APIError {
    MalformedRequest,
    DatabaseError,
    RecordNotFound,
    ParametersOutOfBounds,
    ConnectionError,
}

impl Display for APIError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            APIError::DatabaseError => formatter.write_str("Internal database service error"),
            APIError::MalformedRequest => formatter.write_str("Request deserialization error"),
            APIError::RecordNotFound => formatter.write_str("Requested resource does not exists"),
            APIError::ParametersOutOfBounds => formatter.write_str("Request parameters out of bounds"),
            APIError::ConnectionError => formatter.write_str("Socket connection is down"),
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

impl From<database::Error> for APIError {
    fn from(_: database::Error) -> Self {
        APIError::DatabaseError
    }
}

impl From<io::Error> for APIError {
    fn from(_: io::Error) -> Self {
        APIError::ConnectionError
    }
}
