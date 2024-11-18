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
}

impl AppState {
    pub fn new(db: Database, _cache: Cache) -> Self {
        Self { db }
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
    pub target_id: i64,
    pub review_type: String,
    pub average_rating: f64,
    pub total_reviews: i64,
    pub reviews: Vec<Review>,
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

    let reviews = match state.db.get_paginated_reviews(target_id, 0, 50, &review_type).await {
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

    let average_rating = match state.db.get_average_rating(target_id, &review_type).await {
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
        target_id,
        review_type: review_type_str,
        average_rating,
        total_reviews,
        reviews,
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

    match state.db.has_reviewed(target_id, payload.reviewer_id, &review_type).await {
        Ok(true) => {
            match state.db.update_review(
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
                            error: "Failed to update review".to_string(),
                        }),
                    ))
                }
            }
        },
        Ok(false) => {
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
        },
        Err(e) => {
            error!("Database error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to check existing review".to_string(),
                }),
            ))
        }
    }
} 