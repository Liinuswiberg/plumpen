use std::collections::HashMap;
use serenity::all::{Context, EventHandler, Message, Ready, Guild, UnavailableGuild, RoleId, Role};
use serenity::async_trait;
use tracing::{error, info};

pub struct DiscordBot;

#[async_trait]
impl EventHandler for DiscordBot {
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        info!("Connection to guild '{}' established!", guild.name);
    }

    async fn guild_delete(&self, ctx: Context, incomplete: UnavailableGuild, full: Option<Guild>) {

        let guild_identifier;

        if let Some(guild) = full {
            guild_identifier = guild.name;
        } else {
            guild_identifier = incomplete.id.to_string();
        }

        if(incomplete.unavailable) {
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

async fn prepare_guild(guild: Guild){

    let required_roles = vec![
        "Level 1 (1-800 ELO)",
        "Level 2 (801-950 ELO)",
        "Level 3 (951-1100 ELO)",
        "Level 4 (1101-1250 ELO)",
        "Level 5 (1251-1400 ELO)",
        "Level 6 (1401-1550 ELO)",
        "Level 7 (1551-1700 ELO)",
        "Level 8 (1701-1850 ELO)",
        "Level 9 (1851-2000 ELO)",
        "Level 10 (2001+ ELO)",
    ];

    // 10 #e80128
    // 9 #ff6c20
    // 8 #ff6c20
    // 7 #ffcd25
    // 6 #ffcd25
    // 5 #ffcd25
    // 4 #ffcd25
    // 3 #47e36e
    // 2 #47e36e
    // 1 #dddddd


    let roles: HashMap<RoleId, Role> = guild.roles;


    //guild.create_role()

    info!("Preparing guild...");
}