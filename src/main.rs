mod database;
mod discord;
mod faceit;

use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use discord::DiscordBot;
use faceit::Faceit;
use crate::database::Database;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;
use serenity::all::{GuildId, Http, UserId};
use tokio::time::sleep;
use tracing::{error, info};
use crate::faceit::Player;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type PoiseContext<'a> = poise::Context<'a, Data, Error>;
struct Data {}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {

    std::env::set_var("TURSO_TOKEN", secrets.get("TURSO_TOKEN").expect("'TURSO_TOKEN' was not found"));
    std::env::set_var("TURSO_DATABASE", secrets.get("TURSO_DATABASE").expect("'TURSO_DATABASE' was not found"));
    std::env::set_var("FACEIT_TOKEN", secrets.get("FACEIT_TOKEN").expect("'FACEIT_TOKEN' was not found"));
    std::env::set_var("BOT_OWNER", secrets.get("BOT_OWNER").expect("'BOT_OWNER' was not found"));

    let intents = GatewayIntents::GUILD_MEMBERS |
        GatewayIntents::GUILD_MESSAGES |
        GatewayIntents::DIRECT_MESSAGES |
        GatewayIntents::MESSAGE_CONTENT |
        GatewayIntents::GUILDS;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![discord::commands::help(), discord::commands::link(), discord::commands::unlink()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(ctx, &framework.options().commands, GuildId::new(1325921171417596006)).await?;
                Ok(Data {})
            })
        })
        .build();

//poise::builtins::register_in_guild()
    //poise::builtins::register_globally(ctx, &framework.options().commands).await?;
    let client = Client::builder(secrets.get("DISCORD_TOKEN").expect("'DISCORD_TOKEN' was not found"), intents)
        .framework(framework)
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

        info!("Got {} users from database, starting name sync.", users.len());

        for user in users.iter() {
            let Ok(player) = Faceit::get_faceit_user_by_id(&user.faceit_id).await else { continue };


            match player {
                None => {
                    info!("No player data for user '{}'", user.faceit_id);
                }
                Some(p) => {

                    info!("Syncing user '{}'.", p.nickname);

                    let Ok(u64_id) = user.discord_id.parse::<u64>() else {
                        continue;
                    };

                    DiscordBot::parse_user(&http, UserId::new(u64_id), p).await;
                }
            }

            sleep(Duration::from_millis(70)).await;

        }

        info!("Name sync resting for 10 seconds.");
        sleep(Duration::from_secs(10)).await;
    }
}




