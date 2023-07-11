use crate::types::{Context, Error};
use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use poise::serenity_prelude::{
    async_trait, http::Http, model::prelude::ChannelId, GuildId, Mentionable,
};

use songbird::{
    input::{self, restartable::Restartable},
    Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent,
};

#[poise::command(
    prefix_command,
    slash_command,
    subcommands(
        "deafen",
        "join",
        "leave",
        "mute",
        "play_fade",
        "play",
        "skip",
        "stop",
        "undeafen",
        "unmute"
    )
)]
pub async fn music(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Hello there!").await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
async fn deafen(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_error(ctx.say("Not in a voice channel").await);

            return Ok(());
        }
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_deaf() {
        check_error(ctx.say("Already deafened").await);
    } else {
        if let Err(e) = handler.deafen(true).await {
            check_error(ctx.say(format!("Failed: {:?}", e)).await);
        }

        check_error(ctx.say("Deafened").await);
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_error(ctx.say("Not in a voice channel").await);

            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (handle_lock, success) = manager.join(guild_id, connect_to).await;

    if let Ok(_channel) = success {
        check_error(ctx.say(&format!("Joined {}", connect_to.mention())).await);

        let chan_id = ctx.channel_id();

        let send_http = ctx.serenity_context().http.clone();

        let mut handle = handle_lock.lock().await;

        handle.add_global_event(
            Event::Track(TrackEvent::End),
            TrackEndNotifier {
                chan_id,
                http: send_http,
                manager: manager.clone(),
                guild_id: guild_id.clone(),
            },
        );

        let send_http = ctx.serenity_context().http.clone();

        handle.add_global_event(
            Event::Periodic(Duration::from_secs(60), None),
            ChannelDurationNotifier {
                chan_id,
                count: Default::default(),
                http: send_http,
                voice_channel_id: connect_to.0.to_string(),
            },
        );
    } else {
        check_error(ctx.say("Error joining the channel").await);
    }

    Ok(())
}

struct TrackEndNotifier {
    chan_id: ChannelId,
    http: Arc<Http>,
    guild_id: GuildId,
    manager: Arc<songbird::Songbird>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            check_error(
                self.chan_id
                    .say(&self.http, &format!("Tracks ended: {}.", track_list.len()))
                    .await,
            );
        }

        if let Some(handle_lock) = self.manager.get(self.guild_id) {
            let handle = handle_lock.lock().await;
            handle.queue().stop();
        }
        None
    }
}

struct ChannelDurationNotifier {
    #[allow(unused)]
    chan_id: ChannelId,
    count: Arc<AtomicUsize>,
    #[allow(unused)]
    http: Arc<Http>,
    voice_channel_id: String,
}

#[async_trait]
impl VoiceEventHandler for ChannelDurationNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let count_before = self.count.fetch_add(1, Ordering::Relaxed);

        println!(
            "I've been in this channel ({}) for {} minutes!",
            self.voice_channel_id,
            count_before + 1
        );

        None
    }
}

#[poise::command(slash_command, prefix_command, guild_only)]
async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_error(ctx.say(format!("Failed: {:?}", e)).await);
        }

        check_error(ctx.say("Left voice channel").await);
    } else {
        check_error(ctx.say("Not in a voice channel").await);
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
async fn mute(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_error(ctx.say("Not in a voice channel").await);

            return Ok(());
        }
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        check_error(ctx.say("Already muted").await);
    } else {
        if let Err(e) = handler.mute(true).await {
            check_error(ctx.say(format!("Failed: {:?}", e)).await);
        }

        check_error(ctx.say("Now muted").await);
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
async fn play_fade(
    ctx: Context<'_>,
    #[description = "Music name"] music_name: String,
) -> Result<(), Error> {
    let url = music_name;

    if !url.starts_with("http") {
        check_error(ctx.say("Must provide a valid URL").await);

        return Ok(());
    }

    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let source = match input::ytdl(&url).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                check_error(ctx.say("Error sourcing ffmpeg").await);

                return Ok(());
            }
        };

        // This handler object will allow you to, as needed,
        // control the audio track via events and further commands.
        let song = handler.play_source(source);
        let send_http = ctx.serenity_context().http.clone();
        let chan_id = ctx.channel_id();

        // This shows how to periodically fire an event, in this case to
        // periodically make a track quieter until it can be no longer heard.
        let _ = song.add_event(
            Event::Periodic(Duration::from_secs(5), Some(Duration::from_secs(7))),
            SongFader {
                chan_id,
                http: send_http,
            },
        );

        let send_http = ctx.serenity_context().http.clone();

        // This shows how to fire an event once an audio track completes,
        // either due to hitting the end of the bytestream or stopped by user code.
        let _ = song.add_event(
            Event::Track(TrackEvent::End),
            SongEndNotifier {
                chan_id,
                http: send_http,
            },
        );

        check_error(ctx.say("Playing song").await);
    } else {
        check_error(ctx.say("Not in a voice channel to play in").await);
    }

    Ok(())
}

struct SongFader {
    chan_id: ChannelId,
    http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for SongFader {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(&[(state, track)]) = ctx {
            let _ = track.set_volume(state.volume / 2.0);

            if state.volume < 1e-2 {
                let _ = track.stop();
                check_error(self.chan_id.say(&self.http, "Stopping song...").await);
                Some(Event::Cancel)
            } else {
                check_error(self.chan_id.say(&self.http, "Volume reduced.").await);
                None
            }
        } else {
            None
        }
    }
}

struct SongEndNotifier {
    chan_id: ChannelId,
    http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for SongEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        check_error(
            self.chan_id
                .say(&self.http, "Song faded out completely!")
                .await,
        );

        None
    }
}

#[poise::command(slash_command, prefix_command, guild_only)]
async fn play(
    ctx: Context<'_>,
    #[description = "Url to video or playlist"] music_name: String,
) -> Result<(), Error> {
    ctx.defer().await.unwrap();
    let url = music_name;

    if !url.starts_with("http") {
        check_error(ctx.say("Must provide a valid URL").await);

        return Ok(());
    }

    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        // Here, we use lazy restartable sources to make sure that we don't pay
        // for decoding, playback on tracks which aren't actually live yet.
        let source = match Restartable::ytdl(url, true).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                check_error(ctx.say("Error sourcing ffmpeg").await);

                return Ok(());
            }
        };

        handler.enqueue_source(source.into());

        check_error(
            ctx.say(format!(
                "Added song to queue: position {}",
                handler.queue().len()
            ))
            .await,
        );
    } else {
        check_error(ctx.say("Not in a voice channel to play in").await);
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.skip();

        check_error(
            ctx.say(format!("Song skipped: {} in queue.", queue.len()))
                .await,
        );
    } else {
        check_error(ctx.say("Not in a voice channel to play in").await);
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.stop();

        check_error(ctx.say("Queue cleared.").await);
    } else {
        check_error(ctx.say("Not in a voice channel to play in").await);
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.pause();

        check_error(ctx.say("Queue cleared.").await);
    } else {
        check_error(ctx.say("Not in a voice channel to play in").await);
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
async fn resume(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.resume();

        check_error(ctx.say("Queue cleared.").await);
    } else {
        check_error(ctx.say("Not in a voice channel to play in").await);
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
async fn undeafen(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.deafen(false).await {
            check_error(ctx.say(format!("Failed: {:?}", e)).await);
        };

        check_error(ctx.say("Undeafened").await);
    } else {
        check_error(ctx.say("Not in a voice channel to undeafen in").await);
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
async fn unmute(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.mute(false).await {
            check_error(ctx.say(format!("Failed: {:?}", e)).await);
        }

        check_error(ctx.say("Unmuted").await);
    } else {
        check_error(ctx.say("Not in a voice channel to unmute in").await);
    }

    Ok(())
}

fn check_error<RT, ET: Debug>(result: Result<RT, ET>) {
    match result {
        Err(e) => {
            println!("Have some error on music command {:#?}", e);
        }
        _ => {}
    }
}
