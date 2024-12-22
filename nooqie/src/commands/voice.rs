use crate::{Context, Error};

use poise::{
    async_trait,
    serenity_prelude::{prelude::TypeMapKey, ActivityData, ChannelId, GuildId, OnlineStatus},
};

use log::{debug, error, info, warn};

use songbird::{
    events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent},
    input::YoutubeDl,
    tracks::TrackHandle,
    Songbird,
};

use std::sync::Arc;

use reqwest::Client as HttpClient;

pub struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

async fn get_voice_info(ctx: Context<'_>) -> Result<(GuildId, ChannelId), String> {
    let (guild_id, channel_id) = {
        let guild = match ctx.guild() {
            Some(guild) => guild,
            None => {
                return Err(String::from("user not in guild"));
            }
        };
        let channel_id = guild
            .voice_states
            .get(ctx.author().id.as_ref())
            .and_then(|voice_states| voice_states.channel_id);
        (guild.id, channel_id)
    };

    match channel_id {
        Some(thing) => Ok((guild_id, thing)),
        _ => Err(String::from("user not in voice channel, aborting")),
    }
}

async fn get_manager(ctx: Context<'_>) -> Result<Arc<Songbird>, String> {
    let manager = songbird::get(ctx.as_ref()).await;
    match manager {
        Some(manager) => Ok(manager.clone()),
        _ => Err(String::from(
            "Songbird Voice client placed in at initialisation.",
        )),
    }
}

#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    guild_only = true,
    aliases("vc", "voice"),
    category = "Voice",
    help_text_fn = join_help
)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let (guild_id, connect_to) = match get_voice_info(ctx).await {
        Ok((guild_id, connect_to)) => (guild_id, connect_to),
        Err(err) => {
            error!("{err}");
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.as_ref())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
        let mut handler = handler_lock.lock().await;
        let current_channel = match handler.current_channel() {
            Some(channel) => channel.to_string(),
            None => {
                warn!("user not in voice channel, aborting");
                return Ok(());
            }
        };
        debug!("{}: joined channel", current_channel);
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
    }

    Ok(())
}

pub fn join_help() -> String {
    String::from("joins current voice channel")
}

#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    guild_only = true,
    aliases("fuckoff", "fuck off", "get out"),
    category = "Voice",
    help_text_fn = leave_help
)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match get_voice_info(ctx).await {
        Ok(info) => info.0,
        Err(err) => {
            error!("{err}");
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.as_ref())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let mut current_channel = String::from("");

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        current_channel = match handler.current_channel() {
            Some(channel) => channel.to_string(),
            None => {
                warn!("user not in voice channel, aborting");
                return Ok(());
            }
        };
    } else {
        warn!("can't leave not in voice channel, aborting");
    }

    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(error) = manager.remove(guild_id).await {
            error!("failed to disconnect: {:?}", error);
        }
        debug!("{}: disconnected from voice channel", current_channel);
    } else {
        warn!("can't leave not in voice channel, aborting");
    }

    Ok(())
}

pub fn leave_help() -> String {
    String::from("leaves current voice channel")
}

struct TrackErrorNotifier;

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                error!(
                    "track {:?} encounted an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }
        None
    }
}

#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    guild_only = true,
    aliases("yt"),
    category = "Voice",
    help_text_fn = play_help
)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Youtube URL"] msg: Option<String>,
) -> Result<(), Error> {
    let (guild_id, connect_to) = match get_voice_info(ctx).await {
        Ok((guild_id, connect_to)) => (guild_id, connect_to),
        Err(err) => {
            error!("{err}");
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.as_ref())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let url = match msg {
        Some(msg) => msg,
        None => {
            warn!("missing YouTube URL, aborting");
            return Ok(());
        }
    };

    if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
        let mut handler = handler_lock.lock().await;
        let current_channel = match handler.current_channel() {
            Some(channel) => channel.to_string(),
            None => {
                warn!("user not in voice channel, aborting");
                return Ok(());
            }
        };

        debug!("{}: joined channel", current_channel);
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
    }

    let http_client = {
        let data = ctx.serenity_context().data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let current_channel = match handler.current_channel() {
            Some(channel) => channel.to_string(),
            None => {
                warn!("user not in voice channel, aborting");
                return Ok(());
            }
        };

        let src = YoutubeDl::new(http_client, url);
        let _song: TrackHandle = handler.enqueue_input(src.into()).await;

        ctx.serenity_context().set_presence(
            Some(ActivityData::custom("Darude -Sandstorm")),
            OnlineStatus::DoNotDisturb,
        );

        info!("playing in {}", current_channel);
    } else {
        warn!("no search available, aborting");
    }

    Ok(())
}

pub fn play_help() -> String {
    String::from("plays audio track from YouTube link")
}

#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    guild_only = true,
    aliases("next"),
    category = "Voice",
    help_text_fn = skip_help
)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match get_voice_info(ctx).await {
        Ok(info) => info.0,
        Err(err) => {
            error!("{err}");
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.as_ref())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let current_channel = match handler.current_channel() {
            Some(channel) => channel.to_string(),
            None => {
                warn!("user not in voice channel, aborting");
                return Ok(());
            }
        };

        let queue = handler.queue();
        debug!("{}: skipping audio track", current_channel);
        let _ = queue.skip();
    } else {
        warn!("failed to skip audio track, aborting");
    }

    Ok(())
}

pub fn skip_help() -> String {
    String::from("skips current audio track")
}

#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    guild_only = true,
    aliases("stop", "finish"),
    category = "Voice",
    help_text_fn = clear_help
)]
pub async fn clear(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match get_voice_info(ctx).await {
        Ok(info) => info.0,
        Err(err) => {
            error!("{err}");
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.as_ref())
        .await
        .expect("Songbird Voice client placed in at installation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let current_channel = match handler.current_channel() {
            Some(channel) => channel.to_string(),
            None => {
                warn!("user not in voice channel, aborting");
                return Ok(());
            }
        };

        let queue = handler.queue();
        queue.stop();
        debug!("{}: queue cleared", current_channel);
    } else {
        warn!("failed to clear queue");
    }

    Ok(())
}

pub fn clear_help() -> String {
    String::from("clears audio track queue")
}

#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    guild_only = true,
    aliases("hold"),
    category = "Voice",
    help_text_fn = pause_help
)]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match get_voice_info(ctx).await {
        Ok(info) => info.0,
        Err(err) => {
            error!("{err}");
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.as_ref())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let current_channel = match handler.current_channel() {
            Some(channel) => channel.to_string(),
            None => {
                warn!("user not in voice channel, aborting");
                return Ok(());
            }
        };

        let queue = handler.queue();
        debug!("{}: pausing audio track", current_channel);
        let _ = queue.pause();
    } else {
        warn!("failed to pause audio track");
    }

    Ok(())
}

pub fn pause_help() -> String {
    String::from("pauses current audio track")
}

#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    guild_only = true,
    aliases("continue"),
    category = "Voice",
    help_text_fn = resume_help
)]
pub async fn resume(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match get_voice_info(ctx).await {
        Ok(info) => info.0,
        Err(err) => {
            error!("{err}");
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.as_ref())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let current_channel = match handler.current_channel() {
            Some(channel) => channel.to_string(),
            None => {
                warn!("user not in voice channel, aborting");
                return Ok(());
            }
        };

        let queue = handler.queue();
        debug!("{}: resuming audio track", current_channel);
        let _ = queue.resume();
    } else {
        warn!("failed to resume audio track, aborting");
    }

    Ok(())
}

pub fn resume_help() -> String {
    String::from("resumes current audio track")
}

#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    guild_only = true,
    aliases("loop"),
    category = "Voice",
    help_text_fn = loop_help
)]
pub async fn loop_track(
    ctx: Context<'_>,
    #[description = "Amount"] msg: Option<String>,
) -> Result<(), Error> {
    let guild_id = match get_voice_info(ctx).await {
        Ok(info) => info.0,
        Err(err) => {
            error!("{err}");
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.as_ref())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let amount = match msg {
        Some(msg) => msg,
        None => String::from("0"),
    };

    let loops: usize = match amount.parse::<usize>() {
        Ok(loops) => loops,
        Err(error) => {
            warn!("unable to parse loop amount: {}", error);
            return Ok(());
        }
    };

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let current_channel = match handler.current_channel() {
            Some(channel) => channel.to_string(),
            None => {
                warn!("user not in voice channel, aborting");
                return Ok(());
            }
        };

        let queue = handler.queue();
        let current = match queue.current() {
            Some(current) => current,
            None => {
                warn!("no track to loop");
                return Ok(());
            }
        };
        if loops == 0 {
            debug!("{}: looping audio track", current_channel);
            let _ = current.enable_loop();
        } else {
            debug!("looping audio track for {}", loops);
            let _ = current.loop_for(loops);
        }
    } else {
        warn!("failed to loop audio track, aborting");
    }

    Ok(())
}

pub fn loop_help() -> String {
    String::from("loops current audio track")
}
