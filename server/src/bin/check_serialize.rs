use database::{get_global_db_connection, Flight};
use database::types::Result;
use serde::json;

#[tokio::main]
async fn main() -> Result<()> {
    let connection = get_global_db_connection();
    let connection = connection.lock().await;

    let mut stmt = connection.prepare("SELECT * from flight_informations;")?;

    let flights = stmt.query_map([], |row| {
        Ok(Flight {
            id: row.get(0)?,
            source: row.get(1)?,
            destination: row.get(2)?,
            departure_time: row.get(3)?,
            seat_available: row.get(4)?,
            airfare: row.get(5)?,
        })
    }).expect("Remember to seed the database");

    for flight in flights {
        let flight = flight?;
        let ser_f = json::to_string(&flight);
        println!("Serialized flight {:?}", ser_f);
        let de_f: Flight = json::from_str(&ser_f).unwrap();
        println!("Deserialized flight {:?}", de_f);
    }

    Ok(())
}
