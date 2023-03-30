use rand::seq::SliceRandom;
use rand::SeedableRng;
use rusqlite::{Connection, Result};

pub fn define_schemas(connection: &Connection) -> Result<()> {
    connection.execute("DROP TABLE IF EXISTS flight_informations", ())?;

    let _define_flight_info_schema = "\
            CREATE TABLE IF NOT EXISTS flight_informations (\
                id INTEGER PRIMARY KEY, \
                source TEXT NOT NULL, \
                destination TEXT NOT NULL, \
                departure_time INTEGER NOT NULL, \
                seat_available INTEGER NOT NULL, \
                airfare REAL NOT NULL
            )
        ";
    connection.execute(_define_flight_info_schema, ())?;

    let _define_reservation_schema = "\
            CREATE TABLE IF NOT EXISTS reservations (\
                id INTEGER PRIMARY KEY, \
                flight_id INTEGER NOT NULL, \
                client_ip TEXT NOT NULL, \
                seat_reserved INTEGER NOT NULL, \
                luggage_amount INTEGER DEFAULT 0
            )
        ";
    connection.execute(_define_reservation_schema, ())?;

    Ok(())
}

pub fn seed_db(connection: &Connection) -> Result<()> {
    let insert = move |source: &str, destination: &str, departure_time: i32, seat_available: u32, airfare: f32| -> Result<()> {
        let _insert_query = "\
                INSERT INTO flight_informations \
                (source, destination, departure_time, seat_available, airfare) \
                VALUES (?1, ?2, ?3, ?4, ?5)
            ";
        connection.execute(_insert_query, (source, destination, departure_time, seat_available, airfare))?;
        Ok(())
    };

    const DEADLINE: i32 = 1680105600; // 31 March 2023, 0:00:00

    let departure_times: Vec<i32> = vec![DEADLINE, DEADLINE + 3600, DEADLINE + 86400];
    let locations = ["LAX", "LAS", "SIN", "HAN", "SYD", "PAR"];
    let capacities: Vec<u32> = vec![200, 300, 500];
    let airfares = vec![120.99, 150.99, 590.99];

    const RNG_SEED: u64 = 42;
    const NUM_FLIGHTS: u32 = 10;

    let mut rng = rand::rngs::StdRng::seed_from_u64(RNG_SEED);

    for _ in 0..NUM_FLIGHTS {
        let time_idx = departure_times.choose(&mut rng).unwrap();
        let loc_idx = locations.choose_multiple(&mut rng, 2).collect::<Vec<_>>();
        let cap_idx = capacities.choose(&mut rng).unwrap();
        let fee = airfares.choose(&mut rng).unwrap();

        insert(*loc_idx[0], *loc_idx[1], *time_idx, *cap_idx, *fee)?;
    }

    Ok(())
}

pub fn seed_test_db_with_hardcode_data(connection: &Connection) -> Result<()> {
    let insert = move |source: &str, destination: &str, departure_time: i32, seat_available: u32, airfare: f32| -> Result<()> {
        let _insert_query = "\
                INSERT INTO flight_informations \
                (source, destination, departure_time, seat_available, airfare) \
                VALUES (?1, ?2, ?3, ?4, ?5)
            ";
        connection.execute(_insert_query, (source, destination, departure_time, seat_available, airfare))?;
        Ok(())
    };

    const DEADLINE: i32 = 1680105600; // 31 March 2023, 0:00:00

    const HOUR: i32 = 3600;
    insert("LAS", "HAN", DEADLINE, 500, 150.99)?; // 1
    insert("HAN", "SIN", DEADLINE + HOUR, 300, 590.99)?; // 2
    insert("SYD", "LAX", DEADLINE + 2 * HOUR, 500, 150.99)?; // 3
    insert("PAR", "HAN", DEADLINE + 3 * HOUR, 500, 120.99)?; // 4
    insert("SIN", "LAX", DEADLINE + 4 * HOUR, 500, 150.99)?; // 5
    insert("PAR", "SIN", DEADLINE + 5 * HOUR, 200, 120.99)?; // 6
    insert("LAS", "sIN", DEADLINE + 6 * HOUR, 200, 590.99)?; // 7
    insert("LAS", "HAN", DEADLINE + 7 * HOUR, 300, 120.99)?; // 8
    insert("SIN", "LAS", DEADLINE + 8 * HOUR, 500, 150.99)?; // 9
    insert("HAN", "LAS", DEADLINE + 9 * HOUR, 300, 150.99)?; // 10

    Ok(())
}
