use std::sync::Arc;
use tracing::{error, info};
use crate::{Context, Error, PoiseContext};
use crate::database::Database;
use crate::discord::DiscordBot;

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
            extra_text_at_bottom: "This is an example bot made to showcase features of my custom Discord bot framework",
            ..Default::default()
        },
    )
        .await?;
    Ok(())
}

/// This is the description of my cool command, it can span multiple
/// lines if you need to
///
/// Here in the following paragraphs, you can give information on how \
/// to use the command that will be shown in your command's help.
///
/// You could also put example invocations here:
/// `/link Verity`
#[poise::command(prefix_command, track_edits, slash_command, description_localized("en", "Links to Faceit Account"))]
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
    ctx: PoiseContext<'_>,
) -> Result<(), Error> {

    let author = ctx.author();

    let http = ctx.http();

    let Ok(exists) = Database.user_exists(author.id.to_string()).await else {
        error!("Error checking if user exists");
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