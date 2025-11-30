use std::sync::Arc;
use anyhow::Result;
use sqlx::SqlitePool;
use tracing::info;

use super::connection::Database;

/// Database connection for library operations
pub struct LibraryDatabase {
    database: Arc<Database>,
}

impl LibraryDatabase {
    /// Create a new library database connection
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Initialize library schema
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing library schema");
        // TODO: Create library tables (media_items, file_versions, etc.)
        // For now, this is a placeholder
        Ok(())
    }

    /// Get the connection pool
    pub fn pool(&self) -> &SqlitePool {
        self.database.pool()
    }
}

