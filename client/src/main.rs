use clap::Parser;
use client::api::ServiceConsumer;
use rand::prelude::*;
use std::error::Error;
use std::io::{stdin, stdout, Write};
use tokio::net::UdpSocket;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1")]
    addr: String,

    #[arg(short, long, default_value = "3000")]
    port: u16,

    #[arg(long, default_value = "127.0.0.1")]
    server_addr: String,

    #[arg(long, default_value = "1234")]
    server_port: u16,

    #[arg(short, long, default_value = "false")]
    loss: bool,

    #[arg(long, default_value = "0.25")]
    loss_prob: f64,

    #[arg(short, long, default_value = "false")]
    retry: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let options = Args::parse();
    let addr = options.addr;
    let port = options.port;
    let server_addr = options.server_addr;
    let server_port = options.server_port;
    let loss = options.loss;
    let loss_prob = options.loss_prob;
    let retry = options.retry;

    let socket = UdpSocket::bind((addr, port)).await?;
    let consumer = ServiceConsumer::new(&socket, server_addr, server_port, retry);

    let mut rng = rand::thread_rng();
    loop {
        if loss {
            let is_request_lost = rng.gen_bool(loss_prob);
            if is_request_lost {
                continue;
            }
        }
        println!("\n\nEnter your choice:");
        println!("1. Service 1");
        println!("2. Service 2");
        println!("3. Service 3");
        println!("4. Service 4");
        println!("5. Service 5");
        println!("6. Service 6");
        println!("7. Exit");

        print!("Choice: ");
        let _ = stdout().flush();

        let mut choice = String::new();
        stdin()
            .read_line(&mut choice)
            .expect("Failed to read input.");
        let choice = choice.trim().parse::<u32>().unwrap_or(0);

        match choice {
            1 => {
                print!("Enter flight source: ");
                let _ = stdout().flush();
                let mut source = String::new();
                stdin().read_line(&mut source)?;
                let source = source.trim().to_string();

                print!("Enter flight destination: ");
                let _ = stdout().flush();
                let mut destination = String::new();
                stdin().read_line(&mut destination)?;
                let destination = destination.trim().to_string();

                consumer.invoke_request_service_1(source, destination).await;
            }
            2 => {
                print!("Enter flight id: ");
                let _ = stdout().flush();
                let mut flight_id = String::new();
                stdin().read_line(&mut flight_id)?;
                let flight_id = flight_id.trim().parse::<u32>().unwrap_or(0);

                consumer.invoke_request_service_2(flight_id).await;
            }
            3 => {
                print!("Enter flight id: ");
                let _ = stdout().flush();
                let mut flight_id = String::new();
                stdin().read_line(&mut flight_id)?;
                let flight_id = flight_id.trim().parse::<u32>().unwrap_or(0);

                print!("Enter number of seat to reserved: ");
                let _ = stdout().flush();
                let mut num_seat = String::new();
                stdin().read_line(&mut num_seat)?;
                let num_seat = num_seat.trim().parse::<u32>().unwrap_or(0);

                consumer.invoke_request_service_3(flight_id, num_seat).await;
            }
            4 => {
                print!("Enter flight id: ");
                let _ = stdout().flush();
                let mut flight_id = String::new();
                stdin().read_line(&mut flight_id)?;
                let flight_id = flight_id.trim().parse::<u32>().unwrap_or(0);

                print!("Enter length of monitoring period: ");
                let _ = stdout().flush();
                let mut monitor_interval = String::new();
                stdin().read_line(&mut monitor_interval)?;
                let monitor_interval = monitor_interval.trim().parse::<u32>().unwrap_or(0);

                consumer
                    .request_service_4(flight_id, monitor_interval)
                    .await;
            }
            5 => {
                print!("Enter flight id: ");
                let _ = stdout().flush();
                let mut flight_id = String::new();
                stdin().read_line(&mut flight_id)?;
                let flight_id = flight_id.trim().parse::<u32>().unwrap_or(0);

                let _ = consumer.invoke_request_service_5(flight_id).await;
            }
            6 => {
                print!("Enter flight id: ");
                let _ = stdout().flush();
                let mut flight_id = String::new();
                stdin().read_line(&mut flight_id)?;
                let flight_id = flight_id.trim().parse::<u32>().unwrap_or(0);

                print!("Enter amount of luggague to buy in kg: ");
                let _ = stdout().flush();
                let mut amount = String::new();
                stdin().read_line(&mut amount)?;
                let amount = amount.trim().parse::<u32>().unwrap_or(0);

                consumer.invoke_request_service_3(flight_id, amount).await;
            }
            7 => break,
            _ => println!("Invalid choice. Please try again"),
        }
    }
    Ok(())
}
