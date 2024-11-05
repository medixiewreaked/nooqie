use clap::{crate_description, Parser};

use env_logger::{Builder, Env};

use log::{debug, error, info, warn, LevelFilter};

use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{Client, GatewayIntents};

use songbird::SerenityInit;

use std::{env, sync::Arc, time::Duration};

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
        poise::FrameworkError::GuildOnly { ctx, .. } => {
            warn!("{}: bot not in guild", ctx.author());
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e)
            }
        }
    }
}

async fn event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            info!("{} is connected", data_about_bot.user.name);
        }
        serenity::FullEvent::ShardsReady { total_shards } => {
            info!("{} shards", total_shards);
        }
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let clargs: CLArgs = CLArgs::parse();
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    let _ = dotenvy::dotenv();
    let token: String =
        env::var("DISCORD_TOKEN").expect("'DISCORD_TOKEN' environment variable not set");

    if clargs.loglevel != "none" {
        let mut builder: Builder = Builder::new();

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
        let env: Env<'_> = Env::new();
        env_logger::init_from_env(env);
    }

    info!("Starting...");

    let intents: GatewayIntents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES;

    let prefix: String = match env::var("NOOQIE_PREFIX") {
        Ok(prefix) => prefix,
        Err(_error) => String::from("!"),
    };

    let options = poise::FrameworkOptions {
        commands: vec![
            help(),
            ping(),
            pong(),
            llm(),
            join(),
            leave(),
            play(),
            pause(),
            resume(),
            skip(),
            clear(),
            loop_track(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some(prefix),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        pre_command: |ctx| {
            Box::pin(async move {
                debug!("Executing {} ==========", ctx.command().qualified_name);
            })
        },
        post_command: |ctx| {
            Box::pin(async move {
                debug!("Executed {} ==========", ctx.command().qualified_name);
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

    let mut client: Client = Client::builder(&token, intents)
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
