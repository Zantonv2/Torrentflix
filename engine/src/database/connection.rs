use anyhow::{Result, Context};
use sqlx::{SqlitePool, sqlite::{SqlitePoolOptions, SqliteConnectOptions}};
use tracing::{info, error, warn, debug};
use std::str::FromStr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Connection pool statistics for monitoring
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Total number of connections acquired
    pub total_acquired: Arc<AtomicU64>,
    /// Total number of connections released
    pub total_released: Arc<AtomicU64>,
    /// Number of currently active connections
    pub active_connections: Arc<AtomicU64>,
}

impl PoolStats {
    pub fn new() -> Self {
        Self {
            total_acquired: Arc::new(AtomicU64::new(0)),
            total_released: Arc::new(AtomicU64::new(0)),
            active_connections: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record a connection acquisition
    pub fn record_acquire(&self) {
        self.total_acquired.fetch_add(1, Ordering::Relaxed);
        self.active_connections.fetch_add(1, Ordering::Relaxed);
        debug!(
            "Connection acquired - Active: {}, Total acquired: {}",
            self.active_connections.load(Ordering::Relaxed),
            self.total_acquired.load(Ordering::Relaxed)
        );
    }

    /// Record a connection release
    pub fn record_release(&self) {
        self.total_released.fetch_add(1, Ordering::Relaxed);
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
        debug!(
            "Connection released - Active: {}, Total released: {}",
            self.active_connections.load(Ordering::Relaxed),
            self.total_released.load(Ordering::Relaxed)
        );
    }

    /// Get current statistics
    pub fn get_stats(&self) -> (u64, u64, u64) {
        (
            self.total_acquired.load(Ordering::Relaxed),
            self.total_released.load(Ordering::Relaxed),
            self.active_connections.load(Ordering::Relaxed),
        )
    }

    /// Log current pool statistics
    pub fn log_stats(&self) {
        let (acquired, released, active) = self.get_stats();
        info!(
            "Connection pool stats - Acquired: {}, Released: {}, Active: {}",
            acquired, released, active
        );
    }
}

impl Default for PoolStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Main database connection manager
pub struct Database {
    pool: SqlitePool,
    stats: PoolStats,
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

        let stats = PoolStats::new();
        let db = Self { pool, stats };
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
        
        // Set busy timeout to 5 seconds to handle lock contention
        sqlx::query("PRAGMA busy_timeout = 5000")
            .execute(&self.pool)
            .await?;

        // Run migrations
        use crate::config::migration::ConfigMigration;
        ConfigMigration::initialize(&self.pool).await?;

        info!("Database initialized with WAL mode and busy_timeout");
        Ok(())
    }

    /// Get the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get pool statistics
    pub fn stats(&self) -> &PoolStats {
        &self.stats
    }

    /// Acquire a connection from the pool with tracking
    pub async fn acquire(&self) -> Result<TrackedConnection> {
        debug!("Acquiring connection from pool");
        let conn = self.pool.acquire().await
            .context("Failed to acquire connection from pool")?;
        self.stats.record_acquire();
        Ok(TrackedConnection::new(conn, self.stats.clone()))
    }

    /// Log current pool statistics
    pub fn log_pool_stats(&self) {
        self.stats.log_stats();
    }
}

/// RAII guard for connection tracking
/// Automatically records release when dropped
pub struct TrackedConnection {
    conn: Option<sqlx::pool::PoolConnection<sqlx::Sqlite>>,
    stats: PoolStats,
}

impl TrackedConnection {
    pub fn new(conn: sqlx::pool::PoolConnection<sqlx::Sqlite>, stats: PoolStats) -> Self {
        Self {
            conn: Some(conn),
            stats,
        }
    }

    /// Get a mutable reference to the connection
    pub fn as_mut(&mut self) -> &mut sqlx::pool::PoolConnection<sqlx::Sqlite> {
        self.conn.as_mut().expect("Connection already taken")
    }
}

impl Drop for TrackedConnection {
    fn drop(&mut self) {
        if self.conn.is_some() {
            self.stats.record_release();
        }
    }
}

impl std::ops::Deref for TrackedConnection {
    type Target = sqlx::pool::PoolConnection<sqlx::Sqlite>;

    fn deref(&self) -> &Self::Target {
        self.conn.as_ref().expect("Connection already taken")
    }
}

impl std::ops::DerefMut for TrackedConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.conn.as_mut().expect("Connection already taken")
    }
}

