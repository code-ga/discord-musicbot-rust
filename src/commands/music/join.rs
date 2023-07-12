use std::{time::Duration, sync::{Arc, atomic::{AtomicUsize, Ordering}}};

use crate::{
    types::{Context, Error},
    util::check_error_music,
};
use poise::{serenity_prelude::{ChannelId, Http, GuildId, Mentionable}, async_trait};
use songbird::{
    Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent,
};
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_error_music(ctx.say("Not in a voice channel").await);

            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (handle_lock, success) = manager.join(guild_id, connect_to).await;

    if let Ok(_channel) = success {
        check_error_music(ctx.say(&format!("Joined {}", connect_to.mention())).await);

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
        check_error_music(ctx.say("Error joining the channel").await);
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
            check_error_music(
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
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_error_music(ctx.say(format!("Failed: {:?}", e)).await);
        }

        check_error_music(ctx.say("Left voice channel").await);
    } else {
        check_error_music(ctx.say("Not in a voice channel").await);
    }

    Ok(())
}
