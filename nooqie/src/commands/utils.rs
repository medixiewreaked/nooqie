use crate::{Context, Error};
use poise::serenity_prelude::standard::CommandResult;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn ping(
    ctx: Context<'_>,
    #[description = "send ping, get pong"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    _command: Option<String>,
) -> CommandResult {
    ctx.say("pong!").await?;
    Ok(())
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Show help"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom:
                "Nooqie is a Discord bot with basic LLM functionality though the Ollama API",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}
