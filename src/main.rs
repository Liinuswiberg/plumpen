mod database;
mod discord;
mod faceit;

use anyhow::Context as _;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use discord::DiscordBot;
use crate::database::Database;
use tokio::sync::Mutex;
use std::sync::Arc;

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {

    let database = Database::new(secrets.get("TURSO_DATABASE").context("'TURSO_DATABASE' was not found")?, secrets.get("TURSO_TOKEN").context("'TURSO_TOKEN' was not found")?)
        .await
        .expect("Error establishing database connection");

    let database = Arc::new(Mutex::new(database));

    let intents = GatewayIntents::GUILD_MEMBERS |
        GatewayIntents::GUILD_MESSAGES |
        GatewayIntents::DIRECT_MESSAGES |
        GatewayIntents::MESSAGE_CONTENT |
        GatewayIntents::GUILDS;

    let discord = DiscordBot::new(database);

    let client = Client::builder(secrets.get("DISCORD_TOKEN").context("'DISCORD_TOKEN' was not found")?, intents)
        .event_handler(discord)
        .await
        .expect("Err creating client");

    Ok(client.into())

}




