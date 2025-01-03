use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use serenity::all::{Context, EventHandler, Message, Ready, Guild, UnavailableGuild, RoleId, Role, EditRole, GuildId};
use serenity::model::Colour;
use serenity::async_trait;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, info};

pub struct DiscordBot{
    prepared_guilds: Arc<Mutex<HashMap<GuildId, HashMap<&'static str, RoleId>>>>,
}

impl DiscordBot {
    pub fn new() -> Self {
        Self {
            prepared_guilds: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl EventHandler for DiscordBot {
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {

        info!("Connection to guild '{}' established!", guild.name);

        let mut data = self.prepared_guilds.lock().await;

        if !data.contains_key(&guild.id) {
            info!("Guild '{}' already prepared!", guild.name);
            return;
        }

        let guild_roles = prepare_guild(ctx, &guild).await;

        match guild_roles {
            Some(value) => {
                data.insert(guild.id, value);
            },
            None => {
                error!("Failed to prepare guild '{}'", guild.name)
            },
        }

    }

    async fn guild_delete(&self, ctx: Context, incomplete: UnavailableGuild, full: Option<Guild>) {

        let mut data = self.prepared_guilds.lock().await;
        if !data.contains_key(&incomplete.id) {
            return;
        }

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

        data.remove(&incomplete.id);

        info!("Guild '{}' removed from prepared list!", guild_identifier);

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

async fn prepare_guild(ctx: Context, guild: &Guild) -> Option<HashMap<&'static str, RoleId>> {

    info!("Preparing guild '{}' with ID '{}'!", guild.name, guild.id);

    if guild.id != 1322261733053825086 {
        info!("Testing mode on, returning.");
        return None;
    }

    let required_roles: Vec<(&str, u32)> = vec![
        ("Level 10 (2001+ ELO)", 0xE80128),
        ("Level 9 (1851-2000 ELO)", 0xFF6C20),
        ("Level 8 (1701-1850 ELO)", 0xFF6C20),
        ("Level 7 (1551-1700 ELO)", 0xFFCD25),
        ("Level 6 (1401-1550 ELO)", 0xFFCD25),
        ("Level 5 (1251-1400 ELO)", 0xFFCD25),
        ("Level 4 (1101-1250 ELO)", 0xFFCD25),
        ("Level 3 (951-1100 ELO)", 0x47E36E),
        ("Level 2 (801-950 ELO)", 0x47E36E),
        ("Level 1 (1-800 ELO)", 0xDDDDDD),
    ];

    let roles: &HashMap<RoleId, Role> = &guild.roles;
    let mut actual_roles: HashMap<&str, RoleId> = HashMap::new();

    for (role, color) in &required_roles {
        if let Some((key, value)) = roles.iter().find(|(_, &ref v)| v.name.as_str() == *role) {
            actual_roles.insert(*role, *key);
            info!("Found: key = {}, value = {}", key, value.name);
        } else {
            info!("Role '{}' not found, attempting to create!",role);

            let builder = EditRole::new().name(role.to_string()).colour(Colour::new(*color)).hoist(true).mentionable(true);
            let new_role = guild.create_role(&ctx.http, builder).await;

            match new_role {
                Ok(value) => {
                    actual_roles.insert(*role, value.id);

                    // There are better ways to make sure you don't hit rate limit.
                    // But we don't do that here.
                    sleep(Duration::from_millis(40)).await;
                },
                Err(e) => {
                    error!("Failed to create role in guild '{}' reason: {}", guild.name, e);
                    return None;
                },
            }
        }
    }

    info!("Guild prepared!");

    Some(actual_roles)

}