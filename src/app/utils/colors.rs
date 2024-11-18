use poise::serenity_prelude::Color;
use crate::app::models::review::RatingCategory;

pub fn get_rating_color(category: &RatingCategory) -> Color {
    match category {
        RatingCategory::Unrated => Color::DARK_GREY,
        RatingCategory::Poor => Color::RED,
        RatingCategory::Fair => Color::ORANGE,
        RatingCategory::Good => Color::GOLD,
        RatingCategory::VeryGood => Color::DARK_GREEN,
        RatingCategory::Excellent => Color::DARK_BLUE,
    }
} 