use crate::{util::check_error_music, types::{Context, Error}};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
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

        check_error_music(
            ctx.say(format!("Song skipped: {} in queue.", queue.len()))
                .await,
        );
    } else {
        check_error_music(ctx.say("Not in a voice channel to play in").await);
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
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

        check_error_music(ctx.say("Queue cleared.").await);
    } else {
        check_error_music(ctx.say("Not in a voice channel to play in").await);
    }

    Ok(())
}