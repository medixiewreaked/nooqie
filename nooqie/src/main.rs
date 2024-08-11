#![allow(deprecated)]

use clap::{crate_description,Parser};

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

use std::{collections::{ HashMap, HashSet }, env, env::var, sync::{Arc, Mutex}, time::Duration};

mod commands;

use crate::commands::{ollama::*, utils::*, voice::*};

use reqwest::Client as HttpClient;

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {
    votes: Mutex<HashMap<String, u32>>,
}

#[derive(Parser, Debug)]
#[command(about=crate_description!())]
#[command(version, long_about = None)]
struct CLArgs {
    #[arg(short, long, default_value = "none")]
    loglevel: String,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

// #[group]
// #[commands(ping, llm, join, leave, play, skip, clear, pause, resume)]
// struct General;

// struct Handler;

// #[async_trait]
// impl EventHandler for Handler {
//     async fn ready(&self, _: Context, ready: Ready) {
//         info!("{} is connected", ready.user.name);
//     }
// }

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data
    ) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            info!("{} is connected", data_about_bot.user.name);
        }
        _ => {}
    }
    Ok(())
}

// #[help]
// async fn nooqie_help(
//     context: &Context,
//     msg: &Message,
//     args: Args,
//     help_options: &'static HelpOptions,
//     groups: &[&'static CommandGroup],
//     owners: HashSet<UserId>,
// ) -> CommandResult {
//     let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
//     Ok(())
// }

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

//     let framework = StandardFramework::new()
//         .group(&GENERAL_GROUP)
//         .help(&NOOQIE_HELP);
//
    let prefix = match env::var("NOOQIE_PREFIX") {
        Ok(prefix) => {
            prefix
        }
        Err(_error) => {
            String::from("!")
        }
    };

    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    let options = poise::FrameworkOptions {
        // commands: vec![commands::help(), commands::vote(), commands::getvotes()],
        commands: vec![],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some(prefix.into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            additional_prefixes: vec![
                poise::Prefix::Literal("hey bot"),
                poise::Prefix::Literal("hey bot,"),
            ],
            ..Default::default()
        },
        // The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),
        // This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        // This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        // Every command invocation must pass this check to continue execution
//         command_check: Some(|ctx| {
//             Box::pin(async move {
//                 if ctx.author().id == 123456789 {
//                     return Ok(false);
//                 }
//                 Ok(true)
//             })
//         }),
        // Enforce command checks even for owners (enforced by default)
        // Set to true to bypass checks, which is useful for testing
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
                Ok(Data {
                    votes: Mutex::new(HashMap::new()),
                })
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
