mod database;
mod discord;
mod faceit;

use anyhow::Context as _;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use discord::DiscordBot;

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {

    let token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let intents = GatewayIntents::GUILD_MEMBERS |
        GatewayIntents::GUILD_MESSAGES |
        GatewayIntents::DIRECT_MESSAGES |
        GatewayIntents::MESSAGE_CONTENT |
        GatewayIntents::GUILDS;

    let discord_bot = DiscordBot::new();

    let client = Client::builder(&token, intents)
        .event_handler(discord_bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}




