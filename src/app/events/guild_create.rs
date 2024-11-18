use poise::serenity_prelude::{Context, Guild, CreateEmbed, ChannelType};
use tracing::error;
use crate::app::utils::database::Database;

pub async fn guild_create(ctx: &Context, guild: &Guild, _db: &Database, is_new: bool) {
    if !is_new {
        return;
    }

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
        embed.title("Bot Added to New Guild")
            .field("Guild Name", &guild.name, true)
            .field("Guild ID", guild.id.to_string(), true)
            .field("Owner ID", guild.owner_id.to_string(), true)
            .field("Member Count", guild.member_count.to_string(), true)
            .color(0x00FF00);

        if let Some(icon_url) = guild.icon_url() {
            embed.thumbnail(icon_url);
        }

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