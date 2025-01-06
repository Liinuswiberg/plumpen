use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;
use serenity::all::{Context, EventHandler, Message, Ready, Guild, UnavailableGuild, RoleId, Role, EditRole, GuildId, UserId, Http, Member, ErrorResponse, User, PartialGuild};
use serenity::model::{guild, Colour};
use serenity::{async_trait, http};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::thread;
use anyhow::Error;
use tokio::time::sleep;
use tracing::{error, info};
use regex::Regex;
use serenity::builder::EditMember;
use crate::database::Database;
use crate::faceit::{Faceit, Player};

pub struct DiscordBot;

const ALL_ROLES: &[&str] = &[
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

impl DiscordBot {

    pub async fn link_user(&self, parsed_username: &String, ctx: &Context, msg: &Message) -> Result<bool, Error>{

        let Some(player_data) = Faceit.get_faceit_user_by_nickname(parsed_username.to_string()).await? else {
            msg.channel_id.say(&ctx.http, "Faceit account not found.").await?;
            return Ok(false);
        };

        let exists = Database.user_exists(msg.author.id.to_string()).await?;

        if exists {
            msg.channel_id.say(&ctx.http, "User already linked, unlink using '!unlink'.").await?;
            return Ok(false);
        };

        let success = Database.add_user(player_data.player_id.to_string(), msg.author.id.to_string()).await?;

        let _ = self.parse_user(&ctx.http, &msg.author, player_data);

        Ok(success)
    }

    pub async fn clear_user(&self, http: &Arc<Http>, discord_id: UserId) -> () {

        let Ok(guilds) = http.get_guilds(None, Some(100)).await else {
            error!("Error attempting to get guilds.");
            return;
        };

        for guild_info in guilds.iter() {

            let Ok(guild) = http.get_guild(guild_info.id).await else {
                error!("Error attempting to get guild.");
                continue;
            };

            info!("Attempting to edit user in guild {}.", guild.name);
            let success = self.edit_member(&http, &guild, discord_id, "", None).await;
            if success {
                info!("Edited user in guild {} successfully.", guild.name);
            } else {
                error!("Error attempting to edit user in guild {}.", guild.name);
            }
        }

    }

    pub async fn parse_user(&self, http: &Arc<Http>, user: &User, player: Player) {

        let Some(level) = player.get_player_skill_level() else {
            error!("Could not get player skill level!");
            return;
        };
        let Some(elo) = player.get_player_elo() else {
            error!("Could not get player elo!");
            return;
        };

        let suggested_name: &str = &format!("({} ELO) {}", elo, player.nickname);

        let suggested_role: &str = ALL_ROLES.get(level - 1).unwrap_or(&"");

        let Ok(guilds) = http.get_guilds(None, Some(100)).await else {
            error!("Error attempting to get guilds.");
            return;
        };

        for guild_info in guilds.iter() {

            let Ok(guild) = http.get_guild(guild_info.id).await else {
                error!("Error attempting to get guild.");
                continue;
            };

            info!("Attempting to edit user in guild {}.", guild.name);

            let success;

            match guild.role_by_name(suggested_role) {
                None => {
                    success = self.edit_member(&http, &guild, user.id, suggested_name, None).await;
                }
                Some(role) => {
                    info!("{}", format!("Role id: {}", role.id));
                    success = self.edit_member(&http, &guild, user.id, suggested_name, Some(role.id)).await;
                }
            }

            if success {
                info!("Renamed user in guild {} successfully.", guild.name);
            } else {
                error!("Error attempting to edit user in guild {}.", guild.name);
            }
        }

    }

    async fn edit_member(&self, http: &Arc<Http>, guild: &PartialGuild, member_id: UserId, new_name: &str, role: Option<RoleId>) -> bool {

        let Ok(target_member) = guild.member(&http, &member_id).await else {
            info!("User not in guild {}.", guild.name);
            return true;
        };

        let Ok(current_user) = http.get_current_user().await else {
            error!("Error getting current user");
            return false;
        };

        /*

        @TODO: Permission check wont work with a partial guild. Find out how to do this.
        You can access channels using a partial guild using guild.channels
        This returns a hashmap, I could get a channel ID from there.
        But that feels so clunky...

        let Ok(guild_user) = guild.member(http, current_user.id).await else {
            error!("Error getting current user guild member object");
            return false;
        };

        let Some(default_channel) = guild.default_channel(current_user.id) else {
            error!("No default channel found in guild {}.", guild.name);
            return false;
        };

        let permissions = guild.user_permissions_in(default_channel, &*guild_user);

        if !permissions.manage_nicknames() && !permissions.manage_roles() {
            error!("Cannot manage nicknames and/or roles in guild {}.", guild.name);
            return false;
        }

         */

        if guild.owner_id == member_id {
            info!("Cannot rename owner in guild {}.", guild.name);
            return true;
        }

        /*

        let all_roles: &Vec<RoleId> = &prepared_guild.values().cloned().collect();

        let mut target_roles: Vec<RoleId> = target_member.roles.clone()
            .into_iter()
            .filter(|role| !all_roles.contains(role))
            .collect();

         */

        let mut target_roles = target_member.roles;

        let guild_roles = &guild.roles;

        for (key, role) in guild_roles.iter() {
            if ALL_ROLES.contains(&&*role.name) && target_roles.contains(key) {
                target_roles.retain(|&x| x.get() != key.get());
            }
        }

        if role.is_some() {
            target_roles.push(role.unwrap());
        }

        let result = guild.edit_member(http, member_id, EditMember::new().nickname(new_name).roles(target_roles)).await;

        match result {
            Ok(_) => {
                info!("Successfully edited guild member '{}' in guild '{}'.", member_id, guild.name);
                true
            },
            Err(e) => {
                error!("Error when attempting to edit guild member '{}' in guild '{}'.", member_id, guild.name);
                false
            }
        }

    }

}

#[async_trait]
impl EventHandler for DiscordBot {
    async fn guild_create(&self, ctx: Context, guild: Guild, _is_new: Option<bool>) {

        info!("Connection to guild '{}' established!", guild.name);

        let success = prepare_guild(ctx, &guild).await;

        if success {
            info!("Guild {} prepared successfully!", guild.name);
        } else {
            error!("Guild {} could not be prepared successfully!", guild.name);
        }

    }

    async fn guild_delete(&self, _ctx: Context, incomplete: UnavailableGuild, full: Option<Guild>) {

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

        // Debug
        info!(msg.content);

        let link_regex = Regex::new(r"^!link (\S+)$").unwrap();

        if msg.content == "!status" {

            let Ok(user_count) = Database.count_users().await else {
                error!("Error counting users");
                return;
            };

            let Ok(guilds) = ctx.http.get_guilds(None, Some(100)).await else {
                error!("Error attempting to get guilds.");
                return;
            };

            if let Err(e) = msg.channel_id.say(&ctx.http, format!("Connected to {} guilds. Total of {} users linked.", guilds.len(), user_count)).await {
                error!("Error sending message: {:?}", e);
            }

        } else if msg.content == "!unlink" {

            let Ok(exists) = Database.user_exists(msg.author.id.to_string()).await else {
                error!("Error checking if user exists");
                return;
            };

            if !exists {
                if let Err(e) = msg.channel_id.say(&ctx.http,"User not linked. Please link using '!link *faceitUsername*'").await {
                    error!("Error sending message: {:?}", e);
                }
                return;
            }

            let Ok(success) = Database.unlink_user(msg.author.id.to_string()).await else {
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
                info!("Attempting to clear nickname in all relevant guilds.");
                self.clear_user(&ctx.http, msg.author.id);
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

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

}

async fn prepare_guild(ctx: Context, guild: &Guild) -> bool {

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
                    return false;
                },
            }
        }
    }

    info!("Guild prepared!");

    true

}