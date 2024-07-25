use log::{
    debug,
    error
};

use regex::Regex;

use serde::{
    Deserialize,
    Serialize
};

use serenity::{
    all::EditMessage,
    builder::CreateMessage,
    framework::standard::{
        macros::command,
        CommandResult
    },
    gateway::ActivityData,
    model::{
        channel::Message,
        user::OnlineStatus,
    },
    prelude::*
};

use std::{
    env,
    error::Error
};

#[derive(Serialize, Deserialize)]
struct AIResponse {
    model: String,
    created_at: String,
    response: String,
    done: bool,
    done_reason: String,
    context: Vec<i64>,
    total_duration: i64,
    load_duration: i64,
    prompt_eval_count: i64,
    prompt_eval_duration: i64,
    eval_count: i64,
    eval_duration: i64
}

#[command]
pub async fn llm(ctx: &Context, msg: &Message) -> CommandResult {
    let mut status = OnlineStatus::DoNotDisturb;
    let mut activity = ActivityData::custom("thinking...");
    ctx.set_presence(Some(activity), status);

    let mut new_msg = msg.channel_id
        .send_message(ctx.clone(), CreateMessage::new().content("..."))
        .await
        .unwrap();

    let anwser = prompt_ollama(msg.content.strip_prefix("!llm ").expect("could not strip prefix '!llm '"))
        .await
        .unwrap();

    let builder = EditMessage::new()
        .content(anwser.clone());

    if let Err(why) = new_msg.edit(ctx.clone(), builder).await {
        if why.source().unwrap().to_string() == "Unknown Message" {
            debug!("original message deleted sending new message");
            new_msg.channel_id.say(&ctx.http, anwser).await?;
        }
        error!("Error sending message: {why:?}");
    }
    status = OnlineStatus::Online;
    activity = ActivityData::custom("");
    ctx.set_presence(Some(activity), status);
    Ok(())
}

pub async fn prompt_ollama(prompt: &str) -> Result<String, Box<dyn Error>> {
    let model = env::var("OLLAMA_MODEL")
        .expect("'OLLAMA_MODEL' environment variable not set");

    let post_url = env::var("OLLAMA_POST_URL")
        .expect("'OLLAMA_IP' environment variable not set");

    let client = reqwest::Client::new();

    let model = json_strip_escape(&model);
    let prompt = json_strip_escape(&prompt);

    debug!("[Ollama] prompt: '{}'", prompt);
    let response = client.post(post_url)
        .body(format!(r##"{{"model": "{model}", "prompt": "{prompt}", "stream": false }}"##, model=model, prompt=prompt))
        .send()
        .await;

    match response {
        Ok(t) => {
            let response_text = t.text()
                .await
                .expect("Invalid response could not parse to text");
            let air: AIResponse = serde_json::from_str(&response_text)?;
            debug!("[Ollama] received: '{}'", air.response);
            return Ok(air.response)

        },
        Err(e) => {
            error!("failed to connect to Ollama server: {e}");
            return Ok(String::from("I seem to have dropped my brain :brain:"))
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
