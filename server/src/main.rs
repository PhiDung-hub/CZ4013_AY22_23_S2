use clap::Parser;
use rand::prelude::*;
use rpc_contracts::RPCRequest;
use serde::json;
use server::api::ServiceHandler;
use server::database::{get_connection_pool, DatabaseService};
use server::ReservationStatus;
use std::error::Error;
use std::str;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::sync::Notify;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1")]
    addr: String,

    #[arg(short, long, default_value = "1234")]
    port: u16,

    #[arg(short, long, default_value = "false")]
    loss: bool,

    #[arg(long, default_value = "0.25")]
    loss_prob: f64,
}

static mut SENDER: Option<mpsc::Sender<(u32, ReservationStatus)>> = None;
static mut RECEIVER: Option<mpsc::Receiver<(u32, ReservationStatus)>> = None;
static mut NOTIFY: Option<Notify> = None;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let options = Args::parse();
    let addr = options.addr;
    let port = options.port;
    let loss = options.loss;
    let loss_prob = options.loss_prob;
    let socket = Arc::new(UdpSocket::bind((addr, port)).await?);
    println!("{:?}", socket.local_addr());
    let mut buf = [0_u8; 2048]; // max 2048 bytes
    let pool = get_connection_pool();
    let db_service = DatabaseService::new(&pool).unwrap();
    let db_service_arc = Arc::new(Mutex::new(db_service));

    let (sender, receiver) = mpsc::channel::<(u32, ReservationStatus)>(32);
    let notify = Notify::new();

    unsafe {
        SENDER = Some(sender);
        RECEIVER = Some(receiver);
        NOTIFY = Some(notify);
    }

    let mut rng = rand::thread_rng();
    loop {
        let (byte_idx, client_addr) = socket.recv_from(&mut buf).await?;
        if loss {
            let is_response_lost = rng.gen_bool(loss_prob);
            if is_response_lost {
                continue;
            }
        }
        let data = &buf[..byte_idx];
        let data_str = str::from_utf8(data).unwrap();
        let request: RPCRequest = match json::from_str(data_str) {
            Ok(r) => r,
            Err(err) => {
                println!("Invalid request {:?}\nError {:?}", data_str, err);
                continue;
            }
        };
        println!("Received message from {}, service_type: {:?}", client_addr, request.service_type);
        let db_service_arc = db_service_arc.clone();
        let client_addr = client_addr.clone().to_string();
        let sender = unsafe { SENDER.as_ref().unwrap() };
        let receiver = unsafe { RECEIVER.as_mut().unwrap() };
        let notify = unsafe { NOTIFY.as_ref().unwrap() };
        let mut handler = ServiceHandler::new(socket.clone(), client_addr, db_service_arc, sender, receiver, notify).await;

        let service_type = request.service_type;
        tokio::spawn(async move {
            match service_type {
                1 => handler.service_1(request).await,
                2 => handler.service_2(request).await,
                3 => handler.service_3(request).await,
                4 => handler.service_4(request).await,
                5 => handler.service_5(request).await,
                6 => handler.service_6(request).await,
                _ => handler.invalid_service_type(request).await,
            };
        });
    }
}
