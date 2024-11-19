use crate::{Context, Error};

#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    broadcast_typing = true,
    category = "Utility"
)]
pub async fn ping(
    ctx: Context<'_>,
    #[description = "send ping, get pong"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    _command: Option<String>,
) -> Result<(), Error> {
    ctx.say("pong!").await?;
    Ok(())
}

#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    broadcast_typing = true,
    category = "Utility"
)]
pub async fn pong(
    ctx: Context<'_>,
    #[description = "send pong, get ping"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    _command: Option<String>,
) -> Result<(), Error> {
    ctx.say("ping!").await?;
    Ok(())
}

#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    broadcast_typing = true,
    category = "Utility"
)]
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
