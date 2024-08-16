use log::{debug, error, warn};

use poise::serenity_prelude::standard::CommandResult;
use poise::serenity_prelude::ActivityData;
use poise::serenity_prelude::CreateMessage;
use poise::serenity_prelude::EditMessage;
use poise::serenity_prelude::OnlineStatus;
use poise::CreateReply;

use regex::Regex;

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

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn llm(
    ctx: Context<'_>,
    #[description = "queries offline local Ollama instance"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    #[rest]
    msg: Option<String>,
) -> CommandResult {
    //     let mut status = OnlineStatus::DoNotDisturb;
    //     let mut activity = ActivityData::custom("thinking...");
    //     ctx.set_presence(Some(activity), status);

    let prompt = &msg.unwrap();

    debug!("{}: prompt '{}'", ctx.channel_id(), prompt);

    let mut new_msg = ctx.say("...").await.expect("");

    let anwser = prompt_ollama(prompt).await.unwrap();

    debug!("{}: anwser '{}'", ctx.channel_id(), anwser);

    // let builder = EditMessage::new().content(anwser.clone());
    let builder = CreateReply::default().content(anwser.clone());

    if let Err(error) = new_msg.edit(ctx, builder).await {
        if error.to_string() == "Unknown Message" {
            warn!("original message deleted sending new message");
            ctx.say(anwser).await?;
        }
        error!("Error sending message: {error:?}");
    }
    //     status = OnlineStatus::Online;
    //     activity = ActivityData::custom("");
    //     ctx.set_presence(Some(activity), status);
    Ok(())
}

pub async fn prompt_ollama(prompt: &str) -> Result<String, Error> {
    let model = env::var("OLLAMA_MODEL").expect("'OLLAMA_MODEL' environment variable not set");

    let post_url = env::var("OLLAMA_POST_URL").expect("'OLLAMA_IP' environment variable not set");

    let client = reqwest::Client::new();

    let model = json_strip_escape(&model);
    let prompt = json_strip_escape(&prompt);

    let response = client
        .post(post_url)
        .body(format!(
            r##"{{"model": "{model}", "prompt": "{prompt}", "stream": false }}"##,
            model = model,
            prompt = prompt
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
            return Ok(air.response);
        }
        Err(error) => {
            warn!("failed to connect to Ollama server: {error}");
            return Ok(String::from("I seem to have dropped my brain :brain:"));
        }
    }
}

pub fn json_strip_escape(string: &str) -> String {
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
