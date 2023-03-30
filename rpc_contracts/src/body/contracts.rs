use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ServiceFailedResponse {
    pub error: String,
}

#[derive(Serialize, Deserialize)]
pub struct Service1RequestBody {
    pub source: String,
    pub destination: String,
}

#[derive(Serialize, Deserialize)]
pub struct Service1ResponseBody {
    pub flight_ids: Vec<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct Service2RequestBody {
    pub flight_id: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Service2ResponseBody {
    pub departure_time: i32,
    pub airfare: f32,
    pub seat_avail: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Service3RequestBody {
    pub flight_id: u32,
    pub num_seat: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Service3ResponseBody {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct Service4RequestBody {
    pub flight_id: u32,
    pub monitor_interval: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Service4ResponseBody {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct Service4MonitorResponseBody {
    pub seat_avail: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Service5RequestBody {
    pub flight_id: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Service5ResponseBody {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct Service6RequestBody {
    pub flight_id: u32,
    pub amount_in_kg: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Service6ResponseBody {
    pub message: String,
}
