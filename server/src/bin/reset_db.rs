use database::types::Result;
use database::{define_schemas, get_global_db_connection};

#[tokio::main]
async fn main() -> Result<()> {
    let connection = get_global_db_connection();
    let connection = connection.lock().await;
    define_schemas(&connection)?;
    Ok(())
}
