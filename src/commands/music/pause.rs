use crate::{commands::music::Context, types::Error, util::check_error_music};
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
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

        check_error_music(ctx.say("Queue cleared.").await);
    } else {
        check_error_music(ctx.say("Not in a voice channel to play in").await);
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn resume(ctx: Context<'_>) -> Result<(), Error> {
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

        check_error_music(ctx.say("Queue cleared.").await);
    } else {
        check_error_music(ctx.say("Not in a voice channel to play in").await);
    }

    Ok(())
}