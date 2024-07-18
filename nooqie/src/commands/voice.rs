use log::{debug, error};

use serenity::model::channel::Message;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::async_trait;
use serenity::prelude::TypeMapKey;

use songbird::events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};
use songbird::input::YoutubeDl;
use songbird::Songbird;

use serenity::client::Context;
use serenity::all::GuildId;

use reqwest::Client as HttpClient;
use std::sync::Arc;

pub struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let (guild_id, channel_id) = {
        let guild = msg.guild(&ctx.cache).unwrap();
        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_states| voice_states.channel_id);
        (guild.id, channel_id)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            error!("user not in voice channel");
            return Ok(());
        }
    };

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    debug!("joining channel");
    if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            error!("failed to disconnect: {:?}", e);
        }
        debug!("disconnected from voice channel");
    } else {
        error!("not in voice channel");
    }

    Ok(())
}

struct TrackErrorNotifier;

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                error!("track {:?} encounted an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }
        None
    }
}

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let (guild_id, channel_id) = {
        let guild = msg.guild(&ctx.cache).unwrap();
        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_states| voice_states.channel_id);
        (guild.id, channel_id)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            error!("user not in voice channel");
            return Ok(());
        }
    };

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            error!("must provide a URL to a video or audio");
            return Ok(());
        }
    };

    debug!("joining channel");
    if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
    }

    let guild_id = msg.guild_id.unwrap();

    let http_client = {
        let data = ctx.data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let src = YoutubeDl::new(http_client, url);
        let song = handler.enqueue_input(src.into()).await;
        debug!("added audio to queue");
        let _ = song.add_event(
            Event::Track(TrackEvent::End),
            SongEndLeaver {
                manager,
                guild_id
            },
        );
    } else {
        debug!("no search available");
    }

    Ok(())
}

struct SongEndLeaver {
    manager: Arc<Songbird>,
    guild_id: GuildId
}

#[async_trait]
impl VoiceEventHandler for SongEndLeaver {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {

        if let Some(handler_lock) = self.manager.get(self.guild_id) {
            let handler = handler_lock.lock().await;
            if handler.queue().len() > 0 {
                return None
            }
        }

        let has_handler = self.manager.get(self.guild_id).is_some();

        if has_handler {
            if let Err(e) = self.manager.remove(self.guild_id).await {
                error!("failed to disconnect: {:?}", e);
            }
            debug!("disconnected from voice channel");
        } else {
            error!("not in voice channel");
        }

        None
    }
}

#[command]
#[only_in(guilds)]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        debug!("skipping audio track");
        let _ = queue.skip();
    } else {
        error!("failed to skip audio track");
    }

    Ok(())
}
