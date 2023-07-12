mod deafen;
mod join;
mod mute;
mod pause;
mod play;
mod stop_skip;
use crate::types::{Context, Error};
use deafen::{deafen, undeafen};
use join::{join, leave};
use mute::{mute, unmute};
use pause::{pause, resume};
use play::{play, play_fade};
use stop_skip::{skip, stop};

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
        "unmute",
        "pause",
        "resume"
    )
)]
pub async fn music(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Hello there!").await?;
    Ok(())
}
