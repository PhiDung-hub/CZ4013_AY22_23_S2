use serde::{Deserialize, Serialize};
use std::{error, fmt};

pub type Error = rusqlite::Error;
pub type Result<T> = rusqlite::Result<T>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Flight {
    pub id: u32,
    pub source: String,
    pub destination: String,
    pub departure_time: i32,
    pub seat_available: u32,
    pub airfare: f32,
}

impl Flight {
    pub fn new(id: u32, source: &str, destination: &str, departure_time: i32, seat_available: u32, airfare: f32) -> Self {
        Flight {
            id,
            source: source.to_string(),
            destination: destination.to_string(),
            departure_time,
            seat_available,
            airfare,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reservation {
    pub id: u32,
    pub flight_id: u32,
    pub client_ip: String,
    pub seat_reserved: u32,
    pub luggage_amount: u32,
}

impl Reservation {
    pub fn new(id: u32, flight_id: u32, client_ip: &str, seat_reserved: u32) -> Self {
        Reservation {
            id,
            flight_id,
            client_ip: client_ip.to_string(),
            seat_reserved,
            luggage_amount: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ReservationStatus {
    InvalidFlightID,
    InsufficientCapacity,
    ZeroSeatReserved,
    Created,
    Updated,
}
impl error::Error for ReservationStatus {}
impl fmt::Display for ReservationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReservationStatus::InvalidFlightID => write!(f, "Flight not found"),
            ReservationStatus::InsufficientCapacity => write!(f, "Not enough seat"),
            ReservationStatus::ZeroSeatReserved => write!(f, "Seat reserved should be greater"),
            ReservationStatus::Created => write!(f, "Reservation created"),
            ReservationStatus::Updated => write!(f, "Reservation updated"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CancellationStatus {
    ReservationNotExisted,
    Success,
}
impl error::Error for CancellationStatus {}
impl fmt::Display for CancellationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CancellationStatus::ReservationNotExisted => write!(f, "Reservation not found"),
            CancellationStatus::Success => write!(f, "Reservation successfully cancelled"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BuyLuggageStatus {
    ReservationNotExisted,
    Success,
}
impl error::Error for BuyLuggageStatus {}
impl fmt::Display for BuyLuggageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuyLuggageStatus::ReservationNotExisted => write!(f, "Reservation not found"),
            BuyLuggageStatus::Success => write!(f, "Luggage successfully bought"),
        }
    }
}
