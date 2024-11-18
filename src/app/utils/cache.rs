use redis::{Client, Commands, RedisResult};
use tracing::{error, info};

const CACHE_DURATION: usize = 600;

#[derive(Clone)]
pub struct Cache {
    client: Client,
}

impl Cache {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn cache_rating(&self, target_id: i64, average: f64, prefix: &str) -> RedisResult<()> {
        let mut conn = self.client.get_connection()?;
        let key = format!("{}:{}:rating", prefix, target_id);
        let _: () = conn.set_ex(key, average.to_string(), CACHE_DURATION)?;
        info!("Cached {} rating for ID {}: {}", prefix, target_id, average);
        Ok(())
    }

    pub async fn get_rating(&self, target_id: i64, prefix: &str) -> RedisResult<Option<f64>> {
        let mut conn = self.client.get_connection()?;
        let key = format!("{}:{}:rating", prefix, target_id);
        let value: Option<String> = conn.get(key)?;
        
        match value {
            Some(v) => match v.parse() {
                Ok(rating) => {
                    info!("Cache hit for {} rating ID {}", prefix, target_id);
                    Ok(Some(rating))
                },
                Err(e) => {
                    error!("Failed to parse cached rating: {}", e);
                    Ok(None)
                }
            },
            None => {
                info!("Cache miss for {} rating ID {}", prefix, target_id);
                Ok(None)
            }
        }
    }

    pub async fn cache_count(&self, target_id: i64, count: i64, prefix: &str) -> RedisResult<()> {
        let mut conn = self.client.get_connection()?;
        let key = format!("{}:{}:count", prefix, target_id);
        let _: () = conn.set_ex(key, count.to_string(), CACHE_DURATION)?;
        info!("Cached {} count for ID {}: {}", prefix, target_id, count);
        Ok(())
    }

    pub async fn get_count(&self, target_id: i64, prefix: &str) -> RedisResult<Option<i64>> {
        let mut conn = self.client.get_connection()?;
        let key = format!("{}:{}:count", prefix, target_id);
        let value: Option<String> = conn.get(key)?;
        
        match value {
            Some(v) => match v.parse() {
                Ok(count) => {
                    info!("Cache hit for {} count ID {}", prefix, target_id);
                    Ok(Some(count))
                },
                Err(e) => {
                    error!("Failed to parse cached count: {}", e);
                    Ok(None)
                }
            },
            None => {
                info!("Cache miss for {} count ID {}", prefix, target_id);
                Ok(None)
            }
        }
    }

    pub async fn invalidate(&self, target_id: i64, prefix: &str) -> RedisResult<()> {
        let mut conn = self.client.get_connection()?;
        let rating_key = format!("{}:{}:rating", prefix, target_id);
        let count_key = format!("{}:{}:count", prefix, target_id);
        
        let _: () = conn.del(&[rating_key, count_key])?;
        info!("Invalidated cache for {} ID {}", prefix, target_id);
        Ok(())
    }
} 