use anyhow::Error;
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

    pub async fn user_exists(&self, discord_id: String) -> Result<bool, Error> {

        let con = self.db.connect()?;

        let mut result = con.query("SELECT * FROM users WHERE discord_id = (:discord_id);",
                     libsql::named_params! { ":discord_id": discord_id }).await?;

        match result.next().await? {
            Some(_row) => {Ok(true)},
            None => {Ok(false)},
        }

    }

    pub async fn add_user(&self, faceit_id: String, discord_id: String) -> Result<bool, Error> {
        let con = self.db.connect()?;

        let results = con.execute("INSERT INTO users (discord_id, faceit_id) VALUES (:discord_id, :faceit_id)",
                    libsql::named_params! { ":discord_id": discord_id, ":faceit_id": faceit_id }).await?;

        Ok(results != 0)
    }

    pub async fn unlink_user(&self, discord_id: String) -> Result<bool, Error> {
        let con = self.db.connect()?;


        let results = con.execute("DELETE FROM users WHERE discord_id = :discord_id;",
                                  libsql::named_params! { ":discord_id": discord_id}).await?;

        Ok(results != 0)

    }

    pub async fn count_users(&self) -> Result<i64, Error> {
        let con = self.db.connect()?;

        let mut result = con.query("SELECT COUNT(*) FROM users;", ()).await?;

        while let Some(row) = result.next().await? {
            let count: i64 = row.get(0)?;
            return Ok(count);
        }

        Err(anyhow::anyhow!("Failed to count rows"))

    }

}
