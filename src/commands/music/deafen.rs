use crate::{
    types::{Context, Error},
    util::check_error_music,
};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn deafen(ctx: Context<'_>) -> Result<(), Error> {
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

    if handler.is_deaf() {
        check_error_music(ctx.say("Already deafened").await);
    } else {
        if let Err(e) = handler.deafen(true).await {
            check_error_music(ctx.say(format!("Failed: {:?}", e)).await);
        }

        check_error_music(ctx.say("Deafened").await);
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn undeafen(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.deafen(false).await {
            check_error_music(ctx.say(format!("Failed: {:?}", e)).await);
        };

        check_error_music(ctx.say("Undeafened").await);
    } else {
        check_error_music(ctx.say("Not in a voice channel to undeafen in").await);
    }

    Ok(())
}
