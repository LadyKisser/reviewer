use poise::serenity_prelude::{
    Context,
    ModalSubmitInteraction,
    InteractionResponseType,
};
use crate::app::{
    utils::database::Database,
    models::review::ReviewType,
};

pub async fn handle_modal(
    ctx: &Context,
    interaction: &ModalSubmitInteraction,
    db: &Database,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let custom_id = &interaction.data.custom_id;
    let (target_id, review_type) = if let Some(id) = custom_id.strip_prefix("review_modal:") {
        (id.parse::<i64>()?, ReviewType::User)
    } else if let Some(id) = custom_id.strip_prefix("server_review_modal:") {
        (id.parse::<i64>()?, ReviewType::Server)
    } else {
        return Ok(());
    };

    let rating_str = interaction
        .data
        .components
        .get(1)
        .and_then(|row| row.components.first())
        .and_then(|component| match component {
            poise::serenity_prelude::ActionRowComponent::InputText(input) => Some(input.value.clone()),
            _ => None
        })
        .ok_or("Could not find rating input")?;

    let rating = match rating_str.parse::<i32>() {
        Ok(r) if r >= 1 && r <= 5 => r,
        _ => {
            interaction
                .create_interaction_response(ctx, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.content("❌ Rating must be a number between 1 and 5")
                                .ephemeral(true)
                        })
                })
                .await?;
            return Ok(());
        }
    };

    let comment = interaction
        .data
        .components
        .get(0)
        .and_then(|row| row.components.first())
        .and_then(|component| match component {
            poise::serenity_prelude::ActionRowComponent::InputText(input) => {
                if input.value.trim().is_empty() {
                    None
                } else {
                    Some(input.value.clone())
                }
            },
            _ => None
        });

    let has_reviewed = db.has_reviewed(target_id, interaction.user.id.0 as i64, &review_type).await?;
    
    if has_reviewed {
        db.update_review(
            target_id,
            interaction.user.id.0 as i64,
            rating,
            comment,
            &review_type,
        ).await?;
    } else {
        db.add_review(
            target_id,
            interaction.user.id.0 as i64,
            rating,
            comment,
            &review_type,
        ).await?;
    }

    // TODO: Add moderation such as text moderation (highly likely the moderation AI from the OpenAI API)
    // TODO: Add support for images in the review (will probably use self hosted image moderation AI)

    let _ = db.get_average_rating(target_id, &review_type).await;

    let message = if has_reviewed {
        "✅ Review updated successfully!"
    } else {
        "✅ Review submitted successfully!"
    };

    interaction
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|d| {
                    d.content(message)
                        .ephemeral(true)
                })
        })
        .await?;

    Ok(())
}