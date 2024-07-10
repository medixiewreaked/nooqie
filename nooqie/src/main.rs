#![allow(deprecated)]

use log::{error, info};

use serenity::async_trait;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::{Configuration, StandardFramework};

use serenity::prelude::*;

use songbird::SerenityInit;

use std::env;
use std::error::Error;

mod commands;

use crate::commands::utils::*;
use crate::commands::ollama::*;
use crate::commands::voice::*;

#[group]
#[commands(ping, llm, join)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler { }

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    rustls::crypto::ring::default_provider().install_default().expect("Failed to install rustls crypto provider");
    dotenvy::dotenv()?;
    let token = env::var("DISCORD_TOKEN")
        .expect("'DISCORD_TOKEN' environment variable not set");

    let env = env_logger::Env::new();

    env_logger::init_from_env(env);

    info!("Starting...");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_VOICE_STATES;

    let framework = StandardFramework::new()
        .group(&GENERAL_GROUP);

    framework.configure(Configuration::new()
                        .prefix("!"));

    let mut client =
        Client::builder(&token, intents)
            .event_handler(Handler)
            .framework(framework)
            .register_songbird()
            .await
            .expect("Err creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {why:?}");
    }

    Ok(())
}
