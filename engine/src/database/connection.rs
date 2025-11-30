use anyhow::{Result, Context};
use sqlx::{SqlitePool, sqlite::{SqlitePoolOptions, SqliteConnectOptions}};
use tracing::{info, error, warn};
use std::str::FromStr;
use std::path::Path;

/// Main database connection manager
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Attempting to connect to database: {}", database_url);
        
        // Extract the actual file path from the connection string
        // sqlx 0.8 SqliteConnectOptions accepts file paths directly
        let file_path = if database_url.starts_with("sqlite://") {
            database_url.strip_prefix("sqlite://").unwrap()
        } else if database_url.starts_with("sqlite:") {
            database_url.strip_prefix("sqlite:").unwrap()
        } else {
            database_url
        };
        
        // Ensure the directory exists (only for relative paths, not absolute root paths)
        if let Some(parent) = Path::new(file_path).parent() {
            // Don't try to create root directory (/)
            if parent != Path::new("/") && !parent.exists() {
                info!("Creating database directory: {:?}", parent);
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create database directory: {:?}", parent))?;
            }
        }
        
        // Warn if using root directory (likely a mistake)
        if file_path.starts_with("/") && !file_path.starts_with("/home") && !file_path.starts_with("/tmp") {
            warn!("Database path is in root filesystem: {}. This may not be intended.", file_path);
            warn!("Consider using a path in your home directory or project directory instead.");
        }
        
        info!("Connecting to SQLite database file: {}", file_path);
        
        // Use SqliteConnectOptions which properly handles file paths
        let connect_options = SqliteConnectOptions::from_str(file_path)
            .with_context(|| format!("Failed to parse database path: {}", file_path))?
            .create_if_missing(true); // Create database file if it doesn't exist
        
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect_with(connect_options)
            .await
            .with_context(|| {
                error!("Database connection failed for path: {}", file_path);
                format!("Failed to connect to database at: {}", file_path)
            })?;

        let db = Self { pool };
        db.initialize().await?;
        Ok(db)
    }

    /// Initialize database schema and pragmas
    async fn initialize(&self) -> Result<()> {
        info!("Initializing database");

        // Set SQLite pragmas for optimal performance
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&self.pool)
            .await?;
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&self.pool)
            .await?;
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&self.pool)
            .await?;

        info!("Database initialized with WAL mode");
        Ok(())
    }

    /// Get the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

