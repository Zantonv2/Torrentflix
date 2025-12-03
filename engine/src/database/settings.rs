use std::sync::Arc;
use anyhow::{Result, Context};
use sqlx::SqlitePool;
use tracing::info;
use serde::{Serialize, Deserialize};

use super::connection::Database;
use super::transaction::{with_retry, RetryConfig};

/// Database connection for settings operations
pub struct SettingsDatabase {
    database: Arc<Database>,
}

/// A single settings record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SettingRecord {
    pub id: i64,
    pub key: String,
    pub value: String,
    pub r#type: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl SettingsDatabase {
    /// Create a new settings database connection
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Initialize settings schema
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing settings schema");
        
        // Create settings table with key-value schema
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY,
                key TEXT UNIQUE NOT NULL,
                value TEXT NOT NULL,
                type TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(self.database.pool())
        .await
        .context("Failed to create settings table")?;

        // Create index on key for faster lookups
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_settings_key ON settings(key)
            "#,
        )
        .execute(self.database.pool())
        .await
        .context("Failed to create settings index")?;

        info!("Settings schema initialized successfully");
        Ok(())
    }

    /// Get a setting by key
    pub async fn get(&self, key: &str) -> Result<Option<SettingRecord>> {
        let record = sqlx::query_as::<_, SettingRecord>(
            "SELECT id, key, value, type, created_at, updated_at FROM settings WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(self.database.pool())
        .await
        .context("Failed to fetch setting")?;

        Ok(record)
    }

    /// Get all settings
    pub async fn get_all(&self) -> Result<Vec<SettingRecord>> {
        let records = sqlx::query_as::<_, SettingRecord>(
            "SELECT id, key, value, type, created_at, updated_at FROM settings ORDER BY key"
        )
        .fetch_all(self.database.pool())
        .await
        .context("Failed to fetch all settings")?;

        Ok(records)
    }

    /// Set a setting (insert or update) with retry logic
    pub async fn set(&self, key: &str, value: &str, setting_type: &str) -> Result<()> {
        let key = key.to_string();
        let value = value.to_string();
        let setting_type = setting_type.to_string();
        let pool = self.database.pool().clone();
        
        with_retry(
            || {
                let key = key.clone();
                let value = value.clone();
                let setting_type = setting_type.clone();
                let pool = pool.clone();
                
                async move {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .context("Failed to get current time")?
                        .as_secs() as i64;

                    // Try to update first
                    let updated = sqlx::query(
                        "UPDATE settings SET value = ?, type = ?, updated_at = ? WHERE key = ?"
                    )
                    .bind(&value)
                    .bind(&setting_type)
                    .bind(now)
                    .bind(&key)
                    .execute(&pool)
                    .await
                    .context("Failed to update setting")?;

                    // If no rows were updated, insert a new one
                    if updated.rows_affected() == 0 {
                        sqlx::query(
                            "INSERT INTO settings (key, value, type, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
                        )
                        .bind(&key)
                        .bind(&value)
                        .bind(&setting_type)
                        .bind(now)
                        .bind(now)
                        .execute(&pool)
                        .await
                        .context("Failed to insert setting")?;
                    }

                    Ok(())
                }
            },
            &RetryConfig::default(),
        )
        .await
    }

    /// Set multiple settings in a single transaction
    pub async fn set_many(&self, settings: Vec<(String, String, String)>) -> Result<()> {
        let pool = self.database.pool();
        
        let start = std::time::Instant::now();
        info!("Starting settings transaction");
        
        let mut tx = pool.begin().await
            .context("Failed to begin transaction")?;
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .context("Failed to get current time")?
            .as_secs() as i64;
        
        for (key, value, setting_type) in settings {
            // Try to update first
            let updated = sqlx::query(
                "UPDATE settings SET value = ?, type = ?, updated_at = ? WHERE key = ?"
            )
            .bind(&value)
            .bind(&setting_type)
            .bind(now)
            .bind(&key)
            .execute(&mut *tx)
            .await
            .context("Failed to update setting")?;
            
            // If no rows were updated, insert a new one
            if updated.rows_affected() == 0 {
                sqlx::query(
                    "INSERT INTO settings (key, value, type, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
                )
                .bind(&key)
                .bind(&value)
                .bind(&setting_type)
                .bind(now)
                .bind(now)
                .execute(&mut *tx)
                .await
                .context("Failed to insert setting")?;
            }
        }
        
        tx.commit().await
            .context("Failed to commit transaction")?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            "Settings transaction committed successfully"
        );
        
        Ok(())
    }

    /// Delete a setting by key
    pub async fn delete(&self, key: &str) -> Result<()> {
        sqlx::query("DELETE FROM settings WHERE key = ?")
            .bind(key)
            .execute(self.database.pool())
            .await
            .context("Failed to delete setting")?;

        Ok(())
    }

    /// Clear all settings
    pub async fn clear_all(&self) -> Result<()> {
        sqlx::query("DELETE FROM settings")
            .execute(self.database.pool())
            .await
            .context("Failed to clear all settings")?;

        Ok(())
    }

    /// Get the connection pool
    pub fn pool(&self) -> &SqlitePool {
        self.database.pool()
    }
}
