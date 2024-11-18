use poise::serenity_prelude::{Context, Ready};
use tracing::info;

pub async fn ready(ctx: &Context, ready: &Ready) {
    info!("Bot connected as {}", ready.user.name);
    info!("Connected to {} guilds", ready.guilds.len());

    ctx.set_presence(
        Some(poise::serenity_prelude::Activity::playing("with reviews")),
        poise::serenity_prelude::OnlineStatus::Online,
    ).await;
} 