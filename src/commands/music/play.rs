use std::{sync::Arc, time::Duration};

use poise::{serenity_prelude::{ChannelId, Http}, async_trait};
use songbird::{
    input::{self, restartable::Restartable},
    Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent,
};

use crate::{types::{Context, Error}, util::check_error_music};
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn play_fade(
    ctx: Context<'_>,
    #[description = "Music name"] music_name: String,
) -> Result<(), Error> {
    ctx.defer().await.unwrap();
    let url = music_name;

    if !url.starts_with("http") {
        check_error_music(ctx.say("Must provide a valid URL").await);

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

                check_error_music(ctx.say("Error sourcing ffmpeg").await);

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

        check_error_music(ctx.say("Playing song").await);
    } else {
        check_error_music(ctx.say("Not in a voice channel to play in").await);
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
                check_error_music(self.chan_id.say(&self.http, "Stopping song...").await);
                Some(Event::Cancel)
            } else {
                check_error_music(self.chan_id.say(&self.http, "Volume reduced.").await);
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
        check_error_music(
            self.chan_id
                .say(&self.http, "Song faded out completely!")
                .await,
        );

        None
    }
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Url to video or playlist"] music_name: String,
) -> Result<(), Error> {
    ctx.defer().await.unwrap();
    let url = music_name;

    if !url.starts_with("http") {
        check_error_music(ctx.say("Must provide a valid URL").await);

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

                check_error_music(ctx.say("Error sourcing ffmpeg").await);

                return Ok(());
            }
        };

        handler.enqueue_source(source.into());

        check_error_music(
            ctx.say(format!(
                "Added song to queue: position {}",
                handler.queue().len()
            ))
            .await,
        );
    } else {
        check_error_music(ctx.say("Not in a voice channel to play in").await);
    }

    Ok(())
}
