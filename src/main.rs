mod database;
mod discord;
mod faceit;

use anyhow::Context as _;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use discord::DiscordBot;
use faceit::Faceit;
use crate::database::Database;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;
use serenity::all::Http;
use tokio::time::sleep;
use tracing::{error, info};

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {

    std::env::set_var("TURSO_TOKEN", secrets.get("TURSO_TOKEN").context("'TURSO_TOKEN' was not found")?);
    std::env::set_var("TURSO_DATABASE", secrets.get("TURSO_DATABASE").context("'TURSO_DATABASE' was not found")?);
    std::env::set_var("FACEIT_TOKEN", secrets.get("FACEIT_TOKEN").context("'FACEIT_TOKEN' was not found")?);

    let intents = GatewayIntents::GUILD_MEMBERS |
        GatewayIntents::GUILD_MESSAGES |
        GatewayIntents::DIRECT_MESSAGES |
        GatewayIntents::MESSAGE_CONTENT |
        GatewayIntents::GUILDS;

    let client = Client::builder(secrets.get("DISCORD_TOKEN").context("'DISCORD_TOKEN' was not found")?, intents)
        .event_handler(DiscordBot)
        .await
        .expect("Err creating client");

    tokio::spawn(name_syncer(client.http.clone()));

    Ok(client.into())

}

async fn name_syncer(http: Arc<Http>) {

    info!("Starting name sync task");

    loop {

        let Ok(users) = Database.fetch_users().await else {
            error!("Could not get users from database");
            sleep(Duration::from_secs(2)).await;
            continue;
        };

        for user in users.iter() {
            println!("{}", user.discord_id);
        }
        info!("Name sync resting for 2 seconds.");
        sleep(Duration::from_secs(2)).await;
    }
}




