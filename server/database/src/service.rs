use deadpool_sqlite::Pool;
use rusqlite::{params, Result};

use crate::types::{BuyLuggageStatus, CancellationStatus, ReservationStatus};
use crate::types::{Flight, Reservation};

pub struct DatabaseService<'a> {
    pool: &'a Pool,
}

impl<'a> DatabaseService<'a> {
    pub fn new(pool: &'a Pool) -> Result<Self> {
        Ok(DatabaseService { pool })
    }

    /// internal helper function for other services
    async fn get_flight_by_id(&self, id: u32) -> Result<Option<Flight>> {
        let pool_conn = self.pool.get().await.unwrap();

        pool_conn
            .interact(move |connection| -> Result<Option<Flight>> {
                const GET_FLIGHT_DETAILS: &str = "SELECT * from flight_informations WHERE id = ?1";
                let mut stmt = connection.prepare(GET_FLIGHT_DETAILS)?;

                let flights = stmt.query_map(params![id], |row| {
                    Ok(Flight {
                        id: row.get(0)?,
                        source: row.get(1)?,
                        destination: row.get(2)?,
                        departure_time: row.get(3)?,
                        seat_available: row.get(4)?,
                        airfare: row.get(5)?,
                    })
                })?;

                let result = flights.collect::<Vec<Result<Flight>>>();
                let result = result.into_iter().next();
                result.transpose()
            })
            .await
            .expect("db interaction error")
    }

    /// helper function to check if flight with this id actually exists
    pub async fn is_flight_exists(&self, id: u32) -> Result<bool> {
        let flight = self.get_flight_by_id(id).await?;
        Ok(flight.is_some())
    }

    /// Service 1: <source, destination> -> Vec<id>
    pub async fn get_flight_ids(&self, source: String, destination: String) -> Result<Vec<u32>> {
        let pool_conn = self.pool.get().await.unwrap();
        pool_conn
            .interact(move |connection| {
                const GET_FLIGHT_IDS_QUERY: &str = "SELECT id from flight_informations WHERE source = ?1 AND destination = ?2";
                let mut stmt = connection.prepare(GET_FLIGHT_IDS_QUERY)?;

                let flight_ids = stmt.query_map(params![source, destination], |row| Ok(row.get::<_, u32>(0)?))?;
                let result = flight_ids.collect::<Result<Vec<u32>>>();
                result
            })
            .await
            .unwrap()
    }

    /// Service 2: <id> -> <departure_time, seat_available, airfare>
    pub async fn get_flight_info(&self, id: u32) -> Result<Option<(i32, f32, u32)>> {
        let flight = self.get_flight_by_id(id).await?;
        let flight_info = flight.map(|f| (f.departure_time, f.airfare, f.seat_available));
        Ok(flight_info)
    }

    /// Service 3: <id, num_seat> -> ReservationStatus
    pub async fn make_reservation(&self, flight_id: u32, client_ip: String, num_seat: u32) -> Result<ReservationStatus> {
        if num_seat == 0 {
            return Ok(ReservationStatus::ZeroSeatReserved);
        }

        let target_flight = self.get_flight_by_id(flight_id).await?;
        if target_flight.is_none() {
            return Ok(ReservationStatus::InvalidFlightID);
        }

        let target_flight = target_flight.unwrap();
        if target_flight.seat_available < num_seat {
            return Ok(ReservationStatus::InsufficientCapacity);
        }

        let is_reservation_existed = self.get_reservation_by_client(flight_id, client_ip.clone()).await?.is_some();

        let pool_conn = self.pool.get().await.unwrap();
        let result = pool_conn
            .interact(move |connection| {
                let transaction = connection.transaction()?;
                const UPDATE_FLIGHT_QUERY: &str = "UPDATE flight_informations SET seat_available = seat_available - ?2 WHERE id = ?1";
                transaction.execute(UPDATE_FLIGHT_QUERY, params![flight_id, num_seat])?;
                match is_reservation_existed {
                    true => {
                        const UPDATE_RESERVATION_QUERY: &str =
                            "UPDATE reservations SET seat_reserved = seat_reserved + ?3 WHERE flight_id = ?1 AND client_ip = ?2";
                        transaction.execute(UPDATE_RESERVATION_QUERY, params![flight_id, client_ip, num_seat])?;
                        transaction.commit().map(|_| ReservationStatus::Updated)
                    }
                    false => {
                        const MAKE_RESERVATION_QUERY: &str = "INSERT INTO reservations (flight_id, client_ip, seat_reserved) VALUES (?1, ?2, ?3)";
                        transaction.execute(MAKE_RESERVATION_QUERY, params![flight_id, client_ip, num_seat])?;
                        transaction.commit().map(|_| ReservationStatus::Created)
                    }
                }
            })
            .await
            .unwrap();
        result
    }

    pub async fn get_reservation_by_client(&self, flight_id: u32, client_ip: String) -> Result<Option<Reservation>> {
        let pool_conn = self.pool.get().await.unwrap();
        pool_conn
            .interact(move |connection| {
                const GET_RESERVATION_DETAILS: &str = "SELECT * from reservations WHERE flight_id = ?1 AND client_ip = ?2";
                let mut stmt = connection.prepare(GET_RESERVATION_DETAILS)?;

                let reservations = stmt.query_map(params![flight_id, client_ip], |row| {
                    Ok(Reservation {
                        id: row.get(0)?,
                        flight_id: row.get(1)?,
                        client_ip: row.get(2)?,
                        seat_reserved: row.get(3)?,
                        luggage_amount: row.get(4)?,
                    })
                })?;

                let result = reservations.collect::<Vec<Result<Reservation>>>();
                let result = result.into_iter().next();

                result.transpose()
            })
            .await
            .unwrap()
    }

    /// Service 5: <flight_id, client_ip> -> ReservationStatus
    /// NOTE: This function is idempotent
    pub async fn cancel_reservation(&self, flight_id: u32, client_ip: String) -> Result<CancellationStatus> {
        let reservation_detail = self.get_reservation_by_client(flight_id, client_ip.clone()).await?;
        if reservation_detail.is_none() {
            return Ok(CancellationStatus::ReservationNotExisted);
        }

        let num_reserved = reservation_detail.unwrap().seat_reserved;

        let pool_conn = self.pool.get().await.unwrap();

        pool_conn
            .interact(move |connection| {
                let transaction = connection.transaction()?;

                const UPDATE_RESERVATION_QUERY: &str = "UPDATE flight_informations SET seat_available = seat_available + ?2 WHERE id = ?1";
                transaction.execute(UPDATE_RESERVATION_QUERY, params![flight_id, num_reserved])?;

                const CANCEL_RESERVATION_QUERY: &str = "DELETE FROM reservations WHERE flight_id = ?1 AND client_ip = ?2";
                transaction.execute(CANCEL_RESERVATION_QUERY, params![flight_id, client_ip])?;

                match transaction.commit() {
                    Ok(_) => Ok(CancellationStatus::Success),
                    Err(_e) => panic!("{}", _e.to_string()),
                }
            })
            .await
            .unwrap()
    }

    /// Service 6: <flight_id, client_ip, amount_in_kg> -> BuyLuggageStatus
    /// NOTE: This function is non-idempotent
    pub async fn buy_luggage(&self, flight_id: u32, client_ip: String, amount_in_kg: u32) -> Result<BuyLuggageStatus> {
        let is_reservation_existed = self.get_reservation_by_client(flight_id, client_ip.clone()).await?.is_some();
        if !is_reservation_existed {
            return Ok(BuyLuggageStatus::ReservationNotExisted);
        }

        let pool_conn = self.pool.get().await.unwrap();

        pool_conn
            .interact(move |connection| {
                let transaction = connection.transaction()?;

                const UPDATE_FLIGHT_QUERY: &str =
                    "UPDATE reservations SET luggage_amount = luggage_amount + ?3 WHERE flight_id = ?1 AND client_ip = ?2";
                transaction.execute(UPDATE_FLIGHT_QUERY, params![flight_id, client_ip, amount_in_kg])?;

                match transaction.commit() {
                    Ok(_) => Ok(BuyLuggageStatus::Success),
                    Err(_e) => panic!("{}", _e.to_string()),
                }
            })
            .await
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{define_schemas, seed_test_db_with_hardcode_data, BuyLuggageStatus, ReservationStatus};
    use deadpool_sqlite::{Config, Manager, Runtime};
    use rusqlite::Result;
    use std::{fs, path::PathBuf};

    async fn seed_db_and_prepare_pool(test_db_name: &str) -> Result<Pool> {
        let pool = {
            let config = Config::new(test_db_name);
            let manager = Manager::from_config(&config, Runtime::Tokio1);
            Pool::builder(manager).build().unwrap()
        };
        let pool_conn = pool.get().await.unwrap();
        let seed_db = pool_conn
            .interact(|connection| -> Result<()> {
                define_schemas(connection)?;
                seed_test_db_with_hardcode_data(connection)?;
                Ok(())
            })
            .await
            .expect("tokio error");
        assert!(seed_db.is_ok());
        Ok(pool)
    }

    #[tokio::test]
    async fn get_flight_by_id_should_return_flight_id_1() -> Result<()> {
        let file_path = "test_get_flight_by_id_should_return_flight_id_1.db";
        if PathBuf::from(file_path).exists() {
            fs::remove_file(file_path).unwrap();
        }
        let pool = seed_db_and_prepare_pool(file_path).await?;
        let service = DatabaseService::new(&pool)?;
        let flight_with_id_1 = service.get_flight_by_id(1).await?.unwrap();
        assert_eq!(
            flight_with_id_1,
            Flight {
                id: 1,
                source: "LAS".to_string(),
                destination: "HAN".to_string(),
                departure_time: 1680105600,
                seat_available: 500,
                airfare: 150.99
            }
        );
        Ok(())
    }

    #[tokio::test]
    async fn get_flight_by_id_expect_error_id_not_exists() -> Result<()> {
        let file_path = "get_flight_by_id_expect_error_id_not_exists.db";
        if PathBuf::from(file_path).exists() {
            fs::remove_file(file_path).unwrap();
        }
        let pool = seed_db_and_prepare_pool(file_path).await?;
        let service = DatabaseService::new(&pool)?;
        let flight = service.get_flight_by_id(11).await?;
        assert!(flight.is_none(), "Flight should not be found");
        Ok(())
    }

    #[tokio::test]
    async fn get_flight_by_source_and_destination() -> Result<()> {
        let file_path = "get_flight_by_source_and_destination.db";
        if PathBuf::from(file_path).exists() {
            fs::remove_file(file_path).unwrap();
        }
        let pool = seed_db_and_prepare_pool(file_path).await?;
        let service = DatabaseService::new(&pool)?;
        let flight_ids = service.get_flight_ids("LAS".to_string(), "HAN".to_string()).await?;
        assert_eq!(flight_ids, [1, 8]);
        Ok(())
    }

    #[tokio::test]
    async fn make_reservation_should_success_twice() -> Result<()> {
        let file_path = "make_reservation_should_success_twice.db";
        if PathBuf::from(file_path).exists() {
            fs::remove_file(file_path).unwrap();
        }
        let pool = seed_db_and_prepare_pool(file_path).await?;

        let service = DatabaseService::new(&pool)?;

        const CLIENT_IP: &str = "192.168.0.1";
        const FLIGHT_ID: u32 = 1;
        const NUM_RESERVED: u32 = 5;

        let capacity_prior_to_reservation = service.get_flight_by_id(FLIGHT_ID).await?.unwrap().seat_available;

        // First time make reservation
        let _result = service.make_reservation(FLIGHT_ID, CLIENT_IP.to_string(), NUM_RESERVED).await?;
        assert_eq!(_result, ReservationStatus::Created);
        assert_eq!(service.get_flight_by_id(FLIGHT_ID).await?.unwrap().seat_available, capacity_prior_to_reservation - NUM_RESERVED);
        assert_eq!(
            service
                .get_reservation_by_client(FLIGHT_ID, CLIENT_IP.to_string())
                .await?
                .unwrap()
                .seat_reserved,
            NUM_RESERVED
        );

        // Second time make reservation
        let _result = service.make_reservation(FLIGHT_ID, CLIENT_IP.to_string(), NUM_RESERVED).await?;
        assert_eq!(_result, ReservationStatus::Updated);
        assert_eq!(service.get_flight_by_id(FLIGHT_ID).await?.unwrap().seat_available, capacity_prior_to_reservation - 2 * NUM_RESERVED);
        assert_eq!(
            service
                .get_reservation_by_client(FLIGHT_ID, CLIENT_IP.to_string())
                .await?
                .unwrap()
                .seat_reserved,
            2 * NUM_RESERVED
        );

        Ok(())
    }

    #[tokio::test]
    async fn make_reservation_should_be_invalid() -> Result<()> {
        let file_path = "make_reservation_should_be_invalid.db";
        if PathBuf::from(file_path).exists() {
            fs::remove_file(file_path).unwrap();
        }
        let pool = seed_db_and_prepare_pool(file_path).await?;
        let service = DatabaseService::new(&pool)?;

        const CLIENT_IP: &str = "192.168.0.1";
        const FLIGHT_ID: u32 = 1;

        let capacity_prior_to_reservation = service.get_flight_by_id(FLIGHT_ID).await?.unwrap().seat_available;

        let _result = service.make_reservation(FLIGHT_ID, CLIENT_IP.to_string(), 0).await?;
        assert_eq!(_result, ReservationStatus::ZeroSeatReserved);

        let _result = service.make_reservation(11, CLIENT_IP.to_string(), 5).await?;
        assert_eq!(_result, ReservationStatus::InvalidFlightID);

        let _result = service
            .make_reservation(FLIGHT_ID, CLIENT_IP.to_string(), capacity_prior_to_reservation + 100)
            .await?;
        assert_eq!(_result, ReservationStatus::InsufficientCapacity);

        assert_eq!(
            service.get_flight_by_id(FLIGHT_ID).await?.unwrap().seat_available,
            capacity_prior_to_reservation,
            "Flight state should not change"
        );
        Ok(())
    }

    #[tokio::test]
    async fn cancel_reservation_should_success_then_return_record_not_existed() -> Result<()> {
        let file_path = "cancel_reservation_should_success_then_return_record_not_existed.db";
        if PathBuf::from(file_path).exists() {
            fs::remove_file(file_path).unwrap();
        }
        let pool = seed_db_and_prepare_pool(file_path).await?;
        let service = DatabaseService::new(&pool)?;

        const CLIENT_IP: &str = "192.168.0.1";
        const FLIGHT_ID: u32 = 1;
        const NUM_RESERVED: u32 = 5;

        let capacity_prior_to_reservation = service.get_flight_by_id(FLIGHT_ID).await?.unwrap().seat_available;

        service.make_reservation(FLIGHT_ID, CLIENT_IP.to_string(), NUM_RESERVED).await?;
        service.make_reservation(FLIGHT_ID, CLIENT_IP.to_string(), NUM_RESERVED).await?;
        assert_eq!(service.get_flight_by_id(FLIGHT_ID).await?.unwrap().seat_available, capacity_prior_to_reservation - 2 * NUM_RESERVED);

        // First time cancel
        let cancel_reservation_result = service.cancel_reservation(FLIGHT_ID, CLIENT_IP.to_string()).await?;
        assert_eq!(cancel_reservation_result, CancellationStatus::Success);
        assert_eq!(service.get_flight_by_id(FLIGHT_ID).await?.unwrap().seat_available, capacity_prior_to_reservation);

        // Second time cancel
        assert_eq!(service.cancel_reservation(FLIGHT_ID, CLIENT_IP.to_string()).await?, CancellationStatus::ReservationNotExisted);
        Ok(())
    }

    #[tokio::test]
    async fn buy_luggage() -> Result<()> {
        let file_path = "buy_luggage.db";
        if PathBuf::from(file_path).exists() {
            fs::remove_file(file_path).unwrap();
        }
        let pool = seed_db_and_prepare_pool(file_path).await?;
        let service = DatabaseService::new(&pool)?;

        const CLIENT_IP: &str = "192.168.0.1";
        const FLIGHT_ID: u32 = 1;
        const AMOUNT_LUGGAGE_IN_KG: u32 = 5;

        assert_eq!(service.buy_luggage(FLIGHT_ID, CLIENT_IP.to_string(), 10).await?, BuyLuggageStatus::ReservationNotExisted);

        service.make_reservation(FLIGHT_ID, CLIENT_IP.to_string(), 1).await?;

        let status = service.buy_luggage(FLIGHT_ID, CLIENT_IP.to_string(), AMOUNT_LUGGAGE_IN_KG).await?;
        assert_eq!(status, BuyLuggageStatus::Success);
        assert_eq!(
            service
                .get_reservation_by_client(FLIGHT_ID, CLIENT_IP.to_string())
                .await?
                .unwrap()
                .luggage_amount,
            AMOUNT_LUGGAGE_IN_KG
        );

        let status = service.buy_luggage(FLIGHT_ID, CLIENT_IP.to_string(), AMOUNT_LUGGAGE_IN_KG).await?;
        assert_eq!(status, BuyLuggageStatus::Success);
        assert_eq!(
            service
                .get_reservation_by_client(FLIGHT_ID, CLIENT_IP.to_string())
                .await?
                .unwrap()
                .luggage_amount,
            2 * AMOUNT_LUGGAGE_IN_KG
        );
        Ok(())
    }
}
