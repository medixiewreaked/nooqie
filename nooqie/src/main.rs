#![allow(deprecated)]

use clap::{crate_description, Parser};

use env_logger::Builder;

use log::{error, info, LevelFilter};

//use serenity::{
//    async_trait,
//    framework::standard::{
//        help_commands,
//        macros::{group, help},
//        Args, CommandGroup, CommandResult, Configuration, HelpOptions, StandardFramework,
//    },
//    model::{
//        gateway::Ready,
//        prelude::{Message, UserId},
//    },
//    prelude::*,
//};
use poise::serenity_prelude as serenity;

use songbird::SerenityInit;

use std::{
    collections::{HashMap, HashSet},
    env,
    env::var,
    sync::{Arc, Mutex},
    time::Duration,
};

mod commands;

use crate::commands::{ollama::*, utils::*, voice::*};

use reqwest::Client as HttpClient;

use nooqie::{Context, Data, Error};

#[derive(Parser, Debug)]
#[command(about=crate_description!())]
#[command(version, long_about = None)]
struct CLArgs {
    #[arg(short, long, default_value = "none")]
    loglevel: String,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e)
            }
        }
    }
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            info!("{} is connected", data_about_bot.user.name);
        }
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let clargs = CLArgs::parse();
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    let _ = dotenvy::dotenv();
    let token = env::var("DISCORD_TOKEN").expect("'DISCORD_TOKEN' environment variable not set");

    if clargs.loglevel != "none" {
        let mut builder = Builder::new();

        match clargs.loglevel.to_lowercase().as_str() {
            "trace" => {
                builder.filter_module("nooqie", LevelFilter::Trace).init();
            }
            "debug" => {
                builder.filter_module("nooqie", LevelFilter::Debug).init();
            }
            "info" => {
                builder.filter_module("nooqie", LevelFilter::Info).init();
            }
            "warn" => {
                builder.filter_module("nooqie", LevelFilter::Warn).init();
            }
            "error" => {
                builder.filter_module("nooqie", LevelFilter::Error).init();
            }
            &_ => {}
        }
    } else {
        let env = env_logger::Env::new();
        env_logger::init_from_env(env);
    }

    info!("Starting...");

    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_VOICE_STATES;

    let prefix = match env::var("NOOQIE_PREFIX") {
        Ok(prefix) => prefix,
        Err(_error) => String::from("!"),
    };

    let options = poise::FrameworkOptions {
        commands: vec![
            help(),
            ping(),
            llm(),
            join(),
            leave(),
            play(),
            pause(),
            resume(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some(prefix.into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        pre_command: |ctx| {
            Box::pin(async move {
                info!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        post_command: |ctx| {
            Box::pin(async move {
                info!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        skip_checks_for_owners: false,
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .options(options)
        .build();

    let mut client = serenity::Client::builder(&token, intents)
        .framework(framework)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .await
        .expect("Error creating client");

    tokio::spawn(async move {
        let _ = client
            .start()
            .await
            .map_err(|why| error!("client ended: {:?}", why));
    });

    let _signal_err = tokio::signal::ctrl_c().await;
    info!("Received Ctrl-C, shutting down.");
}
