use axum::{
    routing::{get, post},
    Router,
    extract::{Path, State},
    Json,
    http::StatusCode,
};
use crate::app::{
    utils::{database::Database, cache::Cache},
    models::review::{Review, ReviewType},
};
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub cache: Cache,
}

impl AppState {
    pub fn new(db: Database, cache: Cache) -> Self {
        Self { db, cache }
    }

    pub async fn get_cached_rating(&self, target_id: i64, review_type: &ReviewType) -> Result<Option<f64>, sqlx::Error> {
        if let Ok(Some(rating)) = self.cache.get_rating(target_id, review_type.as_str()).await {
            return Ok(Some(rating));
        }
        
        let rating = self.db.get_average_rating(target_id, review_type).await?;
        if let Some(r) = rating {
            let _ = self.cache.cache_rating(target_id, r, review_type.as_str()).await;
        }
        Ok(rating)
    }
}

#[derive(Deserialize)]
pub struct ReviewRequest {
    pub rating: i32,
    pub comment: Option<String>,
    pub reviewer_id: i64,
}

#[derive(Serialize)]
pub struct ReviewsResponse {
    pub reviews: Vec<Review>,
    pub total_reviews: i64,
    pub average_rating: f64,
    pub review_type: String,
    pub target_id: i64,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/reviews/:review_type/:target_id", get(get_reviews))
        .route("/reviews/:review_type/:target_id", post(add_review)) // TODO: Add some sort of auth
}

async fn get_reviews(
    State(state): State<AppState>,
    Path((review_type_str, target_id)): Path<(String, i64)>,
) -> Result<Json<ReviewsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let review_type = match review_type_str.as_str() {
        "user" => ReviewType::User,
        "server" => ReviewType::Server,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid review type".to_string(),
                }),
            ));
        }
    };

    let reviews = match state.db.get_reviews(target_id, &review_type).await {
        Ok(r) => r,
        Err(e) => {
            error!("Database error: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to fetch reviews".to_string(),
                }),
            ));
        }
    };

    let total_reviews = match state.db.get_reviews_count(target_id, &review_type).await {
        Ok(count) => count,
        Err(e) => {
            error!("Database error: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to count reviews".to_string(),
                }),
            ));
        }
    };

    let average_rating = match state.get_cached_rating(target_id, &review_type).await {
        Ok(avg) => avg.unwrap_or(0.0),
        Err(e) => {
            error!("Database error: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to calculate average rating".to_string(),
                }),
            ));
        }
    };

    Ok(Json(ReviewsResponse {
        reviews,
        total_reviews,
        average_rating,
        review_type: review_type_str,
        target_id,
    }))
}

async fn add_review(
    State(state): State<AppState>,
    Path((review_type_str, target_id)): Path<(String, i64)>,
    Json(payload): Json<ReviewRequest>,
) -> Result<Json<Review>, (StatusCode, Json<ErrorResponse>)> {
    let review_type = match review_type_str.as_str() {
        "user" => ReviewType::User,
        "server" => ReviewType::Server,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid review type".to_string(),
                }),
            ));
        }
    };

    match state.db.add_review(
        target_id,
        payload.reviewer_id,
        payload.rating,
        payload.comment,
        &review_type,
    ).await {
        Ok(review) => Ok(Json(review)),
        Err(e) => {
            error!("Database error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to add review".to_string(),
                }),
            ))
        }
    }
} 