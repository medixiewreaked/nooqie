#![allow(deprecated)]
use serenity::async_trait;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::{Configuration, StandardFramework};

use serenity::prelude::*;

use std::env;
use std::error::Error;

mod commands;

use crate::commands::utils::*;
use crate::commands::ollama::*;

#[group]
#[commands(ping, llm)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler { }

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;
    let token = env::var("DISCORD_TOKEN")
        .expect("'DISCORD_TOKEN' environment variable not set");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = StandardFramework::new()
        .group(&GENERAL_GROUP);

    framework.configure(Configuration::new()
                        .prefix("!"));

    let mut client =
        Client::builder(&token, intents)
            .event_handler(Handler)
            .framework(framework)
            .await
            .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }

    Ok(())
}
