use log::{
    debug,
    error
};

use serenity::{
    all::{
        ActivityData,
        GuildId,
        OnlineStatus
    },
    async_trait,
    client::Context,
    framework::standard::{
        Args,
        CommandResult,
        macros::command
    },
    model::channel::Message,
    prelude::TypeMapKey
};

use songbird::{
    events::{
        Event,
        EventContext,
        EventHandler as VoiceEventHandler,
        TrackEvent
    },
    input::YoutubeDl,
    Songbird
};

use std::sync::Arc;

use reqwest::Client as HttpClient;

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

    let loop_amount = match args.single::<usize>() {
        Ok(loop_amount) => loop_amount,
        Err(_) => {
            debug!("not looping");
            let loop_amount = 0;
            loop_amount
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

        if loop_amount > 0 {
            let _ = song.loop_for(loop_amount);
            debug!("looping audio track for {}", loop_amount.to_string().as_str());
        };

        let _ = song.add_event(
            Event::Track(TrackEvent::Play),
    
            AudioTrackStart {
                ctx: ctx.clone()
            }
        );

        let _ = song.add_event(
            Event::Track(TrackEvent::End),
            AudioTrackEnd {
                manager,
                guild_id,
                ctx: ctx.clone()
            }
        );
    } else {
        debug!("no search available");
    }

    Ok(())
}

struct AudioTrackEnd {
    manager: Arc<Songbird>,
    guild_id: GuildId,
    ctx: Context
}

#[async_trait]
impl VoiceEventHandler for AudioTrackEnd {
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
            let status = OnlineStatus::Online;
            let activity = ActivityData::custom("");
            self.ctx.set_presence(Some(activity), status);

        } else {
            error!("not in voice channel");
        }

        None
    }
}

struct AudioTrackStart {
    ctx: Context
}

#[async_trait]
impl VoiceEventHandler for AudioTrackStart {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {

        let status = OnlineStatus::DoNotDisturb;
        let activity = ActivityData::playing("Darude - Sandstorm");
        self.ctx.set_presence(Some(activity), status);

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

#[command]
#[only_in(guilds)]
async fn clear(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at installation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        queue.stop();
        debug!("queue cleared");
    } else {
        error!("failed to clear queue");
    }

    Ok(())
}
