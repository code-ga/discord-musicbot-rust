extern crate dotenv;

use dotenv::dotenv;
mod types;
use types::Data;
mod commands;
mod util;
use poise::serenity_prelude as serenity;
use songbird::register_from_config;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let commands = vec![commands::music()];
    let framework = poise::Framework::builder()
        .token(std::env::var("DISCORD_TOKEN").unwrap())
        .options(poise::FrameworkOptions {
            pre_command: |ctx| {
                Box::pin(async move {
                    println!(
                        "In pre_command: {:?}",
                        ctx.invocation_data::<&str>().await.as_deref()
                    );
                })
            },
            post_command: |ctx| {
                Box::pin(async move {
                    println!(
                        "In post_command: {:?}",
                        ctx.invocation_data::<&str>().await.as_deref()
                    );
                })
            },
            on_error: |err| {
                Box::pin(async move {
                    match err {
                        poise::FrameworkError::Command { ctx, .. } => {
                            println!(
                                "In on_error: {:?}",
                                ctx.invocation_data::<&str>().await.as_deref()
                            );
                        }
                        err => poise::builtins::on_error(err).await.unwrap(),
                    }
                })
            },

            commands,
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                ctx.set_activity(serenity::model::gateway::Activity::playing(
                    "siuu sìu siuu siuu siuu siuu",
                ))
                .await;
                println!("Login as {}!", ready.user.name);
                Ok(Data {})
            })
        })
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .client_settings(|c| register_from_config(c, Default::default()));
    framework.run_autosharded().await.unwrap();
}
