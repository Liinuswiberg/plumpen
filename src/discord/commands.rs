use std::sync::Arc;
use tracing::{error, info};
use crate::{Context, Error, PoiseContext};
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

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn link(
    ctx: PoiseContext<'_>,
    username: String
) -> Result<(), Error> {

    let author = ctx.author();

    let http = Arc::new(ctx.http());

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