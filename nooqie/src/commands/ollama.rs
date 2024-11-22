use log::{debug, error, warn};

use poise::serenity_prelude::OnlineStatus;
use poise::serenity_prelude::{ActivityData, CreateMessage};
use poise::CreateReply;

use regex::Regex;
use reqwest::Client;

use serde::{Deserialize, Serialize};

use std::env;

use crate::{Context, Error};

#[derive(Serialize, Deserialize)]
struct AIResponse {
    model: String,
    created_at: String,
    response: String,
    done: bool,
    done_reason: String,
    context: Vec<u64>,
    total_duration: u64,
    load_duration: u64,
    prompt_eval_count: u64,
    prompt_eval_duration: u64,
    eval_count: u64,
    eval_duration: u64,
}

#[poise::command(
    prefix_command,
    track_edits,
    aliases("ollama", "query"),
    broadcast_typing = true,
    slash_command,
    help_text_fn = llm_help
)]
pub async fn llm(
    ctx: Context<'_>,
    #[autocomplete = "poise::builtins::autocomplete_command"]
    #[rest]
    msg: Option<String>,
) -> Result<(), Error> {
    let ser_ctx: &poise::serenity_prelude::Context = ctx.serenity_context();
    let mut status: OnlineStatus = OnlineStatus::DoNotDisturb;
    let mut activity: ActivityData = ActivityData::custom("thinking...");
    ctx.serenity_context().set_presence(Some(activity), status);

    let prompt = match msg {
        Some(prompt) => prompt,
        None => {
            warn!("no prompt provided");
            return Ok(());
        }
    };

    debug!("{}: prompt '{}'", ctx.channel_id(), &prompt);

    let new_msg = ctx.say("...").await.expect("");

    let anwser = match prompt_ollama(prompt).await {
        Ok(anwser) => anwser,
        Err(error) => {
            warn!("failed to get response: {}", error);
            return Ok(());
        }
    };

    debug!("{}: anwser '{}'", ctx.channel_id(), anwser);

    let anwser_length = anwser.len();
    if anwser_length > 2000 {
        let mut anwsers: Vec<String> = Vec::new();
        for i in (0..anwser_length).step_by(1000) {
            anwsers.push(anwser[i..std::cmp::min(i + 1000, anwser_length)].to_string());
        }
        for message in anwsers {
            debug!("{message}");
            let builder = CreateMessage::new().content(message);
            let _ = ctx.channel_id().send_message(ctx.http(), builder).await;
        }
        return Ok(());
    };

    let builder = CreateReply::default().content(anwser.clone());

    if let Err(error) = new_msg.edit(ctx, builder).await {
        if error.to_string() == "Unknown Message" {
            warn!("original message deleted sending new message");
            ctx.say(anwser).await?;
        }
        error!("Error sending message: {error:?}");
    }
    status = OnlineStatus::Online;
    activity = ActivityData::custom("");
    ser_ctx.set_presence(Some(activity), status);
    Ok(())
}

pub fn llm_help() -> String {
    String::from("queries offline local Ollama instance")
}

pub async fn prompt_ollama(prompt: String) -> Result<String, Error> {
    let model = env::var("OLLAMA_MODEL").expect("'OLLAMA_MODEL' environment variable not set");

    let post_url = env::var("OLLAMA_POST_URL").expect("'OLLAMA_IP' environment variable not set");

    let client: Client = Client::new();

    let model = json_strip_escape(&model);
    let prompt_fmt = json_strip_escape(&prompt);

    let response = client
        .post(post_url)
        .body(format!(
            r##"{{"model": "{model}", "prompt": "{prompt_fmt}", "stream": false }}"##,
            model = model,
            prompt_fmt = prompt_fmt
        ))
        .send()
        .await;

    match response {
        Ok(t) => {
            let response_text = t
                .text()
                .await
                .expect("Invalid response could not parse to text");
            let air: AIResponse = serde_json::from_str(&response_text)?;
            Ok(air.response)
        }
        Err(error) => {
            error!("failed to connect to Ollama server: {error}");
            Ok(String::from("I seem to have dropped my brain :brain:"))
        }
    }
}

pub fn json_strip_escape(string: &str) -> String {
    // TODO:
    // * create vector with all regex strings
    // * loop through
    // * replace 'unwrap's with match expressions or if let statements
    // * change return value to a 'Result<_, err>'
    let re_reverse_solidus = Regex::new(r#"\\"#).unwrap();
    let re_solidus = Regex::new(r#"/"#).unwrap();
    let re_quotation_mark = Regex::new(r#"""#).unwrap();
    // let re_backspace = Regex::new(r#"\b"#).unwrap();
    // let re_formfeed = Regex::new(r#"\f"#).unwrap();
    let re_linefeed = Regex::new(r#"\n"#).unwrap();
    // let re_carriage_return = Regex::new(r#"\r"#).unwrap();
    // let re_horizontal_tab = Regex::new(r#"\t"#).unwrap();
    // let re_hex = Regex::new(r#"/\u[a-fA-F0-9]{8}"#).unwrap();

    let string = re_linefeed.replace_all(&string, r#" "#);
    // let string = re_backspace.replace_all(&string, r#" "#);
    // let string = re_formfeed.replace_all(&string, r#" "#);
    // let string = re_carriage_return.replace_all(&string, r#" "#);
    // let string = re_horizontal_tab.replace_all(&string, r#"    "#);

    let string = re_reverse_solidus.replace_all(&string, r#"\\"#);
    let string = re_solidus.replace_all(&string, r#"\/"#);
    let string = re_quotation_mark.replace_all(&string, r#"\""#);
    String::from(string)
}
