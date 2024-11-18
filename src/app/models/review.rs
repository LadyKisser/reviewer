use serde::{Serialize, Deserialize};
use time::OffsetDateTime;
use crate::app::utils::datetime::datetime_format;

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "review_type", rename_all = "snake_case")]
pub enum ReviewType {
    User,
    Server,
}

impl ReviewType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReviewType::User => "user",
            ReviewType::Server => "server",
        }
    }
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Review {
    pub id: i32,
    pub target_id: i64,
    pub reviewer_id: i64,
    pub rating: i32,
    pub comment: Option<String>,
    pub review_type: ReviewType,
    #[serde(with = "datetime_format")]
    pub created_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RatingCategory {
    Unrated,
    Poor,
    Fair,
    Good,
    VeryGood,
    Excellent,
}

impl RatingCategory {
    pub fn from_average(avg: f64) -> Self {
        match avg {
            x if x == 0.0 => Self::Unrated,
            x if x <= 1.0 => Self::Poor,
            x if x <= 2.0 => Self::Fair,
            x if x <= 3.0 => Self::Good,
            x if x <= 4.0 => Self::VeryGood,
            _ => Self::Excellent,
        }
    }
}

impl std::fmt::Display for RatingCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unrated => write!(f, "Unrated"),
            Self::Poor => write!(f, "Poor"),
            Self::Fair => write!(f, "Fair"),
            Self::Good => write!(f, "Good"),
            Self::VeryGood => write!(f, "Very Good"),
            Self::Excellent => write!(f, "Excellent"),
        }
    }
} 