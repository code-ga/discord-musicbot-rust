use crate::{
    types::{Context, Error},
    util::check_error_music,
};
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn unmute(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.mute(false).await {
            check_error_music(ctx.say(format!("Failed: {:?}", e)).await);
        }

        check_error_music(ctx.say("Unmuted").await);
    } else {
        check_error_music(ctx.say("Not in a voice channel to unmute in").await);
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn mute(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_error_music(ctx.say("Not in a voice channel").await);

            return Ok(());
        }
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        check_error_music(ctx.say("Already muted").await);
    } else {
        if let Err(e) = handler.mute(true).await {
            check_error_music(ctx.say(format!("Failed: {:?}", e)).await);
        }

        check_error_music(ctx.say("Now muted").await);
    }

    Ok(())
}
