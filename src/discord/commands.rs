use std::sync::Arc;
use regex::Regex;
use serenity::all::{GuildId, UserId};
use tracing::{error, info};
use crate::{Context, Error, PoiseContext};
use crate::database::Database;
use crate::discord::DiscordBot;

// Displays all commands
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: PoiseContext<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "Plumpen's spirit animal is a fat corgi.",
            ..Default::default()
        },
    )
        .await?;
    Ok(())
}

/// Links to Faceit account using Faceit username
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn link(
    ctx: PoiseContext<'_>,
    #[description = "Faceit username"] username: String
) -> Result<(), Error> {

    let author = ctx.author();

    let http = ctx.http();

    match DiscordBot::link_user(&username, http, author.id, Some(&ctx)).await {
        Ok(success) => {
            if success {
                info!("Successfully linked user: {}", author.name);
                ctx.say(format!("Successfully linked Discord user '{}' to Faceit account '{}'.", author.name, username)).await?;
            } else {
                error!("Error linking Discord user '{}' to Faceit account '{}'", author.name, username);
            }
        },
        Err(e) => {
            ctx.say(format!("Error when attempting to link Discord user '{}' to Faceit account '{}'.", author.name, username)).await?;
            error!("Error linking user {}", e);
        }
    }

    Ok(())
}

/// Unlinks from Faceit account
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn unlink(
    ctx: PoiseContext<'_>
) -> Result<(), Error> {

    let author = ctx.author();

    let http = ctx.http();

    let Ok(exists) = Database.user_exists(author.id.to_string()).await else {
        error!("Error checking if user exists");
        ctx.say("Whops! Something went wrong.").await?;
        return Ok(())
    };

    if !exists {
        ctx.say("User not linked. Please link using '!link *faceitUsername*'").await?;
        return Ok(())
    };

    let Ok(success) = Database.unlink_user(author.id.to_string()).await else {
        ctx.say(format!("Error when attempting to unlink user '{}'.", author.name)).await?;
        error!("Error unlinking user");
        return Ok(())
    };

    if success {
        ctx.say(format!("Successfully unlinked user '{}'.", author.name)).await?;
        info!("Attempting to clear nickname in all relevant guilds.");
        DiscordBot::clear_user(http, author.id).await;
    } else {
        ctx.say(format!("Error when attempting to unlink user '{}'.", author.name)).await?;
        error!("Error unlinking user");
    }

    Ok(())
}

/// Displays info about bot
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn status(
    ctx: PoiseContext<'_>
) -> Result<(), Error> {

    let http = ctx.http();

    let Ok(user_count) = Database.count_users().await else {
        error!("Error counting users");
        ctx.say("Whops! Something went wrong.").await?;
        return Ok(());
    };

    let Ok(guilds) = http.get_guilds(None, Some(100)).await else {
        error!("Error attempting to get guilds.");
        ctx.say("Whops! Something went wrong.").await?;
        return Ok(());
    };

    ctx.say(format!("Connected to {} guilds. Total of {} users linked.", guilds.len(), user_count)).await?;

    Ok(())
}

/// Displays info about guilds which bot is member of
#[poise::command(prefix_command, track_edits, slash_command, owners_only)]
pub async fn guilds(
    ctx: PoiseContext<'_>
) -> Result<(), Error> {

    let http = ctx.http();

    let Ok(guilds) = http.get_guilds(None, Some(100)).await else {
        error!("Error attempting to get guilds.");
        ctx.say("Whops! Something went wrong.").await?;
        return Ok(());
    };

    let mut message = String::from("# Guilds \n");

    for guild_info in guilds.iter() {
        message.push_str(format!("**Guild**: '{}', **ID**: '{}'.\n", guild_info.name, guild_info.id).as_str());
    }

    ctx.say(message).await?;

    Ok(())
}

/// Removes bot from guild by ID
#[poise::command(prefix_command, track_edits, slash_command, owners_only)]
pub async fn leave(
    ctx: PoiseContext<'_>,
    #[description = "Guild ID"] guild_id: String
) -> Result<(), Error> {

    let http = ctx.http();

    let Ok(u64_id) = guild_id.parse::<u64>() else {
        ctx.say("Guild ID not in valid format.").await?;
        return Ok(());
    };

    let Ok(guild) = http.get_guild(GuildId::new(u64_id)).await else {
        ctx.say("Guild not found.").await?;
        return Ok(());
    };


    match guild.leave(http).await {
        Ok(_) => {
            info!("Left guild: '{}'", u64_id);
            ctx.say("Left guild.").await?;
        },
        _ => {
            error!("Error leaving guild: '{}'", u64_id);
            ctx.say("Error when leaving guild.").await?;
        }
    }
    Ok(())
}

/// Force unlinks another user from a Faceit account
#[poise::command(prefix_command, track_edits, slash_command, owners_only)]
pub async fn forceunlink(
    ctx: PoiseContext<'_>,
    #[description = "User ID (u64)"] user_id: String
) -> Result<(), Error> {

    let http = ctx.http();

    let Ok(u64_id) = user_id.parse::<u64>() else {
        ctx.say("User ID not in valid format.").await?;
        return Ok(());
    };

    let exists = Database.user_exists(user_id).await?;

    if !exists {
        ctx.say("User not linked.").await?;
        return Ok(());
    }

    let Ok(success) = Database.unlink_user(user_id).await else {
        ctx.say(format!("Error when attempting to force unlink user '{}'.", u64_id)).await?;
        error!("Error force unlinking user");
        return Ok(());
    };

    if success {
        ctx.say(format!("Successfully force unlinked user '{}'.", u64_id)).await?;
        info!("Attempting to clear nickname in all relevant guilds.");
        DiscordBot::clear_user(http, UserId::new(u64_id)).await;
    } else {
        ctx.say(format!("Error when attempting to force unlink user '{}'.", u64_id)).await?;
        error!("Error unlinking user");
    }

    Ok(())
}

/// Force links another user to a Faceit account
#[poise::command(prefix_command, track_edits, slash_command, owners_only)]
pub async fn forcelink(
    ctx: PoiseContext<'_>,
    #[description = "Faceit username"] username: String,
    #[description = "User ID (u64)"] user_id: String
) -> Result<(), Error> {

    let http = ctx.http();

    let Ok(u64_id) = user_id.parse::<u64>() else {
        ctx.say("User ID not in valid format.").await?;
        return Ok(());
    };

    match DiscordBot::link_user(&username, http, UserId::new(u64_id), Some(&ctx)).await {
        Ok(success) => {
            if success {
                info!("Successfully force linked user: {}", u64_id);
                ctx.say(format!("Successfully force linked Discord user '{}' to Faceit account '{}'.", user_id, username)).await?;
            } else {
                error!("Error linking Discord user '{}' to Faceit account '{}'", u64_id, username);
            }
        },
        Err(e) => {
            ctx.say(format!("Error when attempting to forcefully link Discord user '{}' to Faceit account '{}'.", user_id, username)).await?;
            error!("Error linking user {}", e);
        }
    }
    Ok(())
}

/// Restores links based on user nicknames
///
/// Can be used if database is lost
#[poise::command(prefix_command, track_edits, slash_command, owners_only)]
pub async fn restore(
    ctx: PoiseContext<'_>,
) -> Result<(), Error> {

    let http = ctx.http();

    info!("Attempting to restore user links from member nicknames");

    let Ok(guilds) = http.get_guilds(None, Some(100)).await else {
        ctx.say("Error attempting to get guilds.").await?;
        return Ok(());
    };

    let mut counter = 0;
    let mut error_counter = 0;
    let mut add_counter = 0;
    let mut total_counter = 0;

    let refresh_regex = Regex::new(r"\(\d+ ELO\)\s+([A-Za-z0-9-_]+)").unwrap();

    for guild_info in guilds.iter() {

        let Ok(guild) = http.get_guild(guild_info.id).await else {
            error!("Error attempting to get guild.");
            continue;
        };

        let Ok(members) = guild.members(http, None, None).await else {
            error!("Could not get members from guild {}.", guild.name);
            continue;
        };

        for member in members.iter() {

            total_counter += 1;

            let Some(nickname) = &member.nick else {
                continue;
            };

            let Some(caps) = refresh_regex.captures(nickname) else {
                continue;
            };

            let Some(username) = caps.get(1) else {
                error!("Error getting username from regex capture");
                continue;
            };

            let parsed_username = username.as_str().to_string();

            counter += 1;

            if !nickname.ends_with(parsed_username.as_str()) {
                error_counter += 1;
                error!("Issue with regex found, missed '{}', captured: '{}'", nickname, parsed_username);
                continue;
            }

            let result = DiscordBot::link_user(&parsed_username, http, member.user.id, None).await;

            match result {
                Ok(success) => {
                    if success {
                        add_counter += 1;
                    }
                }
                Err(_) => {
                    error_counter += 1;
                    error!("Error when linking user '{}' to faceit account '{}'", member.user.name, parsed_username);
                }
            }

        }

    }

    info!("Restore complete");
    ctx.say(format!("Restore complete. Total: {}, Assumed: {}, Added: {}, Errors: {}", total_counter, counter, add_counter, error_counter)).await?;

    Ok(())
}