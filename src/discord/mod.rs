use std::collections::HashMap;
use std::time::Duration;
use serenity::all::{Context, EventHandler, Message, Ready, Guild, UnavailableGuild, RoleId, Role, EditRole};
use serenity::async_trait;
use tokio::time::sleep;
use tracing::{error, info};

pub struct DiscordBot;

#[async_trait]
impl EventHandler for DiscordBot {
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        info!("Connection to guild '{}' established!", guild.name);

        prepare_guild(ctx, guild).await;
    }

    async fn guild_delete(&self, ctx: Context, incomplete: UnavailableGuild, full: Option<Guild>) {

        let guild_identifier;

        if let Some(guild) = full {
            guild_identifier = guild.name;
        } else {
            guild_identifier = incomplete.id.to_string();
        }

        if incomplete.unavailable {
            info!("Connection to guild '{}' lost!", guild_identifier);
        } else {
            info!("Kicked from guild '{}'!", guild_identifier);
        }

    }

    async fn message(&self, ctx: Context, msg: Message) {
        info!(msg.content);
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "asd!").await {
                error!("Error sending message: {:?}", e);
            }
        }
    }


    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

async fn prepare_guild(ctx: Context, guild: Guild){

    info!("Preparing guild '{}' with ID '{}'!", guild.name, guild.id);

    if guild.id != 1322261733053825086 {
        info!("Testing mode on, returning.");
        return;
    }

    let required_roles: Vec<(&str, &str)> = vec![
        ("Level 1 (1-800 ELO)", "#dddddd"),
        ("Level 2 (801-950 ELO)", "#47e36e"),
        ("Level 3 (951-1100 ELO)", "#47e36e"),
        ("Level 4 (1101-1250 ELO)", "#ffcd25"),
        ("Level 5 (1251-1400 ELO)", "#ffcd25"),
        ("Level 6 (1401-1550 ELO)", "#ffcd25"),
        ("Level 7 (1551-1700 ELO)", "#ffcd25"),
        ("Level 8 (1701-1850 ELO)", "#ff6c20"),
        ("Level 9 (1851-2000 ELO)", "#ff6c20"),
        ("Level 10 (2001+ ELO)", "#e80128"),
    ];

    let roles: &HashMap<RoleId, Role> = &guild.roles;
    let mut actual_roles: HashMap<&str, RoleId> = HashMap::new();

    for (role, color) in &required_roles {
        if let Some((key, value)) = roles.iter().find(|(_, &ref v)| v.name.as_str() == *role) {
            actual_roles.insert(*role, *key);
            info!("Found: key = {}, value = {}", key, value.name);
        } else {
            info!("Role '{}' not found, attempting to create!",role);
            let builder = EditRole::new().name(role).colour(color).hoist(true).mentionable(true);
            let new_role = guild.create_role(&ctx.http, builder).await;
            if new_role.is_err() {
                error!("Failed to create role in guild '{}'", guild.name);
                return;
            } else {
                // There are better ways to make sure you don't hit rate limit.
                // But we don't do that here.
                sleep(Duration::from_millis(40)).await;
                actual_roles.insert(*role, new_role.unwrap().id);
            }
            return;
        }
    }

    //guild.create_role()

    info!("Guild prepared!");
}