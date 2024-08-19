use crate::{Context, Error};
use poise::serenity_prelude::prelude::TypeMapKey;
use poise::serenity_prelude::standard::CommandResult;
use poise::serenity_prelude::Message;

use log::{debug, error, warn};

// use serenity::{
//     all::{ActivityData, GuildId, OnlineStatus},
//     async_trait,
//     client::Context,
//     framework::standard::{macros::command, Args, CommandResult},
//     model::channel::Message,
//     prelude::TypeMapKey,
// };

use songbird::{
    events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent},
    input::YoutubeDl,
    Songbird,
};

use std::sync::Arc;

use reqwest::Client as HttpClient;

pub struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn join(ctx: Context<'_>) -> CommandResult {
    let (guild_id, channel_id) = {
        let guild = ctx.guild().unwrap();
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
        let current_channel = handler.current_channel().unwrap().to_string();
        debug!("{}: joined channel", current_channel);
        // handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
    }

    Ok(())
}

// #[description = "leaves current voice channel"]
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn leave(ctx: Context<'_>) -> CommandResult {
    let guild_id = {
        let guild = ctx.guild().unwrap();
        guild.id
    };

    let manager = songbird::get(ctx.as_ref())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let mut current_channel = String::from("");

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        current_channel = handler.current_channel().unwrap().to_string();
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

// struct TrackErrorNotifier;
//
// #[async_trait]
// impl VoiceEventHandler for TrackErrorNotifier {
//     async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
//         if let EventContext::Track(track_list) = ctx {
//             for (state, handle) in *track_list {
//                 error!(
//                     "track {:?} encounted an error: {:?}",
//                     handle.uuid(),
//                     state.playing
//                 );
//             }
//         }
//         None
//     }
// }
//
// #[command]
// #[only_in(guilds)]
// #[description = "plays audio track from YouTube link"]
// async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
//     let (guild_id, channel_id) = {
//         let guild = msg.guild(&ctx.cache).unwrap();
//         let channel_id = guild
//             .voice_states
//             .get(&msg.author.id)
//             .and_then(|voice_states| voice_states.channel_id);
//         (guild.id, channel_id)
//     };
//
//     let connect_to = match channel_id {
//         Some(channel) => channel,
//         None => {
//             warn!("user not in voice channel, aborting");
//             return Ok(());
//         }
//     };
//
//     let manager = songbird::get(&ctx)
//         .await
//         .expect("Songbird Voice client placed in at initialisation.")
//         .clone();
//
//     let url = match args.single::<String>() {
//         Ok(url) => url,
//         Err(_error) => {
//             warn!("missing YouTube URL, aborting");
//             return Ok(());
//         }
//     };
//
//     let loop_amount = match args.single::<usize>() {
//         Ok(loop_amount) => loop_amount,
//         Err(_error) => {
//             let loop_amount = 0;
//             loop_amount
//         }
//     };
//
//     if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
//         let mut handler = handler_lock.lock().await;
//         let current_channel = handler.current_channel().unwrap().to_string();
//         debug!("{}: joined channel", current_channel);
//         handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
//     }
//
//     let guild_id = msg.guild_id.unwrap();
//
//     let http_client = {
//         let data = ctx.data.read().await;
//         data.get::<HttpKey>()
//             .cloned()
//             .expect("Guaranteed to exist in the typemap.")
//     };
//
//     if let Some(handler_lock) = manager.get(guild_id) {
//         let mut handler = handler_lock.lock().await;
//         let current_channel = handler.current_channel().unwrap().to_string();
//         let src = YoutubeDl::new(http_client, url);
//         let song = handler.enqueue_input(src.into()).await;
//
//         if loop_amount > 0 {
//             let _ = song.loop_for(loop_amount);
//         };
//
//         debug!(
//             "{}: added track to queue, looping {} times",
//             current_channel,
//             loop_amount.to_string().as_str()
//         );
//
//         let _ = song.add_event(
//             Event::Track(TrackEvent::Play),
//             AudioTrackStart { ctx: ctx.clone() },
//         );
//
//         let _ = song.add_event(
//             Event::Track(TrackEvent::End),
//             AudioTrackEnd {
//                 manager,
//                 guild_id,
//                 ctx: ctx.clone(),
//             },
//         );
//     } else {
//         warn!("no search available, aborting");
//     }
//
//     Ok(())
// }
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
//
// #[command]
// #[only_in(guilds)]
// #[description = "skips current audio track"]
// async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
//     let guild_id = msg.guild_id.unwrap();
//     let manager = songbird::get(ctx)
//         .await
//         .expect("Songbird Voice client placed in at initialisation.")
//         .clone();
//
//     if let Some(handler_lock) = manager.get(guild_id) {
//         let handler = handler_lock.lock().await;
//         let current_channel = handler.current_channel().unwrap().to_string();
//         let queue = handler.queue();
//         debug!("{}: skipping audio track", current_channel);
//         let _ = queue.skip();
//     } else {
//         warn!("failed to skip audio track, aborting");
//     }
//
//     Ok(())
// }
//
// #[command]
// #[only_in(guilds)]
// #[description = "clears audio track queue"]
// async fn clear(ctx: &Context, msg: &Message) -> CommandResult {
//     let guild_id = msg.guild_id.unwrap();
//
//     let manager = songbird::get(ctx)
//         .await
//         .expect("Songbird Voice client placed in at installation.")
//         .clone();
//
//     if let Some(handler_lock) = manager.get(guild_id) {
//         let handler = handler_lock.lock().await;
//         let current_channel = handler.current_channel().unwrap().to_string();
//         let queue = handler.queue();
//         queue.stop();
//         debug!("{}: queue cleared", current_channel);
//     } else {
//         warn!("failed to clear queue");
//     }
//
//     Ok(())
// }
//
// #[command]
// #[only_in(guilds)]
// #[description = "pauses current audio track"]
// async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
//     let guild_id = msg.guild_id.unwrap();
//     let manager = songbird::get(ctx)
//         .await
//         .expect("Songbird Voice client placed in at initialisation.")
//         .clone();
//
//     if let Some(handler_lock) = manager.get(guild_id) {
//         let handler = handler_lock.lock().await;
//         let queue = handler.queue();
//         let current_channel = handler.current_channel().unwrap().to_string();
//         debug!("{}: pausing audio track", current_channel);
//         let _ = queue.pause();
//     } else {
//         warn!("failed to pause audio track");
//     }
//
//     Ok(())
// }
//
// #[command]
// #[only_in(guilds)]
// #[description = "resumes current audio track"]
// async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
//     let guild_id = msg.guild_id.unwrap();
//     let manager = songbird::get(ctx)
//         .await
//         .expect("Songbird Voice client placed in at initialisation.")
//         .clone();
//
//     if let Some(handler_lock) = manager.get(guild_id) {
//         let handler = handler_lock.lock().await;
//         let queue = handler.queue();
//         let current_channel = handler.current_channel().unwrap().to_string();
//         debug!("{}: resuming audio track", current_channel);
//         let _ = queue.resume();
//     } else {
//         warn!("failed to resume audio track, aborting");
//     }
//
//     Ok(())
// }
