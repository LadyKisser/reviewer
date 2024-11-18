use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use crate::app::{
    models::review::{Review, ReviewType},
    utils::cache::Cache,
};
use std::time::Duration;

#[derive(Clone)]
pub struct Database {
    pool: Pool<Postgres>,
    cache: Cache,
}

impl Database {
    pub async fn new(database_url: &str, cache: Cache) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(database_url)
            .await?;

        Ok(Self { pool, cache })
    }

    pub async fn add_review(
        &self,
        target_id: i64,
        reviewer_id: i64,
        rating: i32,
        comment: Option<String>,
        review_type: &ReviewType,
    ) -> Result<Review, sqlx::Error> {
        let review = sqlx::query_as!(
            Review,
            r#"
            INSERT INTO reviews (target_id, reviewer_id, rating, comment, review_type)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, target_id, reviewer_id, rating, comment, 
                review_type as "review_type: ReviewType",
                created_at
            "#,
            target_id,
            reviewer_id,
            rating,
            comment,
            review_type as &ReviewType
        )
        .fetch_one(&self.pool)
        .await?;

        let _ = self.cache.invalidate(target_id, review_type.as_str()).await;

        Ok(review)
    }

    pub async fn get_average_rating(
        &self,
        target_id: i64,
        review_type: &ReviewType,
    ) -> Result<Option<f64>, sqlx::Error> {
        let prefix = review_type.as_str();

        if let Ok(Some(rating)) = self.cache.get_rating(target_id, prefix).await {
            return Ok(Some(rating));
        }

        let result = sqlx::query!(
            r#"
            SELECT AVG(rating::float) as average 
            FROM reviews 
            WHERE target_id = $1 AND review_type = $2
            "#,
            target_id,
            review_type as &ReviewType
        )
        .fetch_one(&self.pool)
        .await?;

        if let Some(avg) = result.average {
            let _ = self.cache.cache_rating(target_id, avg, prefix).await;
        }

        Ok(result.average)
    }

    pub async fn get_reviews_count(
        &self,
        target_id: i64,
        review_type: &ReviewType,
    ) -> Result<i64, sqlx::Error> {
        let prefix = review_type.as_str();

        if let Ok(Some(count)) = self.cache.get_count(target_id, prefix).await {
            return Ok(count);
        }

        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count 
            FROM reviews 
            WHERE target_id = $1 AND review_type = $2
            "#,
            target_id,
            review_type as &ReviewType
        )
        .fetch_one(&self.pool)
        .await?;

        let count = result.count.unwrap_or(0);
        let _ = self.cache.cache_count(target_id, count, prefix).await;

        Ok(count)
    }

    pub async fn get_reviews(
        &self,
        target_id: i64,
        review_type: &ReviewType,
    ) -> Result<Vec<Review>, sqlx::Error> {
        sqlx::query_as!(
            Review,
            r#"
            SELECT id, target_id, reviewer_id, rating, comment, 
                review_type as "review_type: ReviewType",
                created_at
            FROM reviews 
            WHERE target_id = $1 AND review_type = $2
            ORDER BY created_at DESC
            "#,
            target_id,
            review_type as &ReviewType
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn has_reviewed(
        &self,
        target_id: i64,
        reviewer_id: i64,
        review_type: &ReviewType,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT EXISTS(
                SELECT 1 
                FROM reviews 
                WHERE target_id = $1 AND reviewer_id = $2 AND review_type = $3
            ) as exists
            "#,
            target_id,
            reviewer_id,
            review_type as &ReviewType
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.exists.unwrap_or(false))
    }

    pub async fn update_review(
        &self,
        target_id: i64,
        reviewer_id: i64,
        rating: i32,
        comment: Option<String>,
        review_type: &ReviewType,
    ) -> Result<Review, sqlx::Error> {
        let review = sqlx::query_as!(
            Review,
            r#"
            UPDATE reviews 
            SET rating = $3, comment = $4, created_at = CURRENT_TIMESTAMP
            WHERE target_id = $1 AND reviewer_id = $2 AND review_type = $5
            RETURNING id, target_id, reviewer_id, rating, comment, 
                review_type as "review_type: ReviewType",
                created_at
            "#,
            target_id,
            reviewer_id,
            rating,
            comment,
            review_type as &ReviewType
        )
        .fetch_one(&self.pool)
        .await?;

        let _ = self.cache.invalidate(target_id, review_type.as_str()).await;

        Ok(review)
    }

    pub async fn get_paginated_reviews(
        &self,
        target_id: i64,
        page: i64,
        per_page: i64,
        review_type: &ReviewType,
    ) -> Result<Vec<Review>, sqlx::Error> {
        sqlx::query_as!(
            Review,
            r#"
            SELECT id, target_id, reviewer_id, rating, comment, 
                review_type as "review_type: ReviewType",
                created_at
            FROM reviews 
            WHERE target_id = $1 AND review_type = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            target_id,
            review_type as &ReviewType,
            per_page,
            page * per_page
        )
        .fetch_all(&self.pool)
        .await
    }
}