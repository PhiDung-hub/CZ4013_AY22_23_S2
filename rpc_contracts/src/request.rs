use lazy_static::lazy_static;
use std::str;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::body::{DecodeBody, EncodeBody};
use serde::json;
use serde::{Deserialize, Error, Result, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RPCRequest {
    pub id: u32,
    pub service_type: u8,
    pub body: Vec<u8>,
}

lazy_static! {
    static ref ID_COUNTER: Arc<Mutex<AtomicU32>> = Arc::new(Mutex::new(AtomicU32::new(0)));
}

pub async fn get_request_id() -> u32 {
    let counter = ID_COUNTER.lock().await;
    let id = counter.fetch_add(1, Ordering::SeqCst);
    id
}

impl RPCRequest {
    #[allow(dead_code)]
    pub async fn new(service_type: u8) -> Self {
        let id = get_request_id().await;
        RPCRequest {
            id,
            service_type,
            body: Vec::new(),
        }
    }
}

impl DecodeBody for RPCRequest {
    fn decode_body<D: Deserialize>(&self) -> Result<D> {
        let body_string = match str::from_utf8(&self.body) {
            Ok(str) => str,
            Err(_) => return Err(Error),
        };
        let decoded_body: D = json::from_str(body_string)?;
        Ok(decoded_body)
    }
}

impl EncodeBody for RPCRequest {
    fn encode_body<S: Serialize>(&mut self, body: S) {
        let body_string = json::to_string(&body);
        self.body = body_string.into_bytes();
    }
}
