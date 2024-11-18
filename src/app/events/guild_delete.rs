use poise::serenity_prelude::{Context, GuildId, CreateEmbed, ChannelType};
use tracing::error;
use crate::app::utils::database::Database;

pub async fn guild_delete(ctx: &Context, guild_id: GuildId, _db: &Database) {
    if let Ok(log_channel) = std::env::var("LOG_CHANNEL") {
        if log_channel.is_empty() {
            return;
        }

        let channel_id = match log_channel.parse::<u64>() {
            Ok(id) => id,
            Err(_) => {
                error!("Invalid LOG_CHANNEL ID format");
                return;
            }
        };

        let mut embed = CreateEmbed::default();
        embed.title("Bot Removed from Guild")
            .field("Guild ID", guild_id.to_string(), true)
            .color(0xFF0000);

        let result = ctx.http.get_channel(channel_id).await;
        if let Ok(channel) = result {
            if let Some(guild_channel) = channel.clone().guild() {
                if matches!(guild_channel.kind, ChannelType::Text) {
                    if let Err(e) = channel.id().send_message(&ctx.http, |m| {
                        m.set_embed(embed.clone())
                    }).await {
                        error!("Failed to send message: {}", e);
                    }
                }
            }
        }
    }
} 