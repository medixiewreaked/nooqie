use crate::{Context, Error};

use poise::{
    async_trait,
    serenity_prelude::{prelude::TypeMapKey, ActivityData, OnlineStatus},
};

use log::{debug, error, info, warn};

use songbird::{
    events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent},
    input::YoutubeDl,
    tracks::TrackHandle,
};

use reqwest::Client as HttpClient;

pub struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    aliases("vc", "voice"),
    category = "Voice"
)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let (guild_id, channel_id) = {
        let guild = match ctx.guild() {
            Some(guild) => guild,
            None => {
                warn!("user not in guild");
                return Ok(());
            }
        };
        let channel_id = guild
            .voice_states
            .get(ctx.author().id.as_ref())
            .and_then(|voice_states| voice_states.channel_id);
        (guild.id, channel_id)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            warn!("user not in voice channel, aborting");
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

// #[description = "leaves current voice channel"]
#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    aliases("fuckoff", "fuck off", "get out"),
    category = "Voice"
)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = {
        let guild = match ctx.guild() {
            Some(guild) => guild,
            None => {
                warn!("bot not in guild");
                return Ok(());
            }
        };
        guild.id
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

// #[description = "plays audio track from YouTube link"]
// async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    aliases("yt"),
    category = "Voice"
)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Youtube URL"] msg: Option<String>,
) -> Result<(), Error> {
    let (guild_id, channel_id) = {
        let guild = match ctx.guild() {
            Some(guild) => guild,
            None => {
                warn!("bot not in guild");
                return Ok(());
            }
        };

        let channel_id = guild
            .voice_states
            .get(ctx.author().id.as_ref())
            .and_then(|voice_states| voice_states.channel_id);
        (guild.id, channel_id)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            warn!("user not in voice channel, aborting");
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
        let current_channel = handler.current_channel().unwrap().to_string();
        let src = YoutubeDl::new(http_client, url);
        let _song: TrackHandle = handler.enqueue_input(src.into()).await;

        ctx.serenity_context().set_presence(
            Some(ActivityData::custom("Darude -Sandstorm")),
            OnlineStatus::DoNotDisturb,
        );

        info!("playing in {}", current_channel);

    //         let _ = song.add_event(
    //             Event::Track(TrackEvent::End),
    //             AudioTrackEnd {
    //                 manager,
    //                 guild_id,
    //                 ctx: ctx.clone(),
    //             },
    //         );
    } else {
        warn!("no search available, aborting");
    }

    Ok(())
}
//
// struct AudioTrackEnd {
//     manager: Arc<Songbird>,
//     guild_id: GuildId,
//     ctx: Context,
// }
//
// #[async_trait]
// impl VoiceEventHandler for AudioTrackEnd {
//     async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
//         let mut current_channel = String::from("");
//
//         if let Some(handler_lock) = self.manager.get(self.guild_id) {
//             let handler = handler_lock.lock().await;
//             current_channel = handler.current_channel().unwrap().to_string();
//             if handler.queue().len() > 0 {
//                 return None;
//             }
//         }
//
//         let has_handler = self.manager.get(self.guild_id).is_some();
//
//         if has_handler {
//             if let Err(error) = self.manager.remove(self.guild_id).await {
//                 error!("{}: failed to disconnect: {:?}", current_channel, error);
//             }
//             debug!("{}: disconnected from voice channel", current_channel);
//             let status = OnlineStatus::Online;
//             let activity = ActivityData::custom("");
//             self.ctx.set_presence(Some(activity), status);
//         } else {
//             warn!("not in voice channel, aborting");
//         }
//
//         None
//     }
// }
//
// struct AudioTrackStart {
//     ctx: Context,
// }
//
// #[async_trait]
// impl VoiceEventHandler for AudioTrackStart {
//     async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
//         let status = OnlineStatus::DoNotDisturb;
//         let activity = ActivityData::playing("Darude - Sandstorm");
//         self.ctx.set_presence(Some(activity), status);
//
//         None
//     }
// }

// #[description = "skips current audio track"]
#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    aliases("next"),
    category = "Voice"
)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = {
        let guild = match ctx.guild() {
            Some(guild) => guild,
            None => {
                warn!("bot not in guild");
                return Ok(());
            }
        };

        guild.id
    };

    let manager = songbird::get(ctx.as_ref())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let current_channel = handler.current_channel().unwrap().to_string();
        let queue = handler.queue();
        debug!("{}: skipping audio track", current_channel);
        let _ = queue.skip();
    } else {
        warn!("failed to skip audio track, aborting");
    }

    Ok(())
}

// #[description = "clears audio track queue"]
#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    aliases("stop"),
    category = "Voice"
)]
pub async fn clear(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = {
        let guild = match ctx.guild() {
            Some(guild) => guild,
            None => {
                warn!("bot not in guild");
                return Ok(());
            }
        };

        guild.id
    };

    let manager = songbird::get(ctx.as_ref())
        .await
        .expect("Songbird Voice client placed in at installation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let current_channel = handler.current_channel().unwrap().to_string();
        let queue = handler.queue();
        queue.stop();
        debug!("{}: queue cleared", current_channel);
    } else {
        warn!("failed to clear queue");
    }

    Ok(())
}

// #[description = "pauses current audio track"]
#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    aliases("hold"),
    category = "Voice"
)]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = {
        let guild = match ctx.guild() {
            Some(guild) => guild,
            None => {
                warn!("bot not in guild");
                return Ok(());
            }
        };

        guild.id
    };

    let manager = songbird::get(ctx.as_ref())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let current_channel = handler.current_channel().unwrap().to_string();
        let queue = handler.queue();
        debug!("{}: pausing audio track", current_channel);
        let _ = queue.pause();
    } else {
        warn!("failed to pause audio track");
    }

    Ok(())
}

// #[description = "resumes current audio track"]
#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    aliases("continue"),
    category = "Voice"
)]
pub async fn resume(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = {
        let guild = match ctx.guild() {
            Some(guild) => guild,
            None => {
                warn!("bot not in guild");
                return Ok(());
            }
        };

        guild.id
    };

    let manager = songbird::get(ctx.as_ref())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let current_channel = handler.current_channel().unwrap().to_string();
        let queue = handler.queue();
        debug!("{}: resuming audio track", current_channel);
        let _ = queue.resume();
    } else {
        warn!("failed to resume audio track, aborting");
    }

    Ok(())
}

#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    aliases("loop"),
    category = "Voice"
)]
pub async fn loop_track(
    ctx: Context<'_>,
    #[description = "Amount"] msg: Option<String>,
) -> Result<(), Error> {
    let guild_id = {
        let guild = match ctx.guild() {
            Some(guild) => guild,
            None => {
                warn!("bot not in guild");
                return Ok(());
            }
        };

        guild.id
    };

    let manager = songbird::get(ctx.as_ref())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let amount = match msg {
        Some(msg) => msg,
        None => String::from("0"),
    };

    let loops: usize = amount.parse::<usize>().unwrap();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let current_channel: String = handler.current_channel().unwrap().to_string();
        let queue = handler.queue();
        let current = queue.current().unwrap();
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
