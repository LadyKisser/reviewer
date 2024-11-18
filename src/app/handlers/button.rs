use poise::serenity_prelude::{
    Context,
    MessageComponentInteraction,
    CreateEmbed,
    User,
    InteractionResponseType,
    InputTextStyle,
    CreateComponents,
    ButtonStyle,
};
use crate::app::{
    utils::{database::Database, colors::get_rating_color},
    models::review::{ReviewType, RatingCategory},
};
use time::format_description::well_known::Rfc3339;

pub async fn handle_button(
    ctx: &Context,
    interaction: &MessageComponentInteraction,
    db: &Database,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let custom_id = &interaction.data.custom_id;

    match custom_id {
        id if id.starts_with("review_button:") => {
            create_review_modal(ctx, interaction, id, ReviewType::User).await?
        }
        id if id.starts_with("server_review_button:") => {
            create_review_modal(ctx, interaction, id, ReviewType::Server).await?
        }
        id if id.starts_with("reviews_list:") => {
            let target_id = id.strip_prefix("reviews_list:").unwrap().parse::<i64>()?;
            show_reviews_page(ctx, interaction, db, target_id, 0, ReviewType::User).await?
        }
        id if id.starts_with("server_reviews_list:") => {
            let target_id = id.strip_prefix("server_reviews_list:").unwrap().parse::<i64>()?;
            show_reviews_page(ctx, interaction, db, target_id, 0, ReviewType::Server).await?
        }
        id if id.starts_with("reviews_page:") => {
            handle_pagination(ctx, interaction, db, id).await?
        }
        _ => {}
    }

    Ok(())
}

async fn create_review_modal(
    ctx: &Context,
    interaction: &MessageComponentInteraction,
    custom_id: &str,
    review_type: ReviewType,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let id = custom_id.split(':').nth(1).unwrap();
    let prefix = match review_type {
        ReviewType::User => "review_modal:",
        ReviewType::Server => "server_review_modal:",
    };
    let title = match review_type {
        ReviewType::User => "User Review",
        ReviewType::Server => "Server Review",
    };
    
    interaction
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::Modal)
                .interaction_response_data(|d| {
                    d.custom_id(format!("{}{}", prefix, id))
                        .title(format!("Write a {}", title))
                        .components(|c| {
                            c.create_action_row(|row| {
                                row.create_input_text(|input| {
                                    input
                                        .custom_id("comment")
                                        .label("Comment (optional)")
                                        .style(InputTextStyle::Paragraph)
                                        .required(false)
                                        .placeholder("Write your review here...")
                                })
                            })
                            .create_action_row(|row| {
                                row.create_input_text(|input| {
                                    input
                                        .custom_id("rating")
                                        .label("Rating (1-5)")
                                        .style(InputTextStyle::Short)
                                        .required(true)
                                        .min_length(1)
                                        .max_length(1)
                                        .placeholder("Enter a number between 1 and 5")
                                })
                            })
                        })
                })
        })
        .await?;
    Ok(())
}

async fn handle_pagination(
    ctx: &Context,
    interaction: &MessageComponentInteraction,
    db: &Database,
    custom_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let parts: Vec<&str> = custom_id.strip_prefix("reviews_page:").unwrap().split(':').collect();
    if parts.len() == 3 {
        let target_id = parts[0].parse::<i64>()?;
        let page = parts[1].parse::<i64>()?;
        let review_type = match parts[2] {
            "user" => ReviewType::User,
            "server" => ReviewType::Server,
            _ => return Ok(()),
        };
        show_reviews_page(ctx, interaction, db, target_id, page, review_type).await?;
    }
    Ok(())
}

async fn show_reviews_page(
    ctx: &Context,
    interaction: &MessageComponentInteraction,
    db: &Database,
    target_id: i64,
    page: i64,
    review_type: ReviewType,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let total_reviews = db.get_reviews_count(target_id, &review_type).await?;

    if total_reviews == 0 {
        let message = match review_type {
            ReviewType::User => "No reviews found for this user.",
            ReviewType::Server => "No reviews found for this server.",
        };
        interaction
            .create_interaction_response(ctx, |r| {
                r.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|d| d.content(message).ephemeral(true))
            })
            .await?;
        return Ok(());
    }

    let reviews = db.get_paginated_reviews(target_id, page, 1, &review_type).await?;
    
    if let Some(review) = reviews.first() {
        let reviewer = ctx.http.get_user(review.reviewer_id as u64).await?;
        let timestamp = review.created_at
            .map(|t| t.format(&Rfc3339).unwrap_or_else(|_| String::from("Unknown date")))
            .unwrap_or_else(|| String::from("Unknown date"));
        
        let mut embed = CreateEmbed::default();
        build_embed(&mut embed, &reviewer, review.rating, &review.comment, timestamp, page, total_reviews, &review_type, interaction, ctx).await?;

        let type_str = match review_type {
            ReviewType::User => "user",
            ReviewType::Server => "server",
        };

        // TODO: Add a button to delete said review (Admin only)
        // TODO: Add a button for all users to report the review
        // TODO: Think about adding a dropdown for adding reactions to the review. Or maybe add an event listener for reactions and register them to the review.

        let mut components = CreateComponents::default();
        components.create_action_row(|row| {
            row.create_button(|b| {
                b.custom_id(format!("reviews_page:{}:{}:{}", target_id, page - 1, type_str))
                    .label("Previous")
                    .style(ButtonStyle::Secondary)
                    .disabled(page <= 0)
            })
            .create_button(|b| {
                b.custom_id(format!("reviews_page:{}:{}:{}", target_id, page + 1, type_str))
                    .label("Next")
                    .style(ButtonStyle::Secondary)
                    .disabled(page >= total_reviews - 1)
            })
        });

        interaction
            .create_interaction_response(ctx, |r| {
                r.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|d| {
                        d.embed(|e| {
                            *e = embed;
                            e
                        })
                        .components(|c| {
                            *c = components;
                            c
                        })
                        .ephemeral(true)
                    })
            })
            .await?;
    }

    Ok(())
}

async fn build_embed(
    embed: &mut CreateEmbed,
    reviewer: &User,
    rating: i32,
    comment: &Option<String>,
    timestamp: String,
    page: i64,
    total_reviews: i64,
    review_type: &ReviewType,
    interaction: &MessageComponentInteraction,
    ctx: &Context,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let star_rating = "⭐".repeat(rating as usize);
    let rating_category = RatingCategory::from_average(rating as f64);
    
    embed
        .color(get_rating_color(&rating_category))
        .author(|a| a.name(&reviewer.name).icon_url(reviewer.face()))
        .title(format!("{} {} of {}", 
            match review_type {
                ReviewType::User => "Review",
                ReviewType::Server => "Server Review",
            },
            page + 1, 
            total_reviews
        ))
        .description(format!("{}", 
            comment.as_deref().unwrap_or("*No comment provided*")
        ))
        .field("Rating", format!("{} ({})", star_rating, rating), true)
        .field("Reviewer", format!("<@{}>", reviewer.id), true);

    if let ReviewType::Server = review_type {
        if let Some(guild) = interaction.guild_id {
            embed.field("Server", guild.name(ctx).unwrap_or_else(|| "Unknown Server".to_string()), true);
        }
    }

    // TODO: humanize the timestamp

    embed.footer(|f| f.text(format!("Posted on {}", timestamp)));
    Ok(())
}