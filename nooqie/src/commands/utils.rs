use serenity::{
    framework::standard::{
        CommandResult,
        macros::command
    },
    model::prelude::*,
    prelude::*
};

#[command]
#[description = "send ping, get pong"]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "pong!").await?;
    Ok(())
}


