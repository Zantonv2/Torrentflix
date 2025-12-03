use std::sync::Arc;
use anyhow::{Result, Context};
use tracing::{info, warn, debug};

use crate::database::SettingsDatabase;
use crate::database::transaction::{with_retry, RetryConfig};
use super::models::Settings;
use super::validation::{
    validate_url, validate_path, validate_numeric_range, validate_numeric_range_u16,
    validate_numeric_range_u64, ValidationError,
};

/// Settings manager for loading, saving, and validating application settings
pub struct SettingsManager {
    db: Arc<SettingsDatabase>,
}

impl SettingsManager {
    /// Create a new settings manager
    pub fn new(db: Arc<SettingsDatabase>) -> Self {
        Self { db }
    }

    /// Load settings from database with defaults for missing values
    ///
    /// This function loads all settings from the database and fills in defaults
    /// for any missing or invalid settings. It ensures the application can always
    /// function even if the database is corrupted or incomplete.
    ///
    /// # Returns
    /// * `Ok(Settings)` - Loaded settings with defaults applied (never fails, uses defaults on error)
    pub async fn load_settings(&self) -> Result<Settings> {
        info!("Loading settings from database");

        // Get all settings from database - if this fails, use all defaults
        let records = match self.db.get_all().await {
            Ok(records) => records,
            Err(e) => {
                warn!("Failed to load settings from database: {}. Using default settings.", e);
                // Return default settings if database is unavailable
                return Ok(Settings::default());
            }
        };

        // Start with defaults
        let mut settings = Settings::default();

        // Override with database values
        for record in records {
            match record.key.as_str() {
                // API & Metadata
                "tmdb_api_key" => settings.tmdb_api_key = if record.value.is_empty() { None } else { Some(record.value) },
                "tmdb_read_access_token" => settings.tmdb_read_access_token = if record.value.is_empty() { None } else { Some(record.value) },
                "kinopoisk_api_token" => settings.kinopoisk_api_token = if record.value.is_empty() { None } else { Some(record.value) },

                // qBittorrent
                "qbittorrent_enabled" => {
                    settings.qbittorrent_enabled = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse qbittorrent_enabled '{}': {}. Using default: true", record.value, e);
                        true
                    });
                }
                "qbittorrent_host" => settings.qbittorrent_host = record.value,
                "qbittorrent_port" => {
                    settings.qbittorrent_port = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse qbittorrent_port '{}': {}. Using default: 5555", record.value, e);
                        5555
                    });
                }
                "qbittorrent_username" => settings.qbittorrent_username = record.value,
                "qbittorrent_password" => settings.qbittorrent_password = record.value,
                "qbittorrent_use_ssl" => {
                    settings.qbittorrent_use_ssl = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse qbittorrent_use_ssl '{}': {}. Using default: false", record.value, e);
                        false
                    });
                }
                "qbittorrent_verify_ssl" => {
                    settings.qbittorrent_verify_ssl = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse qbittorrent_verify_ssl '{}': {}. Using default: false", record.value, e);
                        false
                    });
                }
                "qbittorrent_category" => settings.qbittorrent_category = if record.value.is_empty() { None } else { Some(record.value) },
                "qbittorrent_tags" => settings.qbittorrent_tags = if record.value.is_empty() { None } else { Some(record.value) },
                "qbittorrent_timeout" => {
                    settings.qbittorrent_timeout = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse qbittorrent_timeout '{}': {}. Using default: 10", record.value, e);
                        10
                    });
                }

                // Filesystem
                "library_path" => settings.library_path = record.value,
                "download_path" => settings.download_path = record.value,

                // Notifications
                "telegram_enabled" => {
                    settings.telegram_enabled = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse telegram_enabled '{}': {}. Using default: false", record.value, e);
                        false
                    });
                }
                "telegram_bot_token" => settings.telegram_bot_token = if record.value.is_empty() { None } else { Some(record.value) },
                "telegram_chat_id" => settings.telegram_chat_id = if record.value.is_empty() { None } else { Some(record.value) },
                "telegram_debounce" => {
                    settings.telegram_debounce = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse telegram_debounce '{}': {}. Using default: 300", record.value, e);
                        300
                    });
                }
                "email_enabled" => {
                    settings.email_enabled = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse email_enabled '{}': {}. Using default: false", record.value, e);
                        false
                    });
                }
                "smtp_host" => settings.smtp_host = record.value,
                "smtp_port" => {
                    settings.smtp_port = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse smtp_port '{}': {}. Using default: 587", record.value, e);
                        587
                    });
                }
                "smtp_user" => settings.smtp_user = if record.value.is_empty() { None } else { Some(record.value) },
                "smtp_password" => settings.smtp_password = if record.value.is_empty() { None } else { Some(record.value) },
                "from_email" => settings.from_email = if record.value.is_empty() { None } else { Some(record.value) },
                "to_email" => settings.to_email = if record.value.is_empty() { None } else { Some(record.value) },

                // Application
                "log_level" => settings.log_level = record.value,
                "log_format" => settings.log_format = record.value,
                "debug" => {
                    settings.debug = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse debug '{}': {}. Using default: false", record.value, e);
                        false
                    });
                }
                "search_limit" => {
                    settings.search_limit = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse search_limit '{}': {}. Using default: 100", record.value, e);
                        100
                    });
                }
                "search_timeout" => {
                    settings.search_timeout = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse search_timeout '{}': {}. Using default: 30", record.value, e);
                        30
                    });
                }
                "max_concurrent_downloads" => {
                    settings.max_concurrent_downloads = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse max_concurrent_downloads '{}': {}. Using default: 3", record.value, e);
                        3
                    });
                }
                "enabled_indexers" => settings.enabled_indexers = record.value,

                // Proxy
                "proxy_enabled" => {
                    settings.proxy_enabled = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse proxy_enabled '{}': {}. Using default: false", record.value, e);
                        false
                    });
                }
                "proxy_address" => settings.proxy_address = record.value,
                "proxy_port" => {
                    settings.proxy_port = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse proxy_port '{}': {}. Using default: 1080", record.value, e);
                        1080
                    });
                }

                // Database
                "database_url" => settings.database_url = record.value,
                "db_pool_size" => {
                    settings.db_pool_size = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse db_pool_size '{}': {}. Using default: 10", record.value, e);
                        10
                    });
                }
                "db_max_overflow" => {
                    settings.db_max_overflow = record.value.parse().unwrap_or_else(|e| {
                        warn!("Failed to parse db_max_overflow '{}': {}. Using default: 20", record.value, e);
                        20
                    });
                }

                _ => {
                    warn!("Unknown setting key: {}", record.key);
                }
            }
        }

        info!("Settings loaded successfully");
        Ok(settings)
    }

    /// Save settings to database after validation
    ///
    /// This function validates all settings before persisting them to the database.
    /// If validation fails, no settings are saved and an error is returned.
    /// All settings are saved in a single transaction with automatic retry logic.
    ///
    /// # Arguments
    /// * `settings` - The settings to save
    ///
    /// # Returns
    /// * `Ok(())` - If settings were saved successfully
    /// * `Err(anyhow::Error)` - If validation failed or database operations failed
    pub async fn save_settings(&self, settings: &Settings) -> Result<()> {
        info!("Saving settings to database");

        // Validate all settings before saving (no database changes if validation fails)
        self.validate_settings(settings).context("Settings validation failed")?;

        // Collect all settings into a vector for batch transaction
        let settings_vec = vec![
            ("tmdb_api_key".to_string(), settings.tmdb_api_key.clone().unwrap_or_default(), "string".to_string()),
            ("tmdb_read_access_token".to_string(), settings.tmdb_read_access_token.clone().unwrap_or_default(), "string".to_string()),
            ("kinopoisk_api_token".to_string(), settings.kinopoisk_api_token.clone().unwrap_or_default(), "string".to_string()),
            
            ("qbittorrent_enabled".to_string(), settings.qbittorrent_enabled.to_string(), "bool".to_string()),
            ("qbittorrent_host".to_string(), settings.qbittorrent_host.clone(), "string".to_string()),
            ("qbittorrent_port".to_string(), settings.qbittorrent_port.to_string(), "int".to_string()),
            ("qbittorrent_username".to_string(), settings.qbittorrent_username.clone(), "string".to_string()),
            ("qbittorrent_password".to_string(), settings.qbittorrent_password.clone(), "string".to_string()),
            ("qbittorrent_use_ssl".to_string(), settings.qbittorrent_use_ssl.to_string(), "bool".to_string()),
            ("qbittorrent_verify_ssl".to_string(), settings.qbittorrent_verify_ssl.to_string(), "bool".to_string()),
            ("qbittorrent_category".to_string(), settings.qbittorrent_category.clone().unwrap_or_default(), "string".to_string()),
            ("qbittorrent_tags".to_string(), settings.qbittorrent_tags.clone().unwrap_or_default(), "string".to_string()),
            ("qbittorrent_timeout".to_string(), settings.qbittorrent_timeout.to_string(), "int".to_string()),
            
            ("library_path".to_string(), settings.library_path.clone(), "string".to_string()),
            ("download_path".to_string(), settings.download_path.clone(), "string".to_string()),
            
            ("telegram_enabled".to_string(), settings.telegram_enabled.to_string(), "bool".to_string()),
            ("telegram_bot_token".to_string(), settings.telegram_bot_token.clone().unwrap_or_default(), "string".to_string()),
            ("telegram_chat_id".to_string(), settings.telegram_chat_id.clone().unwrap_or_default(), "string".to_string()),
            ("telegram_debounce".to_string(), settings.telegram_debounce.to_string(), "int".to_string()),
            ("email_enabled".to_string(), settings.email_enabled.to_string(), "bool".to_string()),
            ("smtp_host".to_string(), settings.smtp_host.clone(), "string".to_string()),
            ("smtp_port".to_string(), settings.smtp_port.to_string(), "int".to_string()),
            ("smtp_user".to_string(), settings.smtp_user.clone().unwrap_or_default(), "string".to_string()),
            ("smtp_password".to_string(), settings.smtp_password.clone().unwrap_or_default(), "string".to_string()),
            ("from_email".to_string(), settings.from_email.clone().unwrap_or_default(), "string".to_string()),
            ("to_email".to_string(), settings.to_email.clone().unwrap_or_default(), "string".to_string()),
            
            ("log_level".to_string(), settings.log_level.clone(), "string".to_string()),
            ("log_format".to_string(), settings.log_format.clone(), "string".to_string()),
            ("debug".to_string(), settings.debug.to_string(), "bool".to_string()),
            ("search_limit".to_string(), settings.search_limit.to_string(), "int".to_string()),
            ("search_timeout".to_string(), settings.search_timeout.to_string(), "int".to_string()),
            ("max_concurrent_downloads".to_string(), settings.max_concurrent_downloads.to_string(), "int".to_string()),
            ("enabled_indexers".to_string(), settings.enabled_indexers.clone(), "string".to_string()),
            
            ("proxy_enabled".to_string(), settings.proxy_enabled.to_string(), "bool".to_string()),
            ("proxy_address".to_string(), settings.proxy_address.clone(), "string".to_string()),
            ("proxy_port".to_string(), settings.proxy_port.to_string(), "int".to_string()),
            
            ("database_url".to_string(), settings.database_url.clone(), "string".to_string()),
            ("db_pool_size".to_string(), settings.db_pool_size.to_string(), "int".to_string()),
            ("db_max_overflow".to_string(), settings.db_max_overflow.to_string(), "int".to_string()),
        ];

        // Save all settings in a single transaction with retry logic
        let db = self.db.clone();
        with_retry(
            || {
                let settings_vec = settings_vec.clone();
                let db = db.clone();
                async move {
                    db.set_many(settings_vec).await
                        .context("Failed to save settings in transaction")
                }
            },
            &RetryConfig::default(),
        )
        .await?;

        info!("Settings saved successfully");
        Ok(())
    }

    /// Validate all settings comprehensively
    ///
    /// This function validates all settings fields and returns a list of validation errors.
    /// If any validation fails, an error is returned with details about which fields are invalid.
    ///
    /// # Arguments
    /// * `settings` - The settings to validate
    ///
    /// # Returns
    /// * `Ok(())` - If all settings are valid
    /// * `Err(anyhow::Error)` - If any validation fails
    pub fn validate_settings(&self, settings: &Settings) -> Result<()> {
        let mut errors: Vec<ValidationError> = Vec::new();

        // Validate qBittorrent URL
        if settings.qbittorrent_enabled {
            if let Err(e) = validate_url(&settings.qbittorrent_url(), "qbittorrent_url") {
                errors.push(e);
            }
        }

        // Validate qBittorrent port
        if let Err(e) = validate_numeric_range_u16(settings.qbittorrent_port, 1, 65535, "qbittorrent_port") {
            errors.push(e);
        }

        // Validate qBittorrent timeout
        if let Err(e) = validate_numeric_range_u64(settings.qbittorrent_timeout, 1, 300, "qbittorrent_timeout") {
            errors.push(e);
        }

        // Validate filesystem paths
        if let Err(e) = validate_path(&settings.library_path, "library_path") {
            errors.push(e);
        }

        if let Err(e) = validate_path(&settings.download_path, "download_path") {
            errors.push(e);
        }

        // Validate Telegram settings when enabled
        if settings.telegram_enabled {
            if let Err(e) = super::validation::validate_required_when_enabled(
                &settings.telegram_bot_token,
                "telegram_bot_token",
                "Telegram notifications"
            ) {
                errors.push(e);
            }

            if let Err(e) = super::validation::validate_required_when_enabled(
                &settings.telegram_chat_id,
                "telegram_chat_id",
                "Telegram notifications"
            ) {
                errors.push(e);
            }

            // Validate debounce range
            if let Err(e) = validate_numeric_range_u64(settings.telegram_debounce, 0, 3600, "telegram_debounce") {
                errors.push(e);
            }
        }

        // Validate Email settings when enabled
        if settings.email_enabled {
            if let Err(e) = validate_numeric_range_u16(settings.smtp_port, 1, 65535, "smtp_port") {
                errors.push(e);
            }

            if let Err(e) = super::validation::validate_required_when_enabled(
                &settings.smtp_user,
                "smtp_user",
                "Email notifications"
            ) {
                errors.push(e);
            }

            if let Err(e) = super::validation::validate_required_when_enabled(
                &settings.smtp_password,
                "smtp_password",
                "Email notifications"
            ) {
                errors.push(e);
            }

            if let Err(e) = super::validation::validate_required_when_enabled(
                &settings.from_email,
                "from_email",
                "Email notifications"
            ) {
                errors.push(e);
            }

            if let Err(e) = super::validation::validate_required_when_enabled(
                &settings.to_email,
                "to_email",
                "Email notifications"
            ) {
                errors.push(e);
            }

            // Validate email addresses
            if let Some(ref from_email) = settings.from_email {
                if !from_email.is_empty() {
                    if let Err(e) = super::validation::validate_email(from_email, "from_email") {
                        errors.push(e);
                    }
                }
            }

            if let Some(ref to_email) = settings.to_email {
                if !to_email.is_empty() {
                    if let Err(e) = super::validation::validate_email(to_email, "to_email") {
                        errors.push(e);
                    }
                }
            }
        }

        // Validate proxy port
        if settings.proxy_enabled {
            if let Err(e) = validate_numeric_range_u16(settings.proxy_port, 1, 65535, "proxy_port") {
                errors.push(e);
            }
        }

        // Validate numeric ranges
        if let Err(e) = validate_numeric_range(settings.search_limit, 1, 10000, "search_limit") {
            errors.push(e);
        }

        if let Err(e) = validate_numeric_range_u64(settings.search_timeout, 1, 300, "search_timeout") {
            errors.push(e);
        }

        if let Err(e) = validate_numeric_range(settings.max_concurrent_downloads, 1, 100, "max_concurrent_downloads") {
            errors.push(e);
        }

        if let Err(e) = validate_numeric_range(settings.db_pool_size, 1, 100, "db_pool_size") {
            errors.push(e);
        }

        if let Err(e) = validate_numeric_range(settings.db_max_overflow, 0, 100, "db_max_overflow") {
            errors.push(e);
        }

        // If there are any errors, return them all
        if !errors.is_empty() {
            let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            let error_summary = error_messages.join("\n");
            warn!("Settings validation failed with {} errors:\n{}", errors.len(), error_summary);
            return Err(anyhow::anyhow!("Settings validation failed:\n{}", error_summary));
        }

        debug!("Settings validation passed");
        Ok(())
    }

    /// Test qBittorrent connection with provided credentials
    ///
    /// This function attempts to connect to qBittorrent with the provided credentials
    /// and returns success or a specific error message.
    ///
    /// # Arguments
    /// * `url` - The qBittorrent URL
    /// * `username` - The qBittorrent username
    /// * `password` - The qBittorrent password
    ///
    /// # Returns
    /// * `Ok(())` - If connection is successful
    /// * `Err(anyhow::Error)` - If connection fails with specific error message
    pub async fn test_qbittorrent_connection(
        &self,
        url: &str,
        username: &str,
        password: &str,
    ) -> Result<()> {
        info!("Testing qBittorrent connection to {}", url);

        // Validate URL format first
        if let Err(e) = validate_url(url, "qbittorrent_url") {
            warn!("qBittorrent URL validation failed: {}", e);
            return Err(anyhow::anyhow!("{}", e));
        }

        // Create a simple HTTP client to test connection
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .context("Failed to create HTTP client")?;

        // Try to authenticate with qBittorrent API
        let auth_url = format!("{}/api/v2/auth/login", url);
        debug!("Attempting to authenticate with qBittorrent at {}", auth_url);
        
        let response = client
            .post(&auth_url)
            .form(&[("username", username), ("password", password)])
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    warn!("qBittorrent connection timeout after 10 seconds");
                    anyhow::anyhow!("Connection timeout: qBittorrent server did not respond within 10 seconds")
                } else if e.is_connect() {
                    warn!("qBittorrent connection refused at {}", url);
                    anyhow::anyhow!("Connection refused: Could not connect to qBittorrent at {}", url)
                } else {
                    warn!("qBittorrent connection error: {}", e);
                    anyhow::anyhow!("Connection failed: {}", e)
                }
            })?;

        // Check response status
        match response.status() {
            reqwest::StatusCode::OK => {
                info!("qBittorrent connection successful");
                Ok(())
            }
            reqwest::StatusCode::FORBIDDEN => {
                warn!("qBittorrent authentication failed: invalid credentials");
                Err(anyhow::anyhow!("Invalid credentials: qBittorrent rejected the username/password"))
            }
            reqwest::StatusCode::NOT_FOUND => {
                warn!("qBittorrent API not found at {}", url);
                Err(anyhow::anyhow!("Invalid URL: qBittorrent API not found at {}", url))
            }
            status => {
                warn!("qBittorrent connection failed with status {}", status);
                Err(anyhow::anyhow!("Connection failed with status {}: {}", status, status.canonical_reason().unwrap_or("Unknown error")))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Helper function to create a test settings manager
    async fn create_test_manager() -> Result<(SettingsManager, Arc<SettingsDatabase>, TempDir)> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        let db = Database::new(&db_url).await?;
        let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
        settings_db.initialize().await?;
        
        let manager = SettingsManager::new(settings_db.clone());
        Ok((manager, settings_db, temp_dir))
    }

    #[tokio::test]
    async fn test_load_settings_with_defaults() -> Result<()> {
        let (manager, _, _) = create_test_manager().await?;
        
        let settings = manager.load_settings().await?;
        
        assert_eq!(settings.qbittorrent_enabled, true);
        assert_eq!(settings.qbittorrent_host, "localhost");
        assert_eq!(settings.qbittorrent_port, 5555);
        assert_eq!(settings.log_level, "INFO");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_save_and_load_settings() -> Result<()> {
        let (manager, _, temp_dir) = create_test_manager().await?;
        
        let mut settings = Settings::default();
        settings.qbittorrent_host = "192.168.1.100".to_string();
        settings.qbittorrent_port = 8080;
        settings.log_level = "DEBUG".to_string();
        settings.library_path = temp_dir.path().to_string_lossy().to_string();
        settings.download_path = temp_dir.path().to_string_lossy().to_string();
        
        manager.save_settings(&settings).await?;
        
        let loaded = manager.load_settings().await?;
        
        assert_eq!(loaded.qbittorrent_host, "192.168.1.100");
        assert_eq!(loaded.qbittorrent_port, 8080);
        assert_eq!(loaded.log_level, "DEBUG");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_settings_valid() -> Result<()> {
        let (manager, _, temp_dir) = create_test_manager().await?;
        
        let mut settings = Settings::default();
        settings.library_path = temp_dir.path().to_string_lossy().to_string();
        settings.download_path = temp_dir.path().to_string_lossy().to_string();
        
        assert!(manager.validate_settings(&settings).is_ok());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_settings_invalid_port() -> Result<()> {
        let (manager, _, temp_dir) = create_test_manager().await?;
        
        let mut settings = Settings::default();
        settings.library_path = temp_dir.path().to_string_lossy().to_string();
        settings.download_path = temp_dir.path().to_string_lossy().to_string();
        settings.qbittorrent_port = 0; // Invalid port
        
        assert!(manager.validate_settings(&settings).is_err());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_settings_invalid_path() -> Result<()> {
        let (manager, _, _) = create_test_manager().await?;
        
        let mut settings = Settings::default();
        settings.library_path = "/nonexistent/path".to_string();
        
        assert!(manager.validate_settings(&settings).is_err());
        
        Ok(())
    }

    // Test 15.1: Verify settings survive application close/reopen
    #[tokio::test]
    async fn test_settings_persist_across_restarts() -> Result<()> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_persistence.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        // First session: Create manager, save settings, drop manager
        {
            let db = Database::new(&db_url).await?;
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await?;
            
            let manager = SettingsManager::new(settings_db.clone());
            
            let mut settings = Settings::default();
            settings.qbittorrent_host = "192.168.1.200".to_string();
            settings.qbittorrent_port = 9090;
            settings.qbittorrent_username = "testuser".to_string();
            settings.qbittorrent_password = "testpass".to_string();
            settings.log_level = "DEBUG".to_string();
            settings.search_limit = 500;
            settings.max_concurrent_downloads = 5;
            settings.library_path = temp_dir.path().to_string_lossy().to_string();
            settings.download_path = temp_dir.path().to_string_lossy().to_string();
            settings.tmdb_api_key = Some("test_api_key_12345".to_string());
            settings.kinopoisk_api_token = Some("test_kp_token_67890".to_string());
            
            manager.save_settings(&settings).await?;
            
            // Manager and database connections are dropped here
        }
        
        // Second session: Create new manager, load settings, verify they match
        {
            let db = Database::new(&db_url).await?;
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await?;
            
            let manager = SettingsManager::new(settings_db.clone());
            
            let loaded = manager.load_settings().await?;
            
            // Verify all settings were persisted correctly
            assert_eq!(loaded.qbittorrent_host, "192.168.1.200");
            assert_eq!(loaded.qbittorrent_port, 9090);
            assert_eq!(loaded.qbittorrent_username, "testuser");
            assert_eq!(loaded.qbittorrent_password, "testpass");
            assert_eq!(loaded.log_level, "DEBUG");
            assert_eq!(loaded.search_limit, 500);
            assert_eq!(loaded.max_concurrent_downloads, 5);
            assert_eq!(loaded.tmdb_api_key, Some("test_api_key_12345".to_string()));
            assert_eq!(loaded.kinopoisk_api_token, Some("test_kp_token_67890".to_string()));
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_settings_persist_optional_fields() -> Result<()> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_optional.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        // First session: Save settings with optional fields
        {
            let db = Database::new(&db_url).await?;
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await?;
            
            let manager = SettingsManager::new(settings_db.clone());
            
            let mut settings = Settings::default();
            settings.library_path = temp_dir.path().to_string_lossy().to_string();
            settings.download_path = temp_dir.path().to_string_lossy().to_string();
            settings.qbittorrent_category = Some("movies".to_string());
            settings.qbittorrent_tags = Some("torrentflix,auto".to_string());
            settings.telegram_enabled = true;
            settings.telegram_bot_token = Some("bot123456:ABC-DEF".to_string());
            settings.telegram_chat_id = Some("987654321".to_string());
            
            manager.save_settings(&settings).await?;
        }
        
        // Second session: Load and verify optional fields
        {
            let db = Database::new(&db_url).await?;
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await?;
            
            let manager = SettingsManager::new(settings_db.clone());
            let loaded = manager.load_settings().await?;
            
            assert_eq!(loaded.qbittorrent_category, Some("movies".to_string()));
            assert_eq!(loaded.qbittorrent_tags, Some("torrentflix,auto".to_string()));
            assert_eq!(loaded.telegram_enabled, true);
            assert_eq!(loaded.telegram_bot_token, Some("bot123456:ABC-DEF".to_string()));
            assert_eq!(loaded.telegram_chat_id, Some("987654321".to_string()));
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_settings_persist_empty_optional_fields() -> Result<()> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_empty_optional.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        // First session: Save settings with None optional fields
        {
            let db = Database::new(&db_url).await?;
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await?;
            
            let manager = SettingsManager::new(settings_db.clone());
            
            let mut settings = Settings::default();
            settings.library_path = temp_dir.path().to_string_lossy().to_string();
            settings.download_path = temp_dir.path().to_string_lossy().to_string();
            settings.tmdb_api_key = None;
            settings.kinopoisk_api_token = None;
            settings.qbittorrent_category = None;
            
            manager.save_settings(&settings).await?;
        }
        
        // Second session: Load and verify None fields remain None
        {
            let db = Database::new(&db_url).await?;
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await?;
            
            let manager = SettingsManager::new(settings_db.clone());
            let loaded = manager.load_settings().await?;
            
            assert_eq!(loaded.tmdb_api_key, None);
            assert_eq!(loaded.kinopoisk_api_token, None);
            assert_eq!(loaded.qbittorrent_category, None);
        }
        
        Ok(())
    }

    // Test 15.2: Test with various setting combinations
    #[tokio::test]
    async fn test_settings_persist_all_enabled_features() -> Result<()> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_all_enabled.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        // First session: Save settings with all features enabled
        {
            let db = Database::new(&db_url).await?;
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await?;
            
            let manager = SettingsManager::new(settings_db.clone());
            
            let mut settings = Settings::default();
            settings.library_path = temp_dir.path().to_string_lossy().to_string();
            settings.download_path = temp_dir.path().to_string_lossy().to_string();
            
            // Enable all features
            settings.qbittorrent_enabled = true;
            settings.qbittorrent_use_ssl = true;
            settings.telegram_enabled = true;
            settings.telegram_bot_token = Some("bot_token_123".to_string());
            settings.telegram_chat_id = Some("chat_id_456".to_string());
            settings.email_enabled = true;
            settings.smtp_user = Some("user@example.com".to_string());
            settings.smtp_password = Some("password123".to_string());
            settings.from_email = Some("from@example.com".to_string());
            settings.to_email = Some("to@example.com".to_string());
            settings.proxy_enabled = true;
            settings.debug = true;
            
            manager.save_settings(&settings).await?;
        }
        
        // Second session: Load and verify all features remain enabled
        {
            let db = Database::new(&db_url).await?;
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await?;
            
            let manager = SettingsManager::new(settings_db.clone());
            let loaded = manager.load_settings().await?;
            
            assert_eq!(loaded.qbittorrent_enabled, true);
            assert_eq!(loaded.qbittorrent_use_ssl, true);
            assert_eq!(loaded.telegram_enabled, true);
            assert_eq!(loaded.telegram_bot_token, Some("bot_token_123".to_string()));
            assert_eq!(loaded.telegram_chat_id, Some("chat_id_456".to_string()));
            assert_eq!(loaded.email_enabled, true);
            assert_eq!(loaded.smtp_user, Some("user@example.com".to_string()));
            assert_eq!(loaded.smtp_password, Some("password123".to_string()));
            assert_eq!(loaded.from_email, Some("from@example.com".to_string()));
            assert_eq!(loaded.to_email, Some("to@example.com".to_string()));
            assert_eq!(loaded.proxy_enabled, true);
            assert_eq!(loaded.debug, true);
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_settings_persist_all_disabled_features() -> Result<()> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_all_disabled.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        // First session: Save settings with all features disabled
        {
            let db = Database::new(&db_url).await?;
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await?;
            
            let manager = SettingsManager::new(settings_db.clone());
            
            let mut settings = Settings::default();
            settings.library_path = temp_dir.path().to_string_lossy().to_string();
            settings.download_path = temp_dir.path().to_string_lossy().to_string();
            
            // Disable all features
            settings.qbittorrent_enabled = false;
            settings.telegram_enabled = false;
            settings.email_enabled = false;
            settings.proxy_enabled = false;
            settings.debug = false;
            
            manager.save_settings(&settings).await?;
        }
        
        // Second session: Load and verify all features remain disabled
        {
            let db = Database::new(&db_url).await?;
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await?;
            
            let manager = SettingsManager::new(settings_db.clone());
            let loaded = manager.load_settings().await?;
            
            assert_eq!(loaded.qbittorrent_enabled, false);
            assert_eq!(loaded.telegram_enabled, false);
            assert_eq!(loaded.email_enabled, false);
            assert_eq!(loaded.proxy_enabled, false);
            assert_eq!(loaded.debug, false);
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_settings_persist_extreme_numeric_values() -> Result<()> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_extreme_values.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        // First session: Save settings with extreme but valid numeric values
        {
            let db = Database::new(&db_url).await?;
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await?;
            
            let manager = SettingsManager::new(settings_db.clone());
            
            let mut settings = Settings::default();
            settings.library_path = temp_dir.path().to_string_lossy().to_string();
            settings.download_path = temp_dir.path().to_string_lossy().to_string();
            
            // Set extreme but valid values
            settings.qbittorrent_port = 65535; // Max port
            settings.qbittorrent_timeout = 300; // Max timeout
            settings.search_limit = 10000; // Max search limit
            settings.search_timeout = 300; // Max search timeout
            settings.max_concurrent_downloads = 100; // Max concurrent downloads
            settings.telegram_debounce = 3600; // Max debounce
            settings.smtp_port = 65535; // Max port
            settings.proxy_port = 65535; // Max port
            settings.db_pool_size = 100; // Max pool size
            settings.db_max_overflow = 100; // Max overflow
            
            manager.save_settings(&settings).await?;
        }
        
        // Second session: Load and verify extreme values persist
        {
            let db = Database::new(&db_url).await?;
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await?;
            
            let manager = SettingsManager::new(settings_db.clone());
            let loaded = manager.load_settings().await?;
            
            assert_eq!(loaded.qbittorrent_port, 65535);
            assert_eq!(loaded.qbittorrent_timeout, 300);
            assert_eq!(loaded.search_limit, 10000);
            assert_eq!(loaded.search_timeout, 300);
            assert_eq!(loaded.max_concurrent_downloads, 100);
            assert_eq!(loaded.telegram_debounce, 3600);
            assert_eq!(loaded.smtp_port, 65535);
            assert_eq!(loaded.proxy_port, 65535);
            assert_eq!(loaded.db_pool_size, 100);
            assert_eq!(loaded.db_max_overflow, 100);
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_settings_persist_minimum_numeric_values() -> Result<()> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_minimum_values.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        // First session: Save settings with minimum valid numeric values
        {
            let db = Database::new(&db_url).await?;
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await?;
            
            let manager = SettingsManager::new(settings_db.clone());
            
            let mut settings = Settings::default();
            settings.library_path = temp_dir.path().to_string_lossy().to_string();
            settings.download_path = temp_dir.path().to_string_lossy().to_string();
            
            // Set minimum valid values
            settings.qbittorrent_port = 1; // Min port
            settings.qbittorrent_timeout = 1; // Min timeout
            settings.search_limit = 1; // Min search limit
            settings.search_timeout = 1; // Min search timeout
            settings.max_concurrent_downloads = 1; // Min concurrent downloads
            settings.telegram_debounce = 0; // Min debounce
            settings.smtp_port = 1; // Min port
            settings.proxy_port = 1; // Min port
            settings.db_pool_size = 1; // Min pool size
            settings.db_max_overflow = 0; // Min overflow
            
            manager.save_settings(&settings).await?;
        }
        
        // Second session: Load and verify minimum values persist
        {
            let db = Database::new(&db_url).await?;
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await?;
            
            let manager = SettingsManager::new(settings_db.clone());
            let loaded = manager.load_settings().await?;
            
            assert_eq!(loaded.qbittorrent_port, 1);
            assert_eq!(loaded.qbittorrent_timeout, 1);
            assert_eq!(loaded.search_limit, 1);
            assert_eq!(loaded.search_timeout, 1);
            assert_eq!(loaded.max_concurrent_downloads, 1);
            assert_eq!(loaded.telegram_debounce, 0);
            assert_eq!(loaded.smtp_port, 1);
            assert_eq!(loaded.proxy_port, 1);
            assert_eq!(loaded.db_pool_size, 1);
            assert_eq!(loaded.db_max_overflow, 0);
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_settings_persist_mixed_configuration() -> Result<()> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_mixed_config.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        // First session: Save settings with mixed configuration
        {
            let db = Database::new(&db_url).await?;
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await?;
            
            let manager = SettingsManager::new(settings_db.clone());
            
            let mut settings = Settings::default();
            settings.library_path = temp_dir.path().to_string_lossy().to_string();
            settings.download_path = temp_dir.path().to_string_lossy().to_string();
            
            // Mix of enabled/disabled features and various values
            settings.qbittorrent_enabled = true;
            settings.qbittorrent_host = "custom.host.com".to_string();
            settings.qbittorrent_port = 12345;
            settings.qbittorrent_use_ssl = true;
            settings.telegram_enabled = false; // Disabled
            settings.email_enabled = true;
            settings.smtp_host = "smtp.custom.com".to_string();
            settings.smtp_port = 2525;
            settings.smtp_user = Some("custom@user.com".to_string());
            settings.smtp_password = Some("secure_pass_123".to_string());
            settings.from_email = Some("noreply@custom.com".to_string());
            settings.to_email = Some("admin@custom.com".to_string());
            settings.proxy_enabled = false; // Disabled
            settings.log_level = "WARN".to_string();
            settings.log_format = "text".to_string();
            settings.debug = true;
            settings.search_limit = 250;
            settings.enabled_indexers = "monna,custom".to_string();
            
            manager.save_settings(&settings).await?;
        }
        
        // Second session: Load and verify mixed configuration persists
        {
            let db = Database::new(&db_url).await?;
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await?;
            
            let manager = SettingsManager::new(settings_db.clone());
            let loaded = manager.load_settings().await?;
            
            assert_eq!(loaded.qbittorrent_enabled, true);
            assert_eq!(loaded.qbittorrent_host, "custom.host.com");
            assert_eq!(loaded.qbittorrent_port, 12345);
            assert_eq!(loaded.qbittorrent_use_ssl, true);
            assert_eq!(loaded.telegram_enabled, false);
            assert_eq!(loaded.email_enabled, true);
            assert_eq!(loaded.smtp_host, "smtp.custom.com");
            assert_eq!(loaded.smtp_port, 2525);
            assert_eq!(loaded.smtp_user, Some("custom@user.com".to_string()));
            assert_eq!(loaded.smtp_password, Some("secure_pass_123".to_string()));
            assert_eq!(loaded.from_email, Some("noreply@custom.com".to_string()));
            assert_eq!(loaded.to_email, Some("admin@custom.com".to_string()));
            assert_eq!(loaded.proxy_enabled, false);
            assert_eq!(loaded.log_level, "WARN");
            assert_eq!(loaded.log_format, "text");
            assert_eq!(loaded.debug, true);
            assert_eq!(loaded.search_limit, 250);
            assert_eq!(loaded.enabled_indexers, "monna,custom");
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_settings_persist_multiple_updates() -> Result<()> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_multiple_updates.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        let db = Database::new(&db_url).await?;
        let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
        settings_db.initialize().await?;
        
        let manager = SettingsManager::new(settings_db.clone());
        
        // First update
        {
            let mut settings = Settings::default();
            settings.library_path = temp_dir.path().to_string_lossy().to_string();
            settings.download_path = temp_dir.path().to_string_lossy().to_string();
            settings.qbittorrent_host = "host1".to_string();
            settings.log_level = "INFO".to_string();
            
            manager.save_settings(&settings).await?;
        }
        
        // Second update - change some values
        {
            let mut settings = manager.load_settings().await?;
            settings.qbittorrent_host = "host2".to_string();
            settings.log_level = "DEBUG".to_string();
            settings.search_limit = 500;
            
            manager.save_settings(&settings).await?;
        }
        
        // Third update - change more values
        {
            let mut settings = manager.load_settings().await?;
            settings.qbittorrent_host = "host3".to_string();
            settings.qbittorrent_port = 9999;
            settings.max_concurrent_downloads = 10;
            
            manager.save_settings(&settings).await?;
        }
        
        // Final load - verify latest values
        {
            let loaded = manager.load_settings().await?;
            
            assert_eq!(loaded.qbittorrent_host, "host3");
            assert_eq!(loaded.qbittorrent_port, 9999);
            assert_eq!(loaded.log_level, "DEBUG");
            assert_eq!(loaded.search_limit, 500);
            assert_eq!(loaded.max_concurrent_downloads, 10);
        }
        
        Ok(())
    }

    // Test 15.3: Test with corrupted database entries
    #[tokio::test]
    async fn test_settings_handle_corrupted_numeric_values() -> Result<()> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_corrupted_numeric.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        let db = Database::new(&db_url).await?;
        let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
        settings_db.initialize().await?;
        
        // Manually insert corrupted numeric values
        settings_db.set("qbittorrent_port", "not_a_number", "int").await?;
        settings_db.set("search_limit", "invalid", "int").await?;
        settings_db.set("max_concurrent_downloads", "xyz", "int").await?;
        settings_db.set("qbittorrent_enabled", "maybe", "bool").await?;
        
        let manager = SettingsManager::new(settings_db.clone());
        
        // Load settings - should use defaults for corrupted values
        let loaded = manager.load_settings().await?;
        
        // Verify defaults are used for corrupted values
        assert_eq!(loaded.qbittorrent_port, 5555); // Default
        assert_eq!(loaded.search_limit, 100); // Default
        assert_eq!(loaded.max_concurrent_downloads, 3); // Default
        assert_eq!(loaded.qbittorrent_enabled, true); // Default
        
        Ok(())
    }

    #[tokio::test]
    async fn test_settings_handle_missing_required_fields() -> Result<()> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_missing_fields.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        let db = Database::new(&db_url).await?;
        let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
        settings_db.initialize().await?;
        
        // Only set a few fields, leaving most missing
        settings_db.set("qbittorrent_host", "custom.host", "string").await?;
        settings_db.set("log_level", "WARN", "string").await?;
        
        let manager = SettingsManager::new(settings_db.clone());
        
        // Load settings - should use defaults for missing values
        let loaded = manager.load_settings().await?;
        
        // Verify custom values are loaded
        assert_eq!(loaded.qbittorrent_host, "custom.host");
        assert_eq!(loaded.log_level, "WARN");
        
        // Verify defaults are used for missing values
        assert_eq!(loaded.qbittorrent_port, 5555);
        assert_eq!(loaded.qbittorrent_username, "admin");
        assert_eq!(loaded.search_limit, 100);
        assert_eq!(loaded.max_concurrent_downloads, 3);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_settings_handle_empty_database() -> Result<()> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_empty_db.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        let db = Database::new(&db_url).await?;
        let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
        settings_db.initialize().await?;
        
        let manager = SettingsManager::new(settings_db.clone());
        
        // Load settings from empty database - should return all defaults
        let loaded = manager.load_settings().await?;
        let defaults = Settings::default();
        
        // Verify all values match defaults
        assert_eq!(loaded.qbittorrent_enabled, defaults.qbittorrent_enabled);
        assert_eq!(loaded.qbittorrent_host, defaults.qbittorrent_host);
        assert_eq!(loaded.qbittorrent_port, defaults.qbittorrent_port);
        assert_eq!(loaded.qbittorrent_username, defaults.qbittorrent_username);
        assert_eq!(loaded.log_level, defaults.log_level);
        assert_eq!(loaded.search_limit, defaults.search_limit);
        assert_eq!(loaded.max_concurrent_downloads, defaults.max_concurrent_downloads);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_settings_handle_corrupted_boolean_values() -> Result<()> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_corrupted_bool.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        let db = Database::new(&db_url).await?;
        let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
        settings_db.initialize().await?;
        
        // Manually insert corrupted boolean values
        settings_db.set("qbittorrent_enabled", "yes", "bool").await?;
        settings_db.set("telegram_enabled", "1", "bool").await?;
        settings_db.set("email_enabled", "on", "bool").await?;
        settings_db.set("proxy_enabled", "enabled", "bool").await?;
        settings_db.set("debug", "active", "bool").await?;
        
        let manager = SettingsManager::new(settings_db.clone());
        
        // Load settings - should use defaults for corrupted boolean values
        let loaded = manager.load_settings().await?;
        
        // Verify defaults are used for corrupted boolean values
        assert_eq!(loaded.qbittorrent_enabled, true); // Default
        assert_eq!(loaded.telegram_enabled, false); // Default
        assert_eq!(loaded.email_enabled, false); // Default
        assert_eq!(loaded.proxy_enabled, false); // Default
        assert_eq!(loaded.debug, false); // Default
        
        Ok(())
    }

    #[tokio::test]
    async fn test_settings_handle_out_of_range_values() -> Result<()> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_out_of_range.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        let db = Database::new(&db_url).await?;
        let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
        settings_db.initialize().await?;
        
        // Manually insert out-of-range values (these will parse but fail validation)
        settings_db.set("qbittorrent_port", "99999", "int").await?; // > 65535, will wrap to 34463
        settings_db.set("search_limit", "0", "int").await?; // < 1
        settings_db.set("max_concurrent_downloads", "1000", "int").await?; // > 100
        settings_db.set("library_path", temp_dir.path().to_string_lossy().as_ref(), "string").await?;
        settings_db.set("download_path", temp_dir.path().to_string_lossy().as_ref(), "string").await?;
        
        let manager = SettingsManager::new(settings_db.clone());
        
        // Load settings - values will be loaded (parsing succeeded, but may wrap)
        let loaded = manager.load_settings().await?;
        
        // Port wraps around due to u16 overflow (99999 % 65536 = 34463)
        // This demonstrates that corrupted numeric values can still be loaded
        // but will fail validation when attempting to save
        assert_eq!(loaded.search_limit, 0);
        assert_eq!(loaded.max_concurrent_downloads, 1000);
        
        // Attempting to save these settings should fail validation
        let save_result = manager.save_settings(&loaded).await;
        assert!(save_result.is_err(), "Should fail validation for out-of-range values");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_settings_recover_from_partial_corruption() -> Result<()> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_partial_corruption.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        let db = Database::new(&db_url).await?;
        let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
        settings_db.initialize().await?;
        
        // Insert mix of valid and corrupted values
        settings_db.set("qbittorrent_host", "valid.host.com", "string").await?;
        settings_db.set("qbittorrent_port", "not_a_port", "int").await?; // Corrupted
        settings_db.set("log_level", "DEBUG", "string").await?;
        settings_db.set("search_limit", "invalid_number", "int").await?; // Corrupted
        settings_db.set("max_concurrent_downloads", "5", "int").await?;
        settings_db.set("qbittorrent_enabled", "maybe", "bool").await?; // Corrupted
        
        let manager = SettingsManager::new(settings_db.clone());
        
        // Load settings - should use valid values and defaults for corrupted ones
        let loaded = manager.load_settings().await?;
        
        // Valid values should be loaded
        assert_eq!(loaded.qbittorrent_host, "valid.host.com");
        assert_eq!(loaded.log_level, "DEBUG");
        assert_eq!(loaded.max_concurrent_downloads, 5);
        
        // Corrupted values should use defaults
        assert_eq!(loaded.qbittorrent_port, 5555); // Default
        assert_eq!(loaded.search_limit, 100); // Default
        assert_eq!(loaded.qbittorrent_enabled, true); // Default
        
        Ok(())
    }

    #[tokio::test]
    async fn test_settings_handle_unknown_keys() -> Result<()> {
        use crate::database::Database;
        
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_unknown_keys.db");
        let db_url = db_path.to_string_lossy().to_string();
        
        let db = Database::new(&db_url).await?;
        let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
        settings_db.initialize().await?;
        
        // Insert some valid settings
        settings_db.set("qbittorrent_host", "test.host", "string").await?;
        settings_db.set("log_level", "INFO", "string").await?;
        
        // Insert unknown keys (from future versions or typos)
        settings_db.set("unknown_setting_1", "value1", "string").await?;
        settings_db.set("future_feature_enabled", "true", "bool").await?;
        settings_db.set("deprecated_setting", "old_value", "string").await?;
        
        let manager = SettingsManager::new(settings_db.clone());
        
        // Load settings - should ignore unknown keys and load valid ones
        let loaded = manager.load_settings().await?;
        
        // Valid settings should be loaded
        assert_eq!(loaded.qbittorrent_host, "test.host");
        assert_eq!(loaded.log_level, "INFO");
        
        // Unknown keys should be ignored (logged as warnings)
        // The application should continue functioning normally
        
        Ok(())
    }
}
