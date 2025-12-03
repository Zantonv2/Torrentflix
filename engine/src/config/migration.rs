use anyhow::{Result, Context};
use sqlx::SqlitePool;
use tracing::{info, debug};

/// Configuration migration manager for schema versioning
pub struct ConfigMigration;

impl ConfigMigration {
    /// Initialize the migration system and run pending migrations
    pub async fn initialize(pool: &SqlitePool) -> Result<()> {
        info!("Initializing migration system");
        
        // Create schema_version table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schema_version (
                id INTEGER PRIMARY KEY,
                version INTEGER NOT NULL UNIQUE,
                description TEXT NOT NULL,
                applied_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create schema_version table")?;

        // Get current schema version
        let current_version: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version"
        )
        .fetch_one(pool)
        .await
        .context("Failed to fetch current schema version")?;

        let current_version = current_version as u32;
        debug!("Current schema version: {}", current_version);

        // Run migrations
        Self::run_migrations(pool, current_version).await?;

        info!("Migration system initialized");
        Ok(())
    }

    /// Run all pending migrations
    async fn run_migrations(pool: &SqlitePool, current_version: u32) -> Result<()> {
        let migrations = vec![
            (1u32, "Create settings table", Self::migration_001),
        ];

        for (version, description, migration_fn) in migrations {
            if version > current_version {
                info!("Running migration {}: {}", version, description);
                migration_fn(pool).await?;
                
                // Record migration
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .context("Failed to get current time")?
                    .as_secs() as i64;

                sqlx::query(
                    "INSERT INTO schema_version (version, description, applied_at) VALUES (?, ?, ?)"
                )
                .bind(version as i64)
                .bind(description)
                .bind(now)
                .execute(pool)
                .await
                .context(format!("Failed to record migration {}", version))?;

                info!("Migration {} completed", version);
            }
        }

        Ok(())
    }

    /// Migration 001: Create settings table
    async fn migration_001(pool: &SqlitePool) -> Result<()> {
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
        .execute(pool)
        .await
        .context("Failed to create settings table in migration")?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_settings_key ON settings(key)
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create settings index in migration")?;

        Ok(())
    }

    /// Get current schema version
    pub async fn get_version(pool: &SqlitePool) -> Result<u32> {
        let version: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version"
        )
        .fetch_one(pool)
        .await
        .context("Failed to fetch schema version")?;

        Ok(version as u32)
    }
}
