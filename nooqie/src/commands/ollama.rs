use log::{debug, error};
use serde::{Deserialize, Serialize};
use serenity::model::channel::Message;
use serenity::builder::CreateMessage;
use serenity::all::EditMessage;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::prelude::*;
use std::env;
use std::error::Error;

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
    Ok(())
}

async fn prompt_ollama(prompt: &str) -> Result<String, Box<dyn Error>> {
    let model = env::var("OLLAMA_MODEL")
        .expect("'OLLAMA_MODEL' environment variable not set");

    let post_url = env::var("OLLAMA_POST_URL")
        .expect("'OLLAMA_IP' environment variable not set");

    let client = reqwest::Client::new();

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
