use std::sync::Arc;
use anyhow::{Result, Context};
use sqlx::SqlitePool;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::{info, debug};

use super::connection::Database;

/// Database-backed rating cache
pub struct RatingCacheDatabase {
    database: Arc<Database>,
}

impl RatingCacheDatabase {
    /// Create a new rating cache database
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Initialize rating cache schema
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing rating cache schema");

        // Create rating_cache table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rating_cache (
                movie_id TEXT PRIMARY KEY,
                rating_kinopoisk REAL NULL,
                rating_imdb REAL NULL,
                rating_tmdb REAL NULL,
                cached_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(self.database.pool())
        .await
        .context("Failed to create rating_cache table")?;

        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_rating_cache_cached_at 
            ON rating_cache(cached_at)
            "#,
        )
        .execute(self.database.pool())
        .await?;

        info!("Rating cache schema initialized");
        Ok(())
    }

    /// Get rating from cache
    pub async fn get_rating(&self, movie_id: &str) -> Result<Option<CachedRatingRecord>> {
        use sqlx::Row;

        let row = sqlx::query(
            r#"
            SELECT movie_id, rating_kinopoisk, rating_imdb, rating_tmdb, cached_at, updated_at
            FROM rating_cache
            WHERE movie_id = ?
            "#,
        )
        .bind(movie_id)
        .fetch_optional(self.database.pool())
        .await?;

        if let Some(row) = row {
            Ok(Some(CachedRatingRecord {
                movie_id: row.try_get("movie_id")?,
                rating_kinopoisk: row.try_get("rating_kinopoisk")?,
                rating_imdb: row.try_get("rating_imdb")?,
                rating_tmdb: row.try_get("rating_tmdb")?,
                cached_at: {
                    let ts: i64 = row.try_get("cached_at")?;
                    DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now)
                },
                updated_at: {
                    let ts: i64 = row.try_get("updated_at")?;
                    DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now)
                },
            }))
        } else {
            Ok(None)
        }
    }

    /// Store rating in cache
    pub async fn set_rating(&self, record: &CachedRatingRecord) -> Result<()> {
        let now = Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO rating_cache 
            (movie_id, rating_kinopoisk, rating_imdb, rating_tmdb, cached_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&record.movie_id)
        .bind(record.rating_kinopoisk)
        .bind(record.rating_imdb)
        .bind(record.rating_tmdb)
        .bind(record.cached_at.timestamp())
        .bind(now)
        .execute(self.database.pool())
        .await?;

        debug!("Stored rating in database cache for movie_id: {}", record.movie_id);
        Ok(())
    }

    /// Update specific rating fields (partial update)
    /// For simplicity, we'll just get the existing record, update it, and save it back
    pub async fn update_rating_partial(
        &self,
        movie_id: &str,
        rating_kinopoisk: Option<f32>,
        rating_imdb: Option<f32>,
        rating_tmdb: Option<f32>,
    ) -> Result<()> {
        // Get existing record or create new one
        let mut record = match self.get_rating(movie_id).await? {
            Some(existing) => existing,
            None => CachedRatingRecord::new(movie_id.to_string()),
        };

        // Update only provided fields
        if let Some(kp) = rating_kinopoisk {
            record.rating_kinopoisk = Some(kp);
        }
        if let Some(imdb) = rating_imdb {
            record.rating_imdb = Some(imdb);
        }
        if let Some(tmdb) = rating_tmdb {
            record.rating_tmdb = Some(tmdb);
        }

        // Save back
        self.set_rating(&record).await?;
        Ok(())
    }

    /// Clean up old cache entries (older than specified days)
    pub async fn cleanup_old_entries(&self, days: i64) -> Result<usize> {
        let cutoff = Utc::now().timestamp() - (days * 24 * 60 * 60);

        let result = sqlx::query(
            r#"
            DELETE FROM rating_cache
            WHERE updated_at < ?
            "#,
        )
        .bind(cutoff)
        .execute(self.database.pool())
        .await?;

        let deleted = result.rows_affected() as usize;
        if deleted > 0 {
            info!("Cleaned up {} old rating cache entries", deleted);
        }
        Ok(deleted)
    }

    /// Clear all IMDb ratings from cache (set rating_imdb to NULL)
    pub async fn clear_imdb_ratings(&self) -> Result<usize> {
        let result = sqlx::query(
            r#"
            UPDATE rating_cache
            SET rating_imdb = NULL, updated_at = ?
            WHERE rating_imdb IS NOT NULL
            "#,
        )
        .bind(Utc::now().timestamp())
        .execute(self.database.pool())
        .await?;

        let cleared = result.rows_affected() as usize;
        if cleared > 0 {
            info!("Cleared {} IMDb ratings from cache", cleared);
        }
        Ok(cleared)
    }

    /// Clear all ratings from cache (delete all entries)
    pub async fn clear_all_ratings(&self) -> Result<usize> {
        let result = sqlx::query(
            r#"
            DELETE FROM rating_cache
            "#,
        )
        .execute(self.database.pool())
        .await?;

        let deleted = result.rows_affected() as usize;
        if deleted > 0 {
            info!("Cleared all {} rating cache entries", deleted);
        }
        Ok(deleted)
    }

    /// Get the connection pool
    pub fn pool(&self) -> &SqlitePool {
        self.database.pool()
    }
}

/// Cached rating record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedRatingRecord {
    pub movie_id: String,
    pub rating_kinopoisk: Option<f32>,
    pub rating_imdb: Option<f32>,
    pub rating_tmdb: Option<f32>,
    pub cached_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CachedRatingRecord {
    pub fn new(movie_id: String) -> Self {
        let now = Utc::now();
        Self {
            movie_id,
            rating_kinopoisk: None,
            rating_imdb: None,
            rating_tmdb: None,
            cached_at: now,
            updated_at: now,
        }
    }

    pub fn with_kinopoisk(mut self, rating: Option<f32>) -> Self {
        self.rating_kinopoisk = rating;
        self
    }

    pub fn with_imdb(mut self, rating: Option<f32>) -> Self {
        self.rating_imdb = rating;
        self
    }

    pub fn with_tmdb(mut self, rating: Option<f32>) -> Self {
        self.rating_tmdb = rating;
        self
    }
}

