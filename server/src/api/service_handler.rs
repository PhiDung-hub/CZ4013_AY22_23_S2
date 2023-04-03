use super::{APIError, Result};
use database::types::{BuyLuggageStatus, CancellationStatus, ReservationStatus};
use database::DatabaseService;
use rpc_contracts::body::contracts::*;
use rpc_contracts::{DecodeBody, EncodeBody, RPCRequest, RPCResponse};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio::time::{sleep, Duration};

pub struct ServiceHandler<'a> {
    pub socket: Arc<UdpSocket>,
    pub client_addr: String,
    pub db_service: Arc<Mutex<DatabaseService<'a>>>,
    sender: &'static Sender<(u32, ReservationStatus)>,
    receiver: &'static mut Receiver<(u32, ReservationStatus)>,
    notify: &'static Notify,
}

macro_rules! use_internal_handler {
    ($handler:ident, $internal_service:ident, $_req:ident) => {
        match $handler.$internal_service(&$_req).await {
            Ok(response) => response,
            Err(_e) => {
                let mut response = RPCResponse::failed($_req.id).await;
                response.encode_body(ServiceFailedResponse { error: _e.to_string() }); // encode error message in response
                response
            }
        }
    };
}

impl<'a> ServiceHandler<'a> {
    pub async fn new(
        socket: Arc<UdpSocket>, client_addr: String, db_service: Arc<Mutex<DatabaseService<'a>>>, sender: &'static Sender<(u32, ReservationStatus)>,
        receiver: &'static mut Receiver<(u32, ReservationStatus)>, notify: &'static Notify,
    ) -> ServiceHandler<'a> {
        ServiceHandler {
            socket,
            client_addr,
            db_service,
            sender,
            receiver,
            notify,
        }
    }

    pub async fn invalid_service_type(&self, _req: RPCRequest) {
        let response = RPCResponse::failed_invalid_service_type(_req).await;
        let response_string = serde::json::to_string(&response);
        self.socket
            .clone()
            .send_to(response_string.as_bytes(), self.client_addr.clone())
            .await
            .ok();
    }

    pub async fn service_1(&self, _req: RPCRequest) {
        let response = use_internal_handler!(self, internal_service_1, _req);
        let response_string = serde::json::to_string(&response);
        self.socket.send_to(response_string.as_bytes(), &self.client_addr).await.unwrap();
    }
    async fn internal_service_1(&self, _req: &RPCRequest) -> Result<RPCResponse> {
        let Service1RequestBody { source, destination } = _req.decode_body()?;

        let db_service = self.db_service.lock().await;
        let flight_ids: Vec<u32> = db_service.get_flight_ids(source, destination).await?;

        let mut response = RPCResponse::finished(_req.id).await;
        response.encode_body(Service1ResponseBody { flight_ids });

        Ok(response)
    }

    pub async fn service_2(&self, _req: RPCRequest) {
        let response = use_internal_handler!(self, internal_service_2, _req);
        let response_string = serde::json::to_string(&response);
        self.socket.send_to(response_string.as_bytes(), &self.client_addr).await.unwrap();
    }
    async fn internal_service_2(&self, _req: &RPCRequest) -> Result<RPCResponse> {
        let Service2RequestBody { flight_id } = _req.decode_body()?;

        let db_service = self.db_service.lock().await;

        let result_flight_info: Option<(i32, f32, u32)> = db_service.get_flight_info(flight_id).await?;
        match result_flight_info {
            Some(flight_info) => {
                let (departure_time, airfare, seat_avail) = flight_info;
                let mut response = RPCResponse::finished(_req.id).await;
                response.encode_body(Service2ResponseBody {
                    departure_time,
                    airfare,
                    seat_avail,
                });

                Ok(response)
            }
            None => Err(APIError::RecordNotFound),
        }
    }

    pub async fn service_3(&self, _req: RPCRequest) {
        let response = use_internal_handler!(self, internal_service_3, _req);
        let response_string = serde::json::to_string(&response);
        self.socket.send_to(response_string.as_bytes(), &self.client_addr).await.unwrap();
    }
    async fn internal_service_3(&self, _req: &RPCRequest) -> Result<RPCResponse> {
        let Service3RequestBody { flight_id, num_seat } = _req.decode_body()?;

        let db_service = self.db_service.lock().await;
        let reservation_status = db_service.make_reservation(flight_id, self.client_addr.clone(), num_seat).await?;

        match reservation_status {
            status @ (ReservationStatus::Created | ReservationStatus::Updated) => {
                let message = status.to_string();
                let mut success_response = RPCResponse::finished(_req.id).await;
                success_response.encode_body(Service3ResponseBody { message });
                // NOTE: send update for service 4
                self.send_update(flight_id, status).await;
                Ok(success_response)
            }
            ReservationStatus::InvalidFlightID => Err(APIError::RecordNotFound),
            ReservationStatus::ZeroSeatReserved => Err(APIError::ParametersOutOfBounds),
            ReservationStatus::InsufficientCapacity => Err(APIError::ParametersOutOfBounds),
        }
    }

    pub async fn send_update(&self, flight_id: u32, status: ReservationStatus) {
        self.sender.send((flight_id, status)).await.ok();
        self.notify.notify_one();
    }

    pub async fn service_4(&mut self, _req: RPCRequest) {
        async fn send_response(socket: Arc<UdpSocket>, response: RPCResponse, client_addr: &String) {
            let response_string = serde::json::to_string(&response);
            socket.send_to(response_string.as_bytes(), client_addr).await.ok();
        }
        async fn send_error(socket: Arc<UdpSocket>, error: String, request_id: u32, client_addr: &String) {
            let mut response = RPCResponse::failed(request_id).await;
            response.encode_body(ServiceFailedResponse { error }); // encode error message in response
            send_response(socket, response, client_addr).await;
        }

        let Service4RequestBody { flight_id, monitor_interval } = match _req.decode_body() {
            Ok(body) => body,
            Err(_e) => {
                send_error(self.socket.clone(), _e.to_string(), _req.id, &self.client_addr).await;
                return;
            }
        };

        match self.db_service.lock().await.is_flight_exists(flight_id).await {
            Ok(true) => {}
            Ok(false) => {
                let _e = APIError::RecordNotFound;
                send_error(self.socket.clone(), _e.to_string(), _req.id, &self.client_addr).await;
                return;
            }
            Err(_e) => {
                send_error(self.socket.clone(), _e.to_string(), _req.id, &self.client_addr).await;
                return;
            }
        }

        let mut finished_response = RPCResponse::finished(_req.id).await;
        finished_response.encode_body(Service4ResponseBody {
            message: "Monitor service successfully established".to_string(),
        });
        send_response(self.socket.clone(), finished_response, &self.client_addr).await;

        let monitor_interval = monitor_interval as u64;
        loop {
            let delay = sleep(Duration::from_secs(monitor_interval));
            tokio::pin!(delay);

            let received = tokio::select! {
                received = self.recv_update() => received,
                _ = &mut delay => break,
            };
            match received {
                Some((id, _)) if flight_id != id => continue,
                Some((_, _)) => {
                    let db_service = self.db_service.lock().await;
                    let (_, _, seat_avail) = db_service.get_flight_info(flight_id).await.unwrap().unwrap();
                    let mut update_response = RPCResponse::updated(_req.id).await;
                    update_response.encode_body(Service4MonitorResponseBody { seat_avail });
                    send_response(self.socket.clone(), update_response, &self.client_addr).await;
                }
                None => {
                    println!("Monitor channel closed!");
                    break;
                }
            }
        }
    }

    pub async fn recv_update(&mut self) -> Option<(u32, ReservationStatus)> {
        self.notify.notified().await;
        self.receiver.try_recv().ok()
    }

    pub async fn service_5(&self, _req: RPCRequest) {
        let response = use_internal_handler!(self, internal_service_5, _req);
        let response_string = serde::json::to_string(&response);
        self.socket.send_to(response_string.as_bytes(), &self.client_addr).await.unwrap();
    }
    async fn internal_service_5(&self, _req: &RPCRequest) -> Result<RPCResponse> {
        let db_service = self.db_service.lock().await;
        let Service5RequestBody { flight_id } = _req.decode_body()?;
        let client_addr = self.client_addr.clone();

        let cancellation_status = db_service.cancel_reservation(flight_id, client_addr).await?;

        match cancellation_status {
            status @ CancellationStatus::Success => {
                let message = status.to_string();
                let mut success_response = RPCResponse::finished(_req.id).await;
                success_response.encode_body(Service5ResponseBody { message });
                self.send_update(flight_id, ReservationStatus::Updated).await;
                Ok(success_response)
            }
            CancellationStatus::ReservationNotExisted => Err(APIError::RecordNotFound),
        }
    }

    pub async fn service_6(&self, _req: RPCRequest) {
        let response = use_internal_handler!(self, internal_service_6, _req);
        let response_string = serde::json::to_string(&response);
        self.socket.send_to(response_string.as_bytes(), &self.client_addr).await.unwrap();
    }
    async fn internal_service_6(&self, _req: &RPCRequest) -> Result<RPCResponse> {
        let db_service = self.db_service.lock().await;
        let Service6RequestBody { flight_id, amount_in_kg } = _req.decode_body()?;
        let client_addr = self.client_addr.clone();

        let buy_status = db_service.buy_luggage(flight_id, client_addr, amount_in_kg).await?;

        match buy_status {
            status @ BuyLuggageStatus::Success => {
                let message = status.to_string();
                let mut success_response = RPCResponse::finished(_req.id).await;
                success_response.encode_body(Service6ResponseBody { message });
                Ok(success_response)
            }
            BuyLuggageStatus::ReservationNotExisted => Err(APIError::RecordNotFound),
        }
    }
}
