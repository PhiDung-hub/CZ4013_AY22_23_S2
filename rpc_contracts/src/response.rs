use lazy_static::lazy_static;
use std::str;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::body::{DecodeBody, EncodeBody};
use crate::RPCRequest;
use serde::json;
use serde::{Deserialize, Error, Result, Serialize};

// NOTE: no requirement for request caching so reinitialize all request Id to 1 after each server session,
// can easily be improved in the future.
#[derive(Debug, Serialize, Deserialize)]
pub struct RPCResponse {
    id: u32,
    pub request_id: u32,
    pub status: ResponseStatus,
    pub body: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseStatus {
    Finished,
    Updated,
    Failed,
}

lazy_static! {
    static ref ID_COUNTER: Arc<Mutex<AtomicU32>> = Arc::new(Mutex::new(AtomicU32::new(0)));
}

pub async fn get_response_id() -> u32 {
    let counter = ID_COUNTER.lock().await;
    let id = counter.fetch_add(1, Ordering::SeqCst);
    id
}

impl RPCResponse {
    pub async fn finished(request_id: u32) -> Self {
        let id = get_response_id().await;
        RPCResponse {
            id,
            request_id,
            status: ResponseStatus::Finished,
            body: Vec::new(),
        }
    }

    pub async fn failed(request_id: u32) -> Self {
        let id = get_response_id().await;
        RPCResponse {
            id,
            request_id,
            status: ResponseStatus::Failed,
            body: Vec::new(),
        }
    }

    pub async fn updated(request_id: u32) -> Self {
        let id = get_response_id().await;
        RPCResponse {
            id,
            request_id,
            status: ResponseStatus::Updated,
            body: Vec::new(),
        }
    }

    pub async fn failed_invalid_service_type(request: RPCRequest) -> Self {
        let id = get_response_id().await;
        let message = "Invalid service type";
        let mut response = RPCResponse {
            id,
            request_id: request.id,
            status: ResponseStatus::Failed,
            body: Vec::new(),
        };

        response.encode_body(message);
        response
    }
}

impl EncodeBody for RPCResponse {
    fn encode_body<S: Serialize>(&mut self, body: S) {
        let body_string = json::to_string(&body);
        self.body = body_string.into_bytes();
    }
}

impl DecodeBody for RPCResponse {
    fn decode_body<D: Deserialize>(&self) -> Result<D> {
        let body_string = match str::from_utf8(&self.body) {
            Ok(str) => str,
            Err(_) => return Err(Error),
        };
        let decoded_body: D = json::from_str(body_string)?;
        Ok(decoded_body)
    }
}
