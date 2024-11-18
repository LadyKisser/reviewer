use poise::serenity_prelude::{self as serenity, User, CreateEmbed};
use crate::app::{
    models::review::{ReviewType, RatingCategory},
    utils::{database::Database, colors::get_rating_color},
};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Clone)]
pub struct Data {
    pub db: Database,
}

#[poise::command(slash_command, subcommands("user", "server"))]
pub async fn review(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Please use one of the subcommands: `/review user` or `/review server`").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn user(
    ctx: Context<'_>,
    #[description = "User to review"] user: Option<User>,
) -> Result<(), Error> {
    let target_user = user.as_ref().unwrap_or(ctx.author());
    handle_review(ctx, target_user, ReviewType::User).await
}

#[poise::command(slash_command)]
pub async fn server(
    ctx: Context<'_>,
    #[description = "Server invite link"] invite: String,
) -> Result<(), Error> {
    let invite_code = invite
        .split('/')
        .last()
        .ok_or("Invalid invite link")?;

    let invite_info = ctx
        .serenity_context()
        .http
        .get_invite(invite_code, false, false, None)
        .await
        .map_err(|_| "Invalid or expired invite")?;

    let guild = invite_info.guild
        .ok_or("Could not get server information")?;

    handle_review(ctx, &guild, ReviewType::Server).await
}

async fn handle_review<T>(
    ctx: Context<'_>,
    target: &T,
    review_type: ReviewType,
) -> Result<(), Error>
where
    T: ReviewTarget,
{
    let target_id = target.get_id();
    let average_rating = ctx.data().db.get_average_rating(target_id, &review_type).await?.unwrap_or(0.0);
    let reviews_count = ctx.data().db.get_reviews_count(target_id, &review_type).await?;
    let has_reviewed = ctx.data().db.has_reviewed(
        target_id,
        ctx.author().id.0 as i64,
        &review_type
    ).await?;

    let category = RatingCategory::from_average(average_rating);

    let mut embed = CreateEmbed::default();
    target.build_embed(&mut embed, average_rating, &category, reviews_count);

    if !target.is_self(ctx.author().id.0 as i64) {
        let button_label = if has_reviewed {
            "Update Review"
        } else {
            "Add Review"
        };

        let (review_button_id, reviews_list_id) = match review_type {
            ReviewType::User => (
                format!("review_button:{}", target_id),
                format!("reviews_list:{}", target_id)
            ),
            ReviewType::Server => (
                format!("server_review_button:{}", target_id),
                format!("server_reviews_list:{}", target_id)
            ),
        };

        ctx.send(|m| {
            m.embed(|e| {
                *e = embed;
                e
            })
            .components(|c| {
                c.create_action_row(|r| {
                    r.create_button(|b| {
                        b.custom_id(review_button_id)
                            .label(button_label)
                            .style(serenity::ButtonStyle::Primary)
                    })
                    .create_button(|b| {
                        b.custom_id(reviews_list_id)
                            .label("Reviews")
                            .style(serenity::ButtonStyle::Secondary)
                    })
                })
            })
        })
        .await?;
    } else {
        ctx.send(|m| {
            m.embed(|e| {
                *e = embed;
                e
            })
        })
        .await?;
    }

    Ok(())
}

trait ReviewTarget {
    fn get_id(&self) -> i64;
    fn is_self(&self, user_id: i64) -> bool;
    fn build_embed(&self, embed: &mut CreateEmbed, average_rating: f64, category: &RatingCategory, reviews_count: i64);
}

impl ReviewTarget for User {
    fn get_id(&self) -> i64 {
        self.id.0 as i64
    }

    fn is_self(&self, user_id: i64) -> bool {
        self.id.0 as i64 == user_id
    }

    fn build_embed(&self, embed: &mut CreateEmbed, average_rating: f64, category: &RatingCategory, reviews_count: i64) {
        embed
            .title(format!("User Review: {}", self.name))
            .thumbnail(self.face())
            .field("Rating", format!("{:.1} / 5.0", average_rating), true)
            .field("Category", category.to_string(), true)
            .field("Total Reviews", reviews_count.to_string(), true)
            .color(get_rating_color(category));
    }
}

impl ReviewTarget for serenity::Guild {
    fn get_id(&self) -> i64 {
        self.id.0 as i64
    }

    fn is_self(&self, _user_id: i64) -> bool {
        false
    }

    fn build_embed(&self, embed: &mut CreateEmbed, average_rating: f64, category: &RatingCategory, reviews_count: i64) {
        embed
            .title(format!("Server Review: {}", self.name))
            .thumbnail(self.icon_url().unwrap_or_default())
            .field("Rating", format!("{:.1} / 5.0", average_rating), true)
            .field("Category", category.to_string(), true)
            .field("Total Reviews", reviews_count.to_string(), true)
            .color(get_rating_color(category));
    }
}

impl ReviewTarget for serenity::InviteGuild {
    fn get_id(&self) -> i64 {
        self.id.0 as i64
    }

    fn is_self(&self, _user_id: i64) -> bool {
        false
    }

    fn build_embed(&self, embed: &mut CreateEmbed, average_rating: f64, category: &RatingCategory, reviews_count: i64) {
        embed
            .title(format!("Server Review: {}", self.name))
            .field("Rating", format!("{:.1} / 5.0", average_rating), true)
            .field("Category", category.to_string(), true)
            .field("Total Reviews", reviews_count.to_string(), true)
            .color(get_rating_color(category));

        if let Some(icon) = &self.icon {
            let icon_url = format!(
                "https://cdn.discordapp.com/icons/{}/{}.{}",
                self.id,
                icon,
                if icon.starts_with("a_") { "gif" } else { "png" }
            );
            embed.thumbnail(icon_url);
        }
    }
}
 