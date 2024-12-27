mod database;
mod discord;
mod faceit;

use anyhow::Context as _;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use crate::discord::DiscordBot;

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MEMBERS |
        GatewayIntents::GUILD_MESSAGES |
        GatewayIntents::DIRECT_MESSAGES |
        GatewayIntents::MESSAGE_CONTENT |
        GatewayIntents::GUILDS;

    let client = Client::builder(&token, intents)
        .event_handler(DiscordBot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}




