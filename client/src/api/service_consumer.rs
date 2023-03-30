use std::{io, str};
use tokio::net::UdpSocket;
use tokio::time::{sleep, timeout, Duration};

use super::types::{APIError, Result};
use rpc_contracts::body::contracts::*;
use rpc_contracts::{DecodeBody, EncodeBody, RPCRequest, RPCResponse, ResponseStatus};
use serde::json;

const TIME_OUT: Duration = Duration::from_secs(5);
const RESPONSE_SIZE: usize = 4096; // limit response size to be 4 bytes

pub struct ServiceConsumer<'a> {
    pub socket: &'a UdpSocket,
    pub host_addr: String,
    pub port_addr: u16,
    pub retry: bool,
}

macro_rules! match_consumer_response {
    ($timeout_future:expr, $request:ident, $buffer:ident, $time_elapsed:ident, $service_type:ident, $body_struct:ty, $($field:ident),+) => {
        match $timeout_future.await {
            Ok(Ok((size, _))) => {
                let str_response = str::from_utf8(&$buffer[..size]).unwrap();
                let response: RPCResponse = json::from_str(str_response)?;
                if $request.id == response.request_id {
                    match response.status {
                        ResponseStatus::Finished => {
                            let body = response.decode_body::<$body_struct>()?;
                            $(let $field = body.$field;)+
                            println!("{} response with the following details:", $service_type);
                            $(println!("{} = {:?}", stringify!($field), $field);)+
                        }
                        ResponseStatus::Failed => {
                            let ServiceFailedResponse { error } = response.decode_body()?;
                            println!("{} response with error: {}", $service_type, error.to_string());
                        }
                        ResponseStatus::Updated => {
                            println!("{} received wrong response, unexpeceted status Update", $service_type);
                        }
                    }
                    return Ok(response);
                }
            }
            Ok(Err(_)) => {
                println!("{} communication failed: {:?}", $service_type, APIError::IOFailed);
                return Err(APIError::IOFailed);
            }
            Err(_) => {
                $time_elapsed += Duration::from_secs(1);
                if $time_elapsed > TIME_OUT {
                    println!("{} response time out: {:?}", $service_type, APIError::TimeOutError);
                    return Err(APIError::TimeOutError);
                }
            }
        }
    };
}

impl<'a> ServiceConsumer<'a> {
    pub fn new(socket: &'a UdpSocket, host_addr: String, port_addr: u16, retry: bool) -> Self {
        ServiceConsumer {
            socket,
            host_addr,
            port_addr,
            retry,
        }
    }

    pub async fn send_package(&self, encoded_message: &[u8]) -> io::Result<usize> {
        self.socket
            .send_to(encoded_message, (self.host_addr.clone(), self.port_addr))
            .await
    }

    pub async fn invoke_request_service_1(&self, source: String, destination: String) {
        let mut response = self
            .request_service_1(source.clone(), destination.clone())
            .await;
        while self.retry && response.err() == Some(APIError::TimeOutError) {
            println!("Request timeout, retrying");
            response = self
                .request_service_1(source.clone(), destination.clone())
                .await;
        }
    }
    pub async fn request_service_1(
        &self,
        source: String,
        destination: String,
    ) -> Result<RPCResponse> {
        let mut request = RPCRequest::new(1).await;
        request.encode_body(Service1RequestBody {
            source,
            destination,
        });

        let encoded_request = json::to_string(&request);
        self.send_package(encoded_request.as_bytes()).await?;

        let mut buffer = [0_u8; RESPONSE_SIZE];
        let mut time_elapsed = Duration::from_secs(0);
        loop {
            // TODO: turn this into a macro
            let recv_future = self.socket.recv_from(&mut buffer);
            let timeout_future = timeout(Duration::from_secs(1), recv_future);
            let service = "Service 1";
            match_consumer_response!(
                timeout_future,
                request,
                buffer,
                time_elapsed,
                service,
                Service1ResponseBody,
                flight_ids
            );
        }
    }

    pub async fn invoke_request_service_2(&self, flight_id: u32) {
        let mut response = self.request_service_2(flight_id).await;
        while self.retry && response.err() == Some(APIError::TimeOutError) {
            println!("Request timeout, retrying");
            response = self.request_service_2(flight_id).await;
        }
    }
    pub async fn request_service_2(&self, flight_id: u32) -> Result<RPCResponse> {
        let mut request = RPCRequest::new(2).await;
        request.encode_body(Service2RequestBody { flight_id });
        let encoded_request = json::to_string(&request);
        self.send_package(encoded_request.as_bytes()).await?;

        let mut buffer = [0_u8; RESPONSE_SIZE];
        let mut time_elapsed = Duration::from_secs(0);
        loop {
            let recv_future = self.socket.recv_from(&mut buffer);
            let timeout_future = timeout(Duration::from_secs(1), recv_future);
            let service = "Service 2";
            match_consumer_response!(
                timeout_future,
                request,
                buffer,
                time_elapsed,
                service,
                Service2ResponseBody,
                departure_time,
                airfare,
                seat_avail
            );
        }
    }

    pub async fn invoke_request_service_3(&self, flight_id: u32, num_seat: u32) {
        let mut response = self.request_service_3(flight_id, num_seat).await;
        while self.retry && response.err() == Some(APIError::TimeOutError) {
            println!("Request timeout, retrying");
            response = self.request_service_3(flight_id, num_seat).await;
        }
    }
    pub async fn request_service_3(&self, flight_id: u32, num_seat: u32) -> Result<RPCResponse> {
        let mut request = RPCRequest::new(3).await;
        request.encode_body(Service3RequestBody {
            flight_id,
            num_seat,
        });
        let encoded_request = json::to_string(&request);
        self.send_package(encoded_request.as_bytes()).await?;

        let mut buffer = [0_u8; RESPONSE_SIZE];
        let mut time_elapsed = Duration::from_secs(0);
        loop {
            let recv_future = self.socket.recv_from(&mut buffer);
            let timeout_future = timeout(Duration::from_secs(1), recv_future);
            let service = "Service 3";
            match_consumer_response!(
                timeout_future,
                request,
                buffer,
                time_elapsed,
                service,
                Service3ResponseBody,
                message
            );
        }
    }

    pub async fn request_service_4(&self, flight_id: u32, monitor_interval: u32) {
        let mut request = RPCRequest::new(4).await;
        request.encode_body(Service4RequestBody {
            flight_id,
            monitor_interval,
        });
        let encoded_request = json::to_string(&request);
        self.send_package(encoded_request.as_bytes()).await.ok();

        let mut buffer = [0_u8; RESPONSE_SIZE];
        let service_type = "Service 4";
        let mut ack = false; // ack must be received from server.
        loop {
            let delay = sleep(Duration::from_secs(monitor_interval as u64));
            tokio::pin!(delay);

            let request_timeout = sleep(TIME_OUT);
            tokio::pin!(request_timeout);

            tokio::select! {
                _ = &mut request_timeout => {
                    if !ack {
                        println!("{} response time out: {:?}", service_type, APIError::TimeOutError);
                        if !self.retry {
                            break;
                        }
                        println!("Request timeout, retry enabled, keep waiting");
                    }
                }
                _ = &mut delay => {
                    println!("{} response ended monitoring period, time = {}s", service_type, monitor_interval);
                    break;
                }
                response_result = self.socket.recv_from(&mut buffer) => {
                    match response_result {
                        Ok((size, _)) => {
                            let str_response = str::from_utf8(&buffer[..size]).unwrap();
                            let response: RPCResponse = match json::from_str(str_response) {
                                Ok(response) => response,
                                Err(_) => {
                                    println!("{} receives malformed request format from server", service_type);
                                    continue;
                                }
                            };
                            // NOTE: will handle decode error in the future
                            match response.status {
                                ResponseStatus::Finished => {
                                    let Service4ResponseBody { message } = response.decode_body().unwrap();
                                    println!("{} responses with the following details:", service_type);
                                    println!("message = {:?}", message);
                                    ack = true;
                                }
                                ResponseStatus::Failed => {
                                    let ServiceFailedResponse { error } = response.decode_body().unwrap();
                                    println!("{} responses with error: {}", service_type, error.to_string());
                                    break;
                                }
                                ResponseStatus::Updated => {
                                    let Service4MonitorResponseBody { seat_avail } = response.decode_body().unwrap();
                                    println!("{} receives the following update:", service_type);
                                    println!("seat_avail = {:?}", seat_avail);
                                }
                            }
                        }
                        Err(_) => {
                            println!("{} communication failed: {:?}", service_type, APIError::IOFailed);
                            break;
                        }
                    }
                }
            }
        }
    }

    pub async fn invoke_request_service_5(&self, flight_id: u32) {
        let mut response = self.request_service_5(flight_id).await;
        while self.retry && response.err() == Some(APIError::TimeOutError) {
            println!("Request timeout, retrying");
            response = self.request_service_5(flight_id).await;
        }
    }
    pub async fn request_service_5(&self, flight_id: u32) -> Result<RPCResponse> {
        let mut request = RPCRequest::new(5).await;
        request.encode_body(Service5RequestBody { flight_id });
        let encoded_request = json::to_string(&request);
        self.send_package(encoded_request.as_bytes()).await?;

        let mut buffer = [0_u8; RESPONSE_SIZE];
        let mut time_elapsed = Duration::from_secs(0);
        loop {
            let recv_future = self.socket.recv_from(&mut buffer);
            let timeout_future = timeout(Duration::from_secs(1), recv_future);
            let service = "Service 5";
            match_consumer_response!(
                timeout_future,
                request,
                buffer,
                time_elapsed,
                service,
                Service5ResponseBody,
                message
            );
        }
    }

    pub async fn invoke_request_service_6(&self, flight_id: u32, amount_in_kg: u32) {
        let mut response = self.request_service_6(flight_id, amount_in_kg).await;
        while self.retry && response.err() == Some(APIError::TimeOutError) {
            println!("Request timeout, retrying");
            response = self.request_service_6(flight_id, amount_in_kg).await;
        }
    }
    pub async fn request_service_6(
        &self,
        flight_id: u32,
        amount_in_kg: u32,
    ) -> Result<RPCResponse> {
        let mut request = RPCRequest::new(6).await;
        request.encode_body(Service6RequestBody {
            flight_id,
            amount_in_kg,
        });
        let encoded_request = json::to_string(&request);
        self.send_package(encoded_request.as_bytes()).await?;

        let mut buffer = [0_u8; RESPONSE_SIZE];
        let mut time_elapsed = Duration::from_secs(0);
        loop {
            let recv_future = self.socket.recv_from(&mut buffer);
            let timeout_future = timeout(Duration::from_secs(1), recv_future);
            let service = "Service 6";
            match_consumer_response!(
                timeout_future,
                request,
                buffer,
                time_elapsed,
                service,
                Service6ResponseBody,
                message
            );
        }
    }
}
