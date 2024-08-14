use crate::{Context, Error};
// use serenity::{
//     framework::standard::{macros::command, CommandResult},
//     model::prelude::*,
//     prelude::*,
// };
//
// #[command]
// #[description = "send ping, get pong"]
// pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
//     msg.channel_id.say(&ctx.http, "pong!").await?;
//     Ok(())
// }

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
