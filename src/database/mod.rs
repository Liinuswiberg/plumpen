use std::env;
use anyhow::Error;
use libsql::{Builder, Row};
use serenity::all::User;
use tracing::{error, info};
use serde::{Deserialize, Serialize};

pub struct Database;

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkedUser {
    pub faceit_id: String,
    pub discord_id: String,
}

impl LinkedUser {
    fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
        let faceit_id: String = row.get(0)?;
        let discord_id: String = row.get(1)?;
        Ok(LinkedUser { faceit_id, discord_id })
    }
}

impl Database {

    async fn connect() -> libsql::Database {

        // Just panic if these aren't set.
        let url = env::var("TURSO_DATABASE").expect("Failed to get TURSO_DATABASE!");
        let token = env::var("TURSO_TOKEN").expect("Failed to get TURSO_TOKEN!");

        // @TODO Add more error handling later when the rewrite is done
        Builder::new_remote(url, token)
            .build()
            .await.expect("Could not connect to database")
    }

    pub async fn user_exists(&self, discord_id: String) -> Result<bool, Error> {

        let db: libsql::Database = Self::connect().await;

        let con = db.connect()?;

        let mut result = con.query("SELECT * FROM users WHERE discord_id = (:discord_id);",
                     libsql::named_params! { ":discord_id": discord_id }).await?;

        match result.next().await? {
            Some(_row) => {Ok(true)},
            None => {Ok(false)},
        }

    }

    pub async fn add_user(&self, faceit_id: String, discord_id: String) -> Result<bool, Error> {

        let db: libsql::Database = Self::connect().await;

        let con = db.connect()?;

        let results = con.execute("INSERT INTO users (discord_id, faceit_id) VALUES (:discord_id, :faceit_id)",
                    libsql::named_params! { ":discord_id": discord_id, ":faceit_id": faceit_id }).await?;

        Ok(results != 0)
    }

    pub async fn unlink_user(&self, discord_id: String) -> Result<bool, Error> {

        let db: libsql::Database = Self::connect().await;

        let con = db.connect()?;

        let results = con.execute("DELETE FROM users WHERE discord_id = :discord_id;",
                                  libsql::named_params! { ":discord_id": discord_id}).await?;

        Ok(results != 0)

    }

    pub async fn count_users(&self) -> Result<i64, Error> {

        let db: libsql::Database = Self::connect().await;

        let con = db.connect()?;

        let mut result = con.query("SELECT COUNT(*) FROM users;", ()).await?;

        while let Some(row) = result.next().await? {
            let count: i64 = row.get(0)?;
            return Ok(count);
        }

        Err(anyhow::anyhow!("Failed to count rows"))

    }

    pub async fn fetch_users(&self) -> Result<Vec<LinkedUser>, Error> {

        let db: libsql::Database = Self::connect().await;

        let con = db.connect()?;

        let mut rows = con.query("SELECT faceit_id, discord_id FROM users", ()).await?;

        let mut users = Vec::new();

        while let Some(row) = rows.next().await? {
            let parsed_user = LinkedUser::from_row(&row);
            match parsed_user {
                Ok(parsed_user) => {
                    users.push(parsed_user);
                },
                Err(err) => error!("Failed to parse row to LinkedUser: {}", err)
            }
        }

        Ok(users)
    }

}
