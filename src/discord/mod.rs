use std::collections::HashMap;
use std::time::Duration;
use serenity::all::{Context, EventHandler, Message, Ready, Guild, UnavailableGuild, RoleId, Role, EditRole, GuildId};
use serenity::model::Colour;
use serenity::async_trait;
use tokio::sync::Mutex;
use std::sync::Arc;
use anyhow::Error;
use tokio::time::sleep;
use tracing::{error, info};
use regex::Regex;
use crate::database::Database;
use crate::faceit::Faceit;

pub struct DiscordBot{
    prepared_guilds: Arc<Mutex<HashMap<GuildId, HashMap<&'static str, RoleId>>>>,
    database: Arc<Mutex<Database>>,
    faceit: Faceit
}

impl DiscordBot {
    pub fn new(database: Arc<Mutex<Database>>, faceit: Faceit) -> Self {
        Self {
            prepared_guilds: Arc::new(Mutex::new(HashMap::new())),
            database,
            faceit
        }
    }

    pub async fn link_user(&self, parsed_username: &String, ctx: &Context, msg: &Message) -> Result<bool, Error>{

        let player_data = self.faceit.get_faceit_user_by_nickname(parsed_username.to_string()).await?;

        let db = self.database.lock().await;

        let exists = db.user_exists(msg.author.id.to_string()).await?;

        if exists {
            msg.channel_id.say(&ctx.http, "User already linked, unlink using '!unlink'.").await?;
            return Ok(false);
        };

        let success = db.add_user(player_data.player_id.to_string(), msg.author.id.to_string()).await?;

        Ok(success)
    }

}

#[async_trait]
impl EventHandler for DiscordBot {
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {

        info!("Connection to guild '{}' established!", guild.name);

        let mut data = self.prepared_guilds.lock().await;

        if data.contains_key(&guild.id) {
            info!("Guild '{}' already prepared!", guild.name);
            return;
        }

        let guild_roles = prepare_guild(ctx, &guild).await;

        match guild_roles {
            Some(value) => {
                data.insert(guild.id, value);
                info!("Guild prepared successfully. Total number of prepared guilds: {}", data.len());
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

        // Debug
        info!(msg.content);

        let link_regex = Regex::new(r"^!link (\S+)$").unwrap();

        if msg.content == "!status" {

            let data = self.prepared_guilds.lock().await;
            let db = self.database.lock().await;
            let Ok(user_count) = db.count_users().await else {
                error!("Error counting users");
                return;
            };
            if let Err(e) = msg.channel_id.say(&ctx.http, format!("Connected to {} guilds. Total of {} users linked.", data.len(), user_count)).await {
                error!("Error sending message: {:?}", e);
            }

        } else if msg.content == "!unlink" {

            let db = self.database.lock().await;

            let Ok(exists) = db.user_exists(msg.author.id.to_string()).await else {
                error!("Error checking if user exists");
                return;
            };

            if !exists {
                if let Err(e) = msg.channel_id.say(&ctx.http,"User not linked. Please link using '!link *faceitUsername*'").await {
                    error!("Error sending message: {:?}", e);
                }
                return;
            }

            let Ok(success) = db.unlink_user(msg.author.id.to_string()).await else {
                if let Err(e) = msg.channel_id.say(&ctx.http,format!("Error when attempting to unlink user '{}'.", msg.author.name)).await {
                    error!("Error sending message: {:?}", e);
                }
                error!("Error unlinking user");
                return;
            };

            if success {
                if let Err(e) = msg.channel_id.say(&ctx.http,format!("Successfully unlinked user '{}'.", msg.author.name)).await {
                    error!("Error sending message: {:?}", e);
                }
                error!("Error unlinking user");
            } else {
                if let Err(e) = msg.channel_id.say(&ctx.http,format!("Error when attempting to unlink user '{}'.", msg.author.name)).await {
                    error!("Error sending message: {:?}", e);
                }
                error!("Error unlinking user");
            }

        } else if let Some(caps) = link_regex.captures(&*msg.content) {

            let Some(username) = caps.get(1) else {
                error!("Error getting username from regex capture");
                return;
            };

            let Ok(parsed_username) = username.as_str().parse() else {
                error!("Error parsing username");
                return;
            };

            match self.link_user(&parsed_username, &ctx, &msg).await {
                Ok(success) => {
                    if success {
                        info!("Successfully linked user: {}", msg.author.name);
                        if let Err(e) = msg.channel_id.say(&ctx.http,format!("Successfully linked Discord user '{}' to Faceit account '{}'.", msg.author.name, parsed_username)).await {
                            error!("Error sending message: {:?}", e);
                        }
                    } else {
                        error!("Error linking Discord user '{}' to Faceit account '{}'", msg.author.name, parsed_username);
                    }
                },
                Err(e) => {
                    if let Err(e) = msg.channel_id.say(&ctx.http,format!("Error when attempting to link Discord user '{}' to Faceit account '{}'.", msg.author.name, parsed_username)).await {
                        error!("Error sending message: {:?}", e);
                    }
                    error!("Error linking user {}", e);
                }
            }

        }

    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

}

async fn prepare_guild(ctx: Context, guild: &Guild) -> Option<HashMap<&'static str, RoleId>> {

    info!("Preparing guild '{}' with ID '{}'!", guild.name, guild.id);

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
        if let Some((key, _value)) = roles.iter().find(|(_, &ref v)| v.name.as_str() == *role) {
            actual_roles.insert(*role, *key);
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