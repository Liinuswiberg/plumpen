use libsql::Builder;
use tracing::{info};

pub struct Database {
    db: libsql::Database,
}

impl Database {
    pub async fn new(url: String, token: String) -> Result<Self, libsql::Error> {
        let db_result = Builder::new_remote(url, token)
            .build()
            .await;

        match db_result {
            Ok(db) => {
                info!("Database connection established!");
                Ok(Database { db })
            },
            Err(err) => {
                eprintln!("Failed to connect to the database: {}", err);
                Err(err)
            }
        }
    }
}
