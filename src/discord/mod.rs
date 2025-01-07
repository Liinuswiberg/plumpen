pub(crate) mod commands;

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;
use serenity::all::{Channel, Context, EditRole, ErrorResponse, EventHandler, Guild, GuildId, Http, Member, Message, PartialGuild, Ready, Role, RoleId, UnavailableGuild, User, UserId};
use serenity::model::{guild, Colour};
use serenity::{async_trait, http};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::{env, thread};
use std::future::Future;
use anyhow::Error;
use poise::Command;
use tokio::time::sleep;
use tracing::{error, info};
use regex::Regex;
use serenity::builder::EditMember;
use crate::{Data, PoiseContext};
use crate::database::Database;
use crate::faceit::{Faceit, Player};

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

pub struct DiscordBot;

impl DiscordBot {

    pub async fn link_user<T>(parsed_username: &String, http_t: T, discord_id: UserId, poise_ctx: Option<&PoiseContext<'_>>) -> Result<bool, Error>
    where
        T: AsRef<Http>,
    {

        let http: &Http = http_t.as_ref();

        let Some(player_data) = Faceit::get_faceit_user_by_nickname(parsed_username.to_string()).await? else {
            if let Some(px) = poise_ctx {
                px.say("Faceit account not found.").await?;
            }
            return Ok(false);
        };

        let exists = Database.user_exists(discord_id.to_string()).await?;

        if exists {
            if let Some(px) = poise_ctx {
                px.say("User already linked, unlink using '!unlink'.").await?;
            }
            return Ok(false);
        };

        if let None = player_data.get_player_skill_level() {
            if let Some(px) = poise_ctx {
                px.say("User has not played CS2 on Faceit.").await?;
            }
            return Ok(false);
        }

        let success = Database.add_user(player_data.player_id.to_string(),discord_id.to_string()).await?;

        Self::parse_user(http, discord_id, player_data).await;

        Ok(success)
    }

    pub async fn clear_user<T>(http_t: T, discord_id: UserId)
    where
        T: AsRef<Http>,
    {

        let http: &Http = http_t.as_ref();

        let Ok(guilds) = http.get_guilds(None, Some(100)).await else {
            error!("Error attempting to get guilds.");
            return;
        };

        for guild_info in guilds.iter() {

            let Ok(guild) = http.get_guild(guild_info.id).await else {
                error!("Error attempting to get guild.");
                continue;
            };

            let success = Self::edit_member(&http, &guild, discord_id, "", None).await;
            if success {
                //info!("Edited user in guild {} successfully.", guild.name);
            } else {
                error!("Error attempting to edit user in guild {}.", guild.name);
            }

            sleep(Duration::from_millis(30)).await;

        }

    }

    pub async fn parse_user<T>(http_t: T, user_id: UserId, player: Player)
    where
        T: AsRef<Http>,
    {

        let http: &Http = http_t.as_ref();

        // These might get triggered if user hasn't played cs2.
        let Some(level) = player.get_player_skill_level() else {
            error!("Unlinked user {} {:#?}", user_id, player);
            return;
        };
        let Some(elo) = player.get_player_elo() else {
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

            //info!("Attempting to edit user in guild {}.", guild.name);

            let success;

            match guild.role_by_name(suggested_role) {
                None => {
                    success = Self::edit_member(&http, &guild, user_id, suggested_name, None).await;
                }
                Some(role) => {
                    success = Self::edit_member(&http, &guild, user_id, suggested_name, Some(role.id)).await;
                }
            }

            if success {
                //info!("Renamed user in guild {} successfully.", guild.name);
            } else {
                error!("Error attempting to edit user in guild {}.", guild.name);
            }

            sleep(Duration::from_millis(30)).await;

        }

    }

    async fn edit_member<T>(http_t: T, guild: &PartialGuild, member_id: UserId, new_name: &str, role: Option<RoleId>) -> bool
    where
        T: AsRef<Http>,
    {

        let http: &Http = http_t.as_ref();

        // @TODO: Make this return a result. Result<bool, Error>. As "false" should be returned
        // if no edits were made. But now true is returned if no edits were made but user wasn't in guild.
        // Which works. But its rather ugly.

        let Ok(target_member) = guild.member(&http, &member_id).await else {
            //info!("User not in guild {}.", guild.name);
            return true;
        };

        /*

        @TODO: Permission check wont work with a partial guild. Find out how to do this.
        You can access channels using a partial guild using guild.channels
        This returns a hashmap, I could get a channel ID from there.
        But that feels so clunky...

        let Ok(current_user) = http.get_current_user().await else {
            error!("Error getting current user");
            return false;
        };

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
                //info!("Successfully edited guild member '{}' in guild '{}'.", member_id, guild.name);
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

    async fn ready(&self, _ctx: Context, ready: Ready) {
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