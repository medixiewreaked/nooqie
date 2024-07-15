#![allow(deprecated)]

use log::{error, info};

use serenity::async_trait;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::{Configuration, StandardFramework};
use serenity::model::gateway::Ready;

use serenity::prelude::*;

use songbird::SerenityInit;

use std::env;
use std::error::Error;

mod commands;

use crate::commands::utils::*;
use crate::commands::ollama::*;
use crate::commands::voice::*;

use reqwest::Client as HttpClient;

#[group]
#[commands(ping, llm, join, leave, play)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected", ready.user.name);
    }
}

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
            .type_map_insert::<HttpKey>(HttpClient::new())
            .await
            .expect("Err creating client");

    tokio::spawn(async move {
        let _ = client
            .start()
            .await
            .map_err(|why| error!("client ended: {:?}", why));
    });

    let _signal_err = tokio::signal::ctrl_c().await;
    info!("Received Ctrl-C, shutting down.");

    Ok(())
}
