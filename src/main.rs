mod database;
mod discord;
mod faceit;

use serenity::prelude::*;
use discord::DiscordBot;
use faceit::Faceit;
use crate::database::Database;
use std::sync::Arc;
use std::time::Duration;
use serenity::all::{Http, UserId};
use tokio::time::sleep;
use tracing::{error, info};
use crate::faceit::Player;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type PoiseContext<'a> = poise::Context<'a, Data, Error>;
struct Data {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    let required_vars = ["TURSO_TOKEN", "TURSO_DATABASE", "FACEIT_TOKEN", "BOT_OWNER", "DISCORD_TOKEN"];
    for var in required_vars {
        if std::env::var(var).is_err() {
            panic!("Error: Environment variable '{}' is missing from .env or system environment.", var);
        }
    }

    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found");

    let intents = GatewayIntents::GUILD_MEMBERS |
        GatewayIntents::GUILD_MESSAGES |
        GatewayIntents::DIRECT_MESSAGES |
        GatewayIntents::MESSAGE_CONTENT |
        GatewayIntents::GUILDS;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                discord::commands::help(),
                discord::commands::link(),
                discord::commands::unlink(),
                discord::commands::status(),
                discord::commands::guilds(),
                discord::commands::leave(),
                discord::commands::forceunlink(),
                discord::commands::forcelink(),
                discord::commands::restore(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let mut client = Client::builder(token, intents)
        .framework(framework)
        .event_handler(DiscordBot)
        .await
        .expect("Err creating client");

    tokio::spawn(name_syncer(client.http.clone()));

    info!("Client starting...");
    client.start().await.map_err(|e| e.into())
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