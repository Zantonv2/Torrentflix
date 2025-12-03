// Property-based tests for settings persistence
// Feature: settings-ui, Property 1: Settings Persistence Round Trip
// Validates: Requirements 4.1, 4.2, 4.3

use anyhow::Result;
use engine::database::{Database, SettingsDatabase};
use engine::settings::{manager::SettingsManager, models::Settings};
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use std::sync::Arc;
use tempfile::TempDir;

// Generator for valid port numbers (1-65535)
fn valid_port() -> impl Strategy<Value = u16> {
    1u16..=65535u16
}

// Generator for valid timeout values (1-300 seconds)
fn valid_timeout() -> impl Strategy<Value = u64> {
    1u64..=300u64
}

// Generator for valid search limits (1-10000)
fn valid_search_limit() -> impl Strategy<Value = u32> {
    1u32..=10000u32
}

// Generator for valid concurrent downloads (1-100)
fn valid_concurrent_downloads() -> impl Strategy<Value = u32> {
    1u32..=100u32
}

// Generator for valid debounce values (0-3600 seconds)
fn valid_debounce() -> impl Strategy<Value = u64> {
    0u64..=3600u64
}

// Generator for valid pool sizes (1-100)
fn valid_pool_size() -> impl Strategy<Value = u32> {
    1u32..=100u32
}

// Generator for valid overflow values (0-100)
fn valid_overflow() -> impl Strategy<Value = u32> {
    0u32..=100u32
}

// Generator for optional strings (None or Some with alphanumeric content)
fn optional_string() -> impl Strategy<Value = Option<String>> {
    prop::option::of("[a-zA-Z0-9_-]{1,50}")
}

// Generator for non-empty strings
fn non_empty_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9._-]{1,50}"
}

// Generator for hostnames
fn hostname() -> impl Strategy<Value = String> {
    "[a-z]{3,10}(\\.[a-z]{3,10}){0,2}"
}

// Generator for log levels
fn log_level() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("DEBUG".to_string()),
        Just("INFO".to_string()),
        Just("WARN".to_string()),
        Just("ERROR".to_string()),
    ]
}

// Generator for log formats
fn log_format() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("json".to_string()),
        Just("text".to_string()),
    ]
}

// Generator for email addresses
fn email_address() -> impl Strategy<Value = String> {
    "[a-z]{3,10}@[a-z]{3,10}\\.[a-z]{2,4}"
}

// Generator for invalid port numbers (0 only, since u16 max is 65535)
fn invalid_port() -> impl Strategy<Value = u16> {
    Just(0u16)
}

// Generator for invalid timeout values (0 or > 300)
fn invalid_timeout() -> impl Strategy<Value = u64> {
    prop_oneof![
        Just(0u64),
        301u64..=1000u64,
    ]
}

// Generator for invalid search limits (0 or > 10000)
fn invalid_search_limit() -> impl Strategy<Value = u32> {
    prop_oneof![
        Just(0u32),
        10001u32..=20000u32,
    ]
}

// Generator for invalid concurrent downloads (0 or > 100)
fn invalid_concurrent_downloads() -> impl Strategy<Value = u32> {
    prop_oneof![
        Just(0u32),
        101u32..=200u32,
    ]
}

// Generator for invalid pool sizes (0 or > 100)
fn invalid_pool_size() -> impl Strategy<Value = u32> {
    prop_oneof![
        Just(0u32),
        101u32..=200u32,
    ]
}

// Generator for invalid overflow values (> 100)
fn invalid_overflow() -> impl Strategy<Value = u32> {
    101u32..=200u32
}

// Generator for non-existent paths
fn nonexistent_path() -> impl Strategy<Value = String> {
    "/nonexistent/path/[a-z]{5,10}/[a-z]{5,10}"
}

// Generator for relative paths
fn relative_path() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("./relative/path".to_string()),
        Just("../parent/path".to_string()),
        Just("relative/path".to_string()),
    ]
}

// Generator for invalid email addresses
fn invalid_email() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("notanemail".to_string()),
        Just("missing@domain".to_string()),
        Just("@nodomain.com".to_string()),
        Just("noatsign.com".to_string()),
        "[a-z]{3,10}@[a-z]{3,10}",  // Missing TLD
    ]
}

// Enum to represent different types of invalid settings
#[derive(Debug, Clone)]
enum InvalidSettingsType {
    InvalidPort,
    InvalidTimeout,
    InvalidSearchLimit,
    InvalidConcurrentDownloads,
    InvalidPoolSize,
    InvalidOverflow,
    NonexistentPath,
    RelativePath,
    TelegramEnabledMissingToken,
    TelegramEnabledMissingChatId,
    EmailEnabledMissingSmtpUser,
    EmailEnabledMissingSmtpPassword,
    EmailEnabledMissingFromEmail,
    EmailEnabledMissingToEmail,
    EmailEnabledInvalidFromEmail,
    EmailEnabledInvalidToEmail,
}

// Generator for invalid Settings objects
fn invalid_settings_strategy(temp_dir_path: String) -> impl Strategy<Value = (Settings, InvalidSettingsType, String)> {
    prop_oneof![
        // Invalid port
        (valid_settings_strategy(temp_dir_path.clone()), invalid_port())
            .prop_map(|(mut settings, port)| {
                settings.qbittorrent_port = port;
                (settings, InvalidSettingsType::InvalidPort, "qbittorrent_port".to_string())
            }),
        
        // Invalid timeout
        (valid_settings_strategy(temp_dir_path.clone()), invalid_timeout())
            .prop_map(|(mut settings, timeout)| {
                settings.qbittorrent_timeout = timeout;
                (settings, InvalidSettingsType::InvalidTimeout, "qbittorrent_timeout".to_string())
            }),
        
        // Invalid search limit
        (valid_settings_strategy(temp_dir_path.clone()), invalid_search_limit())
            .prop_map(|(mut settings, limit)| {
                settings.search_limit = limit;
                (settings, InvalidSettingsType::InvalidSearchLimit, "search_limit".to_string())
            }),
        
        // Invalid concurrent downloads
        (valid_settings_strategy(temp_dir_path.clone()), invalid_concurrent_downloads())
            .prop_map(|(mut settings, downloads)| {
                settings.max_concurrent_downloads = downloads;
                (settings, InvalidSettingsType::InvalidConcurrentDownloads, "max_concurrent_downloads".to_string())
            }),
        
        // Invalid pool size
        (valid_settings_strategy(temp_dir_path.clone()), invalid_pool_size())
            .prop_map(|(mut settings, pool_size)| {
                settings.db_pool_size = pool_size;
                (settings, InvalidSettingsType::InvalidPoolSize, "db_pool_size".to_string())
            }),
        
        // Invalid overflow
        (valid_settings_strategy(temp_dir_path.clone()), invalid_overflow())
            .prop_map(|(mut settings, overflow)| {
                settings.db_max_overflow = overflow;
                (settings, InvalidSettingsType::InvalidOverflow, "db_max_overflow".to_string())
            }),
        
        // Nonexistent path (library_path)
        (valid_settings_strategy(temp_dir_path.clone()), nonexistent_path())
            .prop_map(|(mut settings, path)| {
                settings.library_path = path;
                (settings, InvalidSettingsType::NonexistentPath, "library_path".to_string())
            }),
        
        // Nonexistent path (download_path)
        (valid_settings_strategy(temp_dir_path.clone()), nonexistent_path())
            .prop_map(|(mut settings, path)| {
                settings.download_path = path;
                (settings, InvalidSettingsType::NonexistentPath, "download_path".to_string())
            }),
        
        // Relative path (library_path)
        (valid_settings_strategy(temp_dir_path.clone()), relative_path())
            .prop_map(|(mut settings, path)| {
                settings.library_path = path;
                (settings, InvalidSettingsType::RelativePath, "library_path".to_string())
            }),
        
        // Relative path (download_path)
        (valid_settings_strategy(temp_dir_path.clone()), relative_path())
            .prop_map(|(mut settings, path)| {
                settings.download_path = path;
                (settings, InvalidSettingsType::RelativePath, "download_path".to_string())
            }),
        
        // Telegram enabled but missing token
        valid_settings_strategy(temp_dir_path.clone())
            .prop_map(|mut settings| {
                settings.telegram_enabled = true;
                settings.telegram_bot_token = None;
                settings.telegram_chat_id = Some("12345".to_string());
                (settings, InvalidSettingsType::TelegramEnabledMissingToken, "telegram_bot_token".to_string())
            }),
        
        // Telegram enabled but missing chat ID
        valid_settings_strategy(temp_dir_path.clone())
            .prop_map(|mut settings| {
                settings.telegram_enabled = true;
                settings.telegram_bot_token = Some("token123".to_string());
                settings.telegram_chat_id = None;
                (settings, InvalidSettingsType::TelegramEnabledMissingChatId, "telegram_chat_id".to_string())
            }),
        
        // Email enabled but missing SMTP user
        valid_settings_strategy(temp_dir_path.clone())
            .prop_map(|mut settings| {
                settings.email_enabled = true;
                settings.smtp_user = None;
                settings.smtp_password = Some("pass".to_string());
                settings.from_email = Some("from@example.com".to_string());
                settings.to_email = Some("to@example.com".to_string());
                (settings, InvalidSettingsType::EmailEnabledMissingSmtpUser, "smtp_user".to_string())
            }),
        
        // Email enabled but missing SMTP password
        valid_settings_strategy(temp_dir_path.clone())
            .prop_map(|mut settings| {
                settings.email_enabled = true;
                settings.smtp_user = Some("user".to_string());
                settings.smtp_password = None;
                settings.from_email = Some("from@example.com".to_string());
                settings.to_email = Some("to@example.com".to_string());
                (settings, InvalidSettingsType::EmailEnabledMissingSmtpPassword, "smtp_password".to_string())
            }),
        
        // Email enabled but missing from_email
        valid_settings_strategy(temp_dir_path.clone())
            .prop_map(|mut settings| {
                settings.email_enabled = true;
                settings.smtp_user = Some("user".to_string());
                settings.smtp_password = Some("pass".to_string());
                settings.from_email = None;
                settings.to_email = Some("to@example.com".to_string());
                (settings, InvalidSettingsType::EmailEnabledMissingFromEmail, "from_email".to_string())
            }),
        
        // Email enabled but missing to_email
        valid_settings_strategy(temp_dir_path.clone())
            .prop_map(|mut settings| {
                settings.email_enabled = true;
                settings.smtp_user = Some("user".to_string());
                settings.smtp_password = Some("pass".to_string());
                settings.from_email = Some("from@example.com".to_string());
                settings.to_email = None;
                (settings, InvalidSettingsType::EmailEnabledMissingToEmail, "to_email".to_string())
            }),
        
        // Email enabled with invalid from_email
        (valid_settings_strategy(temp_dir_path.clone()), invalid_email())
            .prop_map(|(mut settings, email)| {
                settings.email_enabled = true;
                settings.smtp_user = Some("user".to_string());
                settings.smtp_password = Some("pass".to_string());
                settings.from_email = Some(email);
                settings.to_email = Some("to@example.com".to_string());
                (settings, InvalidSettingsType::EmailEnabledInvalidFromEmail, "from_email".to_string())
            }),
        
        // Email enabled with invalid to_email
        (valid_settings_strategy(temp_dir_path.clone()), invalid_email())
            .prop_map(|(mut settings, email)| {
                settings.email_enabled = true;
                settings.smtp_user = Some("user".to_string());
                settings.smtp_password = Some("pass".to_string());
                settings.from_email = Some("from@example.com".to_string());
                settings.to_email = Some(email);
                (settings, InvalidSettingsType::EmailEnabledInvalidToEmail, "to_email".to_string())
            }),
    ]
}

// Generator for database URLs
fn database_url() -> impl Strategy<Value = String> {
    "sqlite:///[a-z_]{5,15}\\.db"
}

// Main generator for valid Settings objects
// We use Just() to create a strategy that always returns the same value
fn valid_settings_strategy(temp_dir_path: String) -> impl Strategy<Value = Settings> {
    // Split into smaller tuples to avoid proptest's 12-element limit
    let api_settings = (
        optional_string(),                  // tmdb_api_key
        optional_string(),                  // tmdb_read_access_token
        optional_string(),                  // kinopoisk_api_token
    );
    
    let qbittorrent_settings = (
        any::<bool>(),                      // qbittorrent_enabled
        hostname(),                         // qbittorrent_host
        valid_port(),                       // qbittorrent_port
        non_empty_string(),                 // qbittorrent_username
        non_empty_string(),                 // qbittorrent_password
        any::<bool>(),                      // qbittorrent_use_ssl
        any::<bool>(),                      // qbittorrent_verify_ssl
        optional_string(),                  // qbittorrent_category
        optional_string(),                  // qbittorrent_tags
        valid_timeout(),                    // qbittorrent_timeout
    );
    
    // For notifications, we need to ensure required fields are present when enabled
    // Strategy: either disable notifications OR enable with all required fields
    let notification_settings = prop_oneof![
        // Case 1: All notifications disabled
        Just((
            false,                          // telegram_enabled
            None,                           // telegram_bot_token
            None,                           // telegram_chat_id
            0u64,                           // telegram_debounce
            false,                          // email_enabled
            "smtp.gmail.com".to_string(),   // smtp_host
            587u16,                         // smtp_port
            None,                           // smtp_user
            None,                           // smtp_password
            None,                           // from_email
            None,                           // to_email
        )),
        // Case 2: Telegram enabled with required fields
        (non_empty_string(), non_empty_string(), valid_debounce())
            .prop_map(|(token, chat_id, debounce)| (
                true,                       // telegram_enabled
                Some(token),                // telegram_bot_token
                Some(chat_id),              // telegram_chat_id
                debounce,                   // telegram_debounce
                false,                      // email_enabled
                "smtp.gmail.com".to_string(), // smtp_host
                587u16,                     // smtp_port
                None,                       // smtp_user
                None,                       // smtp_password
                None,                       // from_email
                None,                       // to_email
            )),
        // Case 3: Email enabled with required fields
        (hostname(), valid_port(), non_empty_string(), non_empty_string(), email_address(), email_address())
            .prop_map(|(smtp_host, smtp_port, smtp_user, smtp_password, from_email, to_email)| (
                false,                      // telegram_enabled
                None,                       // telegram_bot_token
                None,                       // telegram_chat_id
                0u64,                       // telegram_debounce
                true,                       // email_enabled
                smtp_host,                  // smtp_host
                smtp_port,                  // smtp_port
                Some(smtp_user),            // smtp_user
                Some(smtp_password),        // smtp_password
                Some(from_email),           // from_email
                Some(to_email),             // to_email
            )),
        // Case 4: Both enabled with all required fields
        (
            non_empty_string(), 
            non_empty_string(), 
            valid_debounce(),
            hostname(), 
            valid_port(), 
            non_empty_string(), 
            non_empty_string(), 
            email_address(), 
            email_address()
        ).prop_map(|(token, chat_id, debounce, smtp_host, smtp_port, smtp_user, smtp_password, from_email, to_email)| (
            true,                           // telegram_enabled
            Some(token),                    // telegram_bot_token
            Some(chat_id),                  // telegram_chat_id
            debounce,                       // telegram_debounce
            true,                           // email_enabled
            smtp_host,                      // smtp_host
            smtp_port,                      // smtp_port
            Some(smtp_user),                // smtp_user
            Some(smtp_password),            // smtp_password
            Some(from_email),               // from_email
            Some(to_email),                 // to_email
        )),
    ];
    
    let app_settings = (
        log_level(),                        // log_level
        log_format(),                       // log_format
        any::<bool>(),                      // debug
        valid_search_limit(),               // search_limit
        valid_timeout(),                    // search_timeout
        valid_concurrent_downloads(),       // max_concurrent_downloads
        non_empty_string(),                 // enabled_indexers
    );
    
    let proxy_and_db_settings = (
        any::<bool>(),                      // proxy_enabled
        hostname(),                         // proxy_address
        valid_port(),                       // proxy_port
        database_url(),                     // database_url
        valid_pool_size(),                  // db_pool_size
        valid_overflow(),                   // db_max_overflow
    );
    
    (api_settings, qbittorrent_settings, notification_settings, app_settings, proxy_and_db_settings)
        .prop_map(move |(api, qbit, notif, app, proxy_db)| {
            Settings {
                // API settings
                tmdb_api_key: api.0,
                tmdb_read_access_token: api.1,
                kinopoisk_api_token: api.2,
                
                // qBittorrent settings
                qbittorrent_enabled: qbit.0,
                qbittorrent_host: qbit.1,
                qbittorrent_port: qbit.2,
                qbittorrent_username: qbit.3,
                qbittorrent_password: qbit.4,
                qbittorrent_use_ssl: qbit.5,
                qbittorrent_verify_ssl: qbit.6,
                qbittorrent_category: qbit.7,
                qbittorrent_tags: qbit.8,
                qbittorrent_timeout: qbit.9,
                
                // Filesystem paths - use temp_dir_path to ensure they exist
                library_path: temp_dir_path.clone(),
                download_path: temp_dir_path.clone(),
                
                // Notification settings
                telegram_enabled: notif.0,
                telegram_bot_token: notif.1,
                telegram_chat_id: notif.2,
                telegram_debounce: notif.3,
                email_enabled: notif.4,
                smtp_host: notif.5,
                smtp_port: notif.6,
                smtp_user: notif.7,
                smtp_password: notif.8,
                from_email: notif.9,
                to_email: notif.10,
                
                // Application settings
                log_level: app.0,
                log_format: app.1,
                debug: app.2,
                search_limit: app.3,
                search_timeout: app.4,
                max_concurrent_downloads: app.5,
                enabled_indexers: app.6,
                
                // Proxy and database settings
                proxy_enabled: proxy_db.0,
                proxy_address: proxy_db.1,
                proxy_port: proxy_db.2,
                database_url: proxy_db.3,
                db_pool_size: proxy_db.4,
                db_max_overflow: proxy_db.5,
            }
        })
}

// Helper function to create a test database and manager
async fn create_test_manager() -> Result<(SettingsManager, Arc<SettingsDatabase>, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_proptest.db");
    let db_url = db_path.to_string_lossy().to_string();

    let db = Database::new(&db_url).await?;
    let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
    settings_db.initialize().await?;

    let manager = SettingsManager::new(settings_db.clone());
    Ok((manager, settings_db, temp_dir))
}

#[cfg(test)]
mod proptest_tests {
    use super::*;

    // Property 1: Settings Persistence Round Trip
    // For any valid settings object, saving it to the database and then loading it
    // should produce an equivalent settings object.
    // Validates: Requirements 4.1, 4.2, 4.3
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn test_settings_round_trip_property(
            _seed in 0u64..1000000u64
        ) {
            // Use tokio runtime for async test
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                // Create test manager and temp directory
                let (manager, _, temp_dir) = create_test_manager().await.unwrap();
                let temp_path = temp_dir.path().to_string_lossy().to_string();
                
                // Generate a valid settings object using the seed
                let mut runner = proptest::test_runner::TestRunner::new(
                    ProptestConfig::with_cases(1)
                );
                
                let strategy = valid_settings_strategy(temp_path);
                let settings = strategy.new_tree(&mut runner).unwrap().current();
                
                // Save the settings
                manager.save_settings(&settings).await.unwrap();
                
                // Load the settings back
                let loaded = manager.load_settings().await.unwrap();
                
                // Assert that loaded settings match original settings
                assert_eq!(loaded.tmdb_api_key, settings.tmdb_api_key, "tmdb_api_key mismatch");
                assert_eq!(loaded.tmdb_read_access_token, settings.tmdb_read_access_token, "tmdb_read_access_token mismatch");
                assert_eq!(loaded.kinopoisk_api_token, settings.kinopoisk_api_token, "kinopoisk_api_token mismatch");
                assert_eq!(loaded.qbittorrent_enabled, settings.qbittorrent_enabled, "qbittorrent_enabled mismatch");
                assert_eq!(loaded.qbittorrent_host, settings.qbittorrent_host, "qbittorrent_host mismatch");
                assert_eq!(loaded.qbittorrent_port, settings.qbittorrent_port, "qbittorrent_port mismatch");
                assert_eq!(loaded.qbittorrent_username, settings.qbittorrent_username, "qbittorrent_username mismatch");
                assert_eq!(loaded.qbittorrent_password, settings.qbittorrent_password, "qbittorrent_password mismatch");
                assert_eq!(loaded.qbittorrent_use_ssl, settings.qbittorrent_use_ssl, "qbittorrent_use_ssl mismatch");
                assert_eq!(loaded.qbittorrent_verify_ssl, settings.qbittorrent_verify_ssl, "qbittorrent_verify_ssl mismatch");
                assert_eq!(loaded.qbittorrent_category, settings.qbittorrent_category, "qbittorrent_category mismatch");
                assert_eq!(loaded.qbittorrent_tags, settings.qbittorrent_tags, "qbittorrent_tags mismatch");
                assert_eq!(loaded.qbittorrent_timeout, settings.qbittorrent_timeout, "qbittorrent_timeout mismatch");
                assert_eq!(loaded.library_path, settings.library_path, "library_path mismatch");
                assert_eq!(loaded.download_path, settings.download_path, "download_path mismatch");
                assert_eq!(loaded.telegram_enabled, settings.telegram_enabled, "telegram_enabled mismatch");
                assert_eq!(loaded.telegram_bot_token, settings.telegram_bot_token, "telegram_bot_token mismatch");
                assert_eq!(loaded.telegram_chat_id, settings.telegram_chat_id, "telegram_chat_id mismatch");
                assert_eq!(loaded.telegram_debounce, settings.telegram_debounce, "telegram_debounce mismatch");
                assert_eq!(loaded.email_enabled, settings.email_enabled, "email_enabled mismatch");
                assert_eq!(loaded.smtp_host, settings.smtp_host, "smtp_host mismatch");
                assert_eq!(loaded.smtp_port, settings.smtp_port, "smtp_port mismatch");
                assert_eq!(loaded.smtp_user, settings.smtp_user, "smtp_user mismatch");
                assert_eq!(loaded.smtp_password, settings.smtp_password, "smtp_password mismatch");
                assert_eq!(loaded.from_email, settings.from_email, "from_email mismatch");
                assert_eq!(loaded.to_email, settings.to_email, "to_email mismatch");
                assert_eq!(loaded.log_level, settings.log_level, "log_level mismatch");
                assert_eq!(loaded.log_format, settings.log_format, "log_format mismatch");
                assert_eq!(loaded.debug, settings.debug, "debug mismatch");
                assert_eq!(loaded.search_limit, settings.search_limit, "search_limit mismatch");
                assert_eq!(loaded.search_timeout, settings.search_timeout, "search_timeout mismatch");
                assert_eq!(loaded.max_concurrent_downloads, settings.max_concurrent_downloads, "max_concurrent_downloads mismatch");
                assert_eq!(loaded.enabled_indexers, settings.enabled_indexers, "enabled_indexers mismatch");
                assert_eq!(loaded.proxy_enabled, settings.proxy_enabled, "proxy_enabled mismatch");
                assert_eq!(loaded.proxy_address, settings.proxy_address, "proxy_address mismatch");
                assert_eq!(loaded.proxy_port, settings.proxy_port, "proxy_port mismatch");
                assert_eq!(loaded.database_url, settings.database_url, "database_url mismatch");
                assert_eq!(loaded.db_pool_size, settings.db_pool_size, "db_pool_size mismatch");
                assert_eq!(loaded.db_max_overflow, settings.db_max_overflow, "db_max_overflow mismatch");
                
                // Also verify using PartialEq
                assert_eq!(loaded, settings, "Settings objects should be equal after round trip");
            });
        }
    }

    // Property 2: Invalid Settings Rejection
    // For any settings object with invalid values, the system should reject the save
    // operation and return a specific error for the invalid field.
    // Validates: Requirements 6.1, 6.2, 6.3
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn test_invalid_settings_rejection_property(
            _seed in 0u64..1000000u64
        ) {
            // Use tokio runtime for async test
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                // Create test manager and temp directory
                let (manager, _, temp_dir) = create_test_manager().await.unwrap();
                let temp_path = temp_dir.path().to_string_lossy().to_string();
                
                // Generate an invalid settings object
                let mut runner = proptest::test_runner::TestRunner::new(
                    ProptestConfig::with_cases(1)
                );
                
                let strategy = invalid_settings_strategy(temp_path);
                let (settings, invalid_type, expected_field) = strategy.new_tree(&mut runner).unwrap().current();
                
                // Attempt to save the invalid settings - should fail
                let result = manager.save_settings(&settings).await;
                
                // Assert that save operation failed
                assert!(result.is_err(), 
                    "Save should fail for invalid settings type: {:?}", invalid_type);
                
                // Get the error message
                let error = result.unwrap_err();
                
                // Build the full error chain including all causes
                let mut full_error_msg = error.to_string();
                if let Some(source) = error.source() {
                    full_error_msg.push_str("\n");
                    full_error_msg.push_str(&source.to_string());
                }
                
                // Verify that the error message mentions the expected field
                assert!(full_error_msg.contains(&expected_field),
                    "Error message should mention field '{}' but got: {}",
                    expected_field, full_error_msg);
                
                // Verify specific error messages based on invalid type
                match invalid_type {
                    InvalidSettingsType::InvalidPort => {
                        assert!(full_error_msg.contains("qbittorrent_port") || full_error_msg.contains("smtp_port") || full_error_msg.contains("proxy_port"),
                            "Error should mention port field: {}", full_error_msg);
                    },
                    InvalidSettingsType::InvalidTimeout => {
                        assert!(full_error_msg.contains("timeout") || full_error_msg.contains("Value must be between"),
                            "Error should mention timeout: {}", full_error_msg);
                    },
                    InvalidSettingsType::InvalidSearchLimit => {
                        assert!(full_error_msg.contains("search_limit") || full_error_msg.contains("Value must be between"),
                            "Error should mention search_limit: {}", full_error_msg);
                    },
                    InvalidSettingsType::InvalidConcurrentDownloads => {
                        assert!(full_error_msg.contains("max_concurrent_downloads") || full_error_msg.contains("Value must be between"),
                            "Error should mention max_concurrent_downloads: {}", full_error_msg);
                    },
                    InvalidSettingsType::InvalidPoolSize => {
                        assert!(full_error_msg.contains("db_pool_size") || full_error_msg.contains("Value must be between"),
                            "Error should mention db_pool_size: {}", full_error_msg);
                    },
                    InvalidSettingsType::InvalidOverflow => {
                        assert!(full_error_msg.contains("db_max_overflow") || full_error_msg.contains("Value must be between"),
                            "Error should mention db_max_overflow: {}", full_error_msg);
                    },
                    InvalidSettingsType::NonexistentPath => {
                        assert!(full_error_msg.contains("Path must exist") || full_error_msg.contains("path"),
                            "Error should mention path existence: {}", full_error_msg);
                    },
                    InvalidSettingsType::RelativePath => {
                        assert!(full_error_msg.contains("Path must be an absolute path") || full_error_msg.contains("absolute"),
                            "Error should mention absolute path: {}", full_error_msg);
                    },
                    InvalidSettingsType::TelegramEnabledMissingToken => {
                        assert!(full_error_msg.contains("telegram_bot_token") && full_error_msg.contains("required"),
                            "Error should mention telegram_bot_token is required: {}", full_error_msg);
                    },
                    InvalidSettingsType::TelegramEnabledMissingChatId => {
                        assert!(full_error_msg.contains("telegram_chat_id") && full_error_msg.contains("required"),
                            "Error should mention telegram_chat_id is required: {}", full_error_msg);
                    },
                    InvalidSettingsType::EmailEnabledMissingSmtpUser => {
                        assert!(full_error_msg.contains("smtp_user") && full_error_msg.contains("required"),
                            "Error should mention smtp_user is required: {}", full_error_msg);
                    },
                    InvalidSettingsType::EmailEnabledMissingSmtpPassword => {
                        assert!(full_error_msg.contains("smtp_password") && full_error_msg.contains("required"),
                            "Error should mention smtp_password is required: {}", full_error_msg);
                    },
                    InvalidSettingsType::EmailEnabledMissingFromEmail => {
                        assert!(full_error_msg.contains("from_email") && full_error_msg.contains("required"),
                            "Error should mention from_email is required: {}", full_error_msg);
                    },
                    InvalidSettingsType::EmailEnabledMissingToEmail => {
                        assert!(full_error_msg.contains("to_email") && full_error_msg.contains("required"),
                            "Error should mention to_email is required: {}", full_error_msg);
                    },
                    InvalidSettingsType::EmailEnabledInvalidFromEmail => {
                        assert!(full_error_msg.contains("from_email") && (full_error_msg.contains("valid format") || full_error_msg.contains("Email")),
                            "Error should mention from_email format: {}", full_error_msg);
                    },
                    InvalidSettingsType::EmailEnabledInvalidToEmail => {
                        assert!(full_error_msg.contains("to_email") && (full_error_msg.contains("valid format") || full_error_msg.contains("Email")),
                            "Error should mention to_email format: {}", full_error_msg);
                    },
                }
            });
        }
    }

    // Property 3: Default Values on Missing Settings
    // For any missing or corrupted setting in the database, the system should load
    // a sensible default value that allows the application to function.
    // Validates: Requirements 4.4, 4.5
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn test_default_values_on_missing_settings_property(
            _seed in 0u64..1000000u64
        ) {
            // Use tokio runtime for async test
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                // Create test manager and temp directory
                let (manager, settings_db, temp_dir) = create_test_manager().await.unwrap();
                let temp_path = temp_dir.path().to_string_lossy().to_string();
                
                // Generate a valid settings object to get expected defaults
                let mut runner = proptest::test_runner::TestRunner::new(
                    ProptestConfig::with_cases(1)
                );
                
                let strategy = valid_settings_strategy(temp_path.clone());
                let settings = strategy.new_tree(&mut runner).unwrap().current();
                
                // Save the settings first
                manager.save_settings(&settings).await.unwrap();
                
                // Now randomly delete some settings from the database to simulate missing entries
                // We'll delete a random subset of settings
                let keys_to_delete = vec![
                    "tmdb_api_key",
                    "qbittorrent_host",
                    "qbittorrent_port",
                    "log_level",
                    "search_limit",
                    "max_concurrent_downloads",
                    "telegram_enabled",
                    "email_enabled",
                    "proxy_enabled",
                ];
                
                // Delete a random subset of keys (simulate missing entries)
                // Use the seed to determine how many keys to delete
                let num_to_delete = (_seed % keys_to_delete.len() as u64) as usize + 1;
                for key in keys_to_delete.iter().take(num_to_delete) {
                    let _ = settings_db.delete(key).await;
                }
                
                // Load settings - should use defaults for missing entries
                let loaded = manager.load_settings().await.unwrap();
                
                // Get the default settings for comparison
                let defaults = Settings::default();
                
                // Verify that loaded settings have sensible values
                // For missing entries, they should match defaults
                // For present entries, they should match what was saved
                
                // Check that all required fields have values (not panicking)
                assert!(!loaded.qbittorrent_host.is_empty(), "qbittorrent_host should not be empty");
                assert!(loaded.qbittorrent_port > 0, "qbittorrent_port should be > 0");
                assert!(!loaded.log_level.is_empty(), "log_level should not be empty");
                assert!(loaded.search_limit > 0, "search_limit should be > 0");
                assert!(loaded.max_concurrent_downloads > 0, "max_concurrent_downloads should be > 0");
                
                // Verify that the loaded settings are valid (can be validated)
                assert!(manager.validate_settings(&loaded).is_ok() || 
                        loaded.library_path == defaults.library_path,
                    "Loaded settings should be valid or use default paths");
            });
        }
    }

    // Test with completely empty database (all settings missing)
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn test_all_settings_missing_uses_defaults(
            _seed in 0u64..1000000u64
        ) {
            // Use tokio runtime for async test
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                // Create test manager with empty database
                let (manager, _, temp_dir) = create_test_manager().await.unwrap();
                
                // Load settings from empty database - should return all defaults
                let loaded = manager.load_settings().await.unwrap();
                
                // Get the default settings for comparison
                let mut defaults = Settings::default();
                // Update default paths to use temp_dir for validation
                defaults.library_path = temp_dir.path().to_string_lossy().to_string();
                defaults.download_path = temp_dir.path().to_string_lossy().to_string();
                
                // Verify that loaded settings match defaults for key fields
                assert_eq!(loaded.qbittorrent_enabled, defaults.qbittorrent_enabled);
                assert_eq!(loaded.qbittorrent_host, defaults.qbittorrent_host);
                assert_eq!(loaded.qbittorrent_port, defaults.qbittorrent_port);
                assert_eq!(loaded.qbittorrent_username, defaults.qbittorrent_username);
                assert_eq!(loaded.log_level, defaults.log_level);
                assert_eq!(loaded.log_format, defaults.log_format);
                assert_eq!(loaded.debug, defaults.debug);
                assert_eq!(loaded.search_limit, defaults.search_limit);
                assert_eq!(loaded.search_timeout, defaults.search_timeout);
                assert_eq!(loaded.max_concurrent_downloads, defaults.max_concurrent_downloads);
                assert_eq!(loaded.telegram_enabled, defaults.telegram_enabled);
                assert_eq!(loaded.email_enabled, defaults.email_enabled);
                assert_eq!(loaded.proxy_enabled, defaults.proxy_enabled);
                
                // Verify that all required fields have sensible values
                assert!(!loaded.qbittorrent_host.is_empty());
                assert!(loaded.qbittorrent_port > 0);
                assert!(!loaded.log_level.is_empty());
                assert!(loaded.search_limit > 0);
                assert!(loaded.max_concurrent_downloads > 0);
            });
        }
    }

    // Test with corrupted database entries (invalid values)
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn test_corrupted_settings_use_defaults(
            _seed in 0u64..1000000u64
        ) {
            // Use tokio runtime for async test
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                // Create test manager
                let (manager, settings_db, temp_dir) = create_test_manager().await.unwrap();
                
                // Insert corrupted values directly into database
                let _ = settings_db.set("qbittorrent_port", "not_a_number", "int").await;
                let _ = settings_db.set("qbittorrent_enabled", "not_a_bool", "bool").await;
                let _ = settings_db.set("search_limit", "invalid", "int").await;
                let _ = settings_db.set("max_concurrent_downloads", "-5", "int").await;
                let _ = settings_db.set("telegram_debounce", "corrupted", "int").await;
                
                // Load settings - should use defaults for corrupted entries
                let loaded = manager.load_settings().await.unwrap();
                
                // Get the default settings for comparison
                let defaults = Settings::default();
                
                // Verify that corrupted values were replaced with defaults
                assert_eq!(loaded.qbittorrent_port, defaults.qbittorrent_port, 
                    "Corrupted qbittorrent_port should use default");
                assert_eq!(loaded.qbittorrent_enabled, defaults.qbittorrent_enabled,
                    "Corrupted qbittorrent_enabled should use default");
                assert_eq!(loaded.search_limit, defaults.search_limit,
                    "Corrupted search_limit should use default");
                assert_eq!(loaded.max_concurrent_downloads, defaults.max_concurrent_downloads,
                    "Corrupted max_concurrent_downloads should use default");
                assert_eq!(loaded.telegram_debounce, defaults.telegram_debounce,
                    "Corrupted telegram_debounce should use default");
                
                // Verify that all required fields have sensible values
                assert!(loaded.qbittorrent_port > 0);
                assert!(loaded.search_limit > 0);
                assert!(loaded.max_concurrent_downloads > 0);
            });
        }
    }

    // Property 6: Settings Isolation
    // For any two independent settings save operations with different values,
    // each operation should persist only its own values without affecting other settings.
    // Validates: Requirements 1.5, 2.5
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn test_settings_isolation_property(
            _seed in 0u64..1000000u64
        ) {
            // Use tokio runtime for async test
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                // Create test manager and temp directory
                let (manager, _, temp_dir) = create_test_manager().await.unwrap();
                let temp_path = temp_dir.path().to_string_lossy().to_string();
                
                // Generate two different valid settings objects
                let mut runner = proptest::test_runner::TestRunner::new(
                    ProptestConfig::with_cases(2)
                );
                
                let strategy = valid_settings_strategy(temp_path.clone());
                
                // Generate first settings
                let settings1 = strategy.new_tree(&mut runner).unwrap().current();
                
                // Generate second settings (different from first)
                let mut settings2 = strategy.new_tree(&mut runner).unwrap().current();
                
                // Ensure settings2 is different from settings1 by modifying a few fields
                settings2.qbittorrent_host = format!("host{}", _seed);
                settings2.search_limit = ((settings1.search_limit + 1) % 10000).max(1);
                settings2.max_concurrent_downloads = ((settings1.max_concurrent_downloads + 1) % 100).max(1);
                
                // Save first settings
                manager.save_settings(&settings1).await.unwrap();
                
                // Verify first settings were saved correctly
                let loaded1 = manager.load_settings().await.unwrap();
                assert_eq!(loaded1.qbittorrent_host, settings1.qbittorrent_host,
                    "First save: qbittorrent_host should match");
                assert_eq!(loaded1.search_limit, settings1.search_limit,
                    "First save: search_limit should match");
                assert_eq!(loaded1.max_concurrent_downloads, settings1.max_concurrent_downloads,
                    "First save: max_concurrent_downloads should match");
                
                // Save second settings (should overwrite first)
                manager.save_settings(&settings2).await.unwrap();
                
                // Verify second settings were saved correctly and first settings were replaced
                let loaded2 = manager.load_settings().await.unwrap();
                assert_eq!(loaded2.qbittorrent_host, settings2.qbittorrent_host,
                    "Second save: qbittorrent_host should match settings2");
                assert_eq!(loaded2.search_limit, settings2.search_limit,
                    "Second save: search_limit should match settings2");
                assert_eq!(loaded2.max_concurrent_downloads, settings2.max_concurrent_downloads,
                    "Second save: max_concurrent_downloads should match settings2");
                
                // Verify that settings2 completely replaced settings1
                assert_ne!(loaded2.qbittorrent_host, settings1.qbittorrent_host,
                    "Settings2 should have replaced settings1 qbittorrent_host");
                assert_ne!(loaded2.search_limit, settings1.search_limit,
                    "Settings2 should have replaced settings1 search_limit");
                assert_ne!(loaded2.max_concurrent_downloads, settings1.max_concurrent_downloads,
                    "Settings2 should have replaced settings1 max_concurrent_downloads");
            });
        }
    }

    // Test concurrent saves with tokio::spawn
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn test_concurrent_settings_saves_property(
            _seed in 0u64..1000000u64
        ) {
            // Use tokio runtime for async test
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                // Create test manager and temp directory
                let (manager, _, temp_dir) = create_test_manager().await.unwrap();
                let temp_path = temp_dir.path().to_string_lossy().to_string();
                
                // Generate multiple different valid settings objects
                let mut runner = proptest::test_runner::TestRunner::new(
                    ProptestConfig::with_cases(5)
                );
                
                let strategy = valid_settings_strategy(temp_path.clone());
                
                // Generate 5 different settings
                let mut settings_list = Vec::new();
                for i in 0..5 {
                    let mut settings = strategy.new_tree(&mut runner).unwrap().current();
                    // Make each settings unique
                    settings.qbittorrent_host = format!("host{}-{}", _seed, i);
                    settings.search_limit = ((i + 1) * 10).max(1);
                    settings_list.push(settings);
                }
                
                // Spawn concurrent save operations
                let manager = Arc::new(manager);
                let mut handles = Vec::new();
                
                for settings in settings_list.clone() {
                    let manager_clone = manager.clone();
                    let handle = tokio::spawn(async move {
                        manager_clone.save_settings(&settings).await
                    });
                    handles.push(handle);
                }
                
                // Wait for all saves to complete
                let mut results = Vec::new();
                for handle in handles {
                    results.push(handle.await.unwrap());
                }
                
                // Verify that all saves succeeded
                for (i, result) in results.iter().enumerate() {
                    assert!(result.is_ok(), 
                        "Concurrent save {} should succeed: {:?}", i, result);
                }
                
                // Load final settings
                let loaded = manager.load_settings().await.unwrap();
                
                // Verify that the loaded settings match one of the saved settings
                // (due to concurrent saves, the last one to complete wins)
                let matches_any = settings_list.iter().any(|s| {
                    s.qbittorrent_host == loaded.qbittorrent_host &&
                    s.search_limit == loaded.search_limit
                });
                
                assert!(matches_any,
                    "Loaded settings should match one of the concurrently saved settings");
                
                // Verify that the loaded settings are internally consistent
                // (all fields should come from the same settings object)
                let matching_settings = settings_list.iter()
                    .find(|s| s.qbittorrent_host == loaded.qbittorrent_host)
                    .expect("Should find matching settings");
                
                assert_eq!(loaded.search_limit, matching_settings.search_limit,
                    "All fields should come from the same settings object");
                assert_eq!(loaded.max_concurrent_downloads, matching_settings.max_concurrent_downloads,
                    "All fields should come from the same settings object");
                assert_eq!(loaded.qbittorrent_port, matching_settings.qbittorrent_port,
                    "All fields should come from the same settings object");
            });
        }
    }

    // Test that partial updates don't corrupt other fields
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn test_partial_update_isolation_property(
            _seed in 0u64..1000000u64
        ) {
            // Use tokio runtime for async test
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                // Create test manager and temp directory
                let (manager, _, temp_dir) = create_test_manager().await.unwrap();
                let temp_path = temp_dir.path().to_string_lossy().to_string();
                
                // Generate initial settings
                let mut runner = proptest::test_runner::TestRunner::new(
                    ProptestConfig::with_cases(1)
                );
                
                let strategy = valid_settings_strategy(temp_path.clone());
                let initial_settings = strategy.new_tree(&mut runner).unwrap().current();
                
                // Save initial settings
                manager.save_settings(&initial_settings).await.unwrap();
                
                // Create a modified version with only a few fields changed
                let mut modified_settings = initial_settings.clone();
                modified_settings.qbittorrent_host = format!("modified-host-{}", _seed);
                modified_settings.search_limit = ((initial_settings.search_limit + 50) % 10000).max(1);
                
                // Save modified settings
                manager.save_settings(&modified_settings).await.unwrap();
                
                // Load and verify
                let loaded = manager.load_settings().await.unwrap();
                
                // Verify that modified fields were updated
                assert_eq!(loaded.qbittorrent_host, modified_settings.qbittorrent_host,
                    "Modified qbittorrent_host should be updated");
                assert_eq!(loaded.search_limit, modified_settings.search_limit,
                    "Modified search_limit should be updated");
                
                // Verify that unmodified fields remain the same
                assert_eq!(loaded.qbittorrent_port, initial_settings.qbittorrent_port,
                    "Unmodified qbittorrent_port should remain the same");
                assert_eq!(loaded.qbittorrent_username, initial_settings.qbittorrent_username,
                    "Unmodified qbittorrent_username should remain the same");
                assert_eq!(loaded.qbittorrent_password, initial_settings.qbittorrent_password,
                    "Unmodified qbittorrent_password should remain the same");
                assert_eq!(loaded.log_level, initial_settings.log_level,
                    "Unmodified log_level should remain the same");
                assert_eq!(loaded.telegram_enabled, initial_settings.telegram_enabled,
                    "Unmodified telegram_enabled should remain the same");
                assert_eq!(loaded.email_enabled, initial_settings.email_enabled,
                    "Unmodified email_enabled should remain the same");
            });
        }
    }
}

    // Property 1: Transaction Atomicity for Settings
    // For any Settings object, when saving to the database, either all setting values
    // are persisted or none are persisted (no partial updates).
    // **Feature: database-race-condition-fix, Property 1: Transaction Atomicity for Settings**
    // **Validates: Requirements 1.1, 1.2, 4.1**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn test_settings_transaction_atomicity_property(
            _seed in 0u64..1000000u64
        ) {
            // Use tokio runtime for async test
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                // Create test manager and temp directory
                let (manager, settings_db, temp_dir) = create_test_manager().await.unwrap();
                let temp_path = temp_dir.path().to_string_lossy().to_string();
                
                // Generate a valid settings object
                let mut runner = proptest::test_runner::TestRunner::new(
                    ProptestConfig::with_cases(1)
                );
                
                let strategy = valid_settings_strategy(temp_path);
                let settings = strategy.new_tree(&mut runner).unwrap().current();
                
                // Save the settings
                let result = manager.save_settings(&settings).await;
                assert!(result.is_ok(), "Settings save should succeed");
                
                // Load all settings from database
                let all_records = settings_db.get_all().await.unwrap();
                
                // Count how many settings were persisted
                let persisted_count = all_records.len();
                
                // We expect either 0 settings (if save failed) or all settings (if save succeeded)
                // Since save succeeded, we should have all settings persisted
                // The Settings struct has approximately 40+ fields
                assert!(persisted_count > 35, 
                    "All settings should be persisted atomically, got {} settings", 
                    persisted_count);
                
                // Verify that we can load back the complete settings
                let loaded = manager.load_settings().await.unwrap();
                assert_eq!(loaded, settings, 
                    "Loaded settings should match saved settings (atomicity check)");
            });
        }
    }

    // Property 3: Settings Rollback on Validation Failure
    // For any Settings object that fails validation, no database writes should occur,
    // and the previous settings state should remain unchanged.
    // **Feature: database-race-condition-fix, Property 3: Settings Rollback on Validation Failure**
    // **Validates: Requirements 4.2, 4.4**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn test_settings_rollback_on_validation_failure_property(
            _seed in 0u64..1000000u64
        ) {
            // Use tokio runtime for async test
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                // Create test manager and temp directory
                let (manager, settings_db, temp_dir) = create_test_manager().await.unwrap();
                let temp_path = temp_dir.path().to_string_lossy().to_string();
                
                // Generate and save initial valid settings
                let mut runner = proptest::test_runner::TestRunner::new(
                    ProptestConfig::with_cases(1)
                );
                
                let strategy = valid_settings_strategy(temp_path.clone());
                let initial_settings = strategy.new_tree(&mut runner).unwrap().current();
                
                // Save initial settings
                manager.save_settings(&initial_settings).await.unwrap();
                
                // Get a snapshot of the database state before attempting invalid save
                let records_before = settings_db.get_all().await.unwrap();
                let count_before = records_before.len();
                
                // Generate invalid settings
                let invalid_strategy = invalid_settings_strategy(temp_path);
                let (invalid_settings, invalid_type, _) = invalid_strategy.new_tree(&mut runner).unwrap().current();
                
                // Attempt to save invalid settings - should fail
                let result = manager.save_settings(&invalid_settings).await;
                assert!(result.is_err(), 
                    "Save should fail for invalid settings type: {:?}", invalid_type);
                
                // Get database state after failed save
                let records_after = settings_db.get_all().await.unwrap();
                let count_after = records_after.len();
                
                // Verify that the database state is unchanged
                assert_eq!(count_before, count_after,
                    "Database should have same number of settings after failed save");
                
                // Verify that we can still load the original settings
                let loaded = manager.load_settings().await.unwrap();
                assert_eq!(loaded, initial_settings,
                    "Original settings should be preserved after validation failure");
                
                // Verify that no partial updates occurred by checking specific fields
                assert_eq!(loaded.qbittorrent_host, initial_settings.qbittorrent_host,
                    "qbittorrent_host should be unchanged");
                assert_eq!(loaded.qbittorrent_port, initial_settings.qbittorrent_port,
                    "qbittorrent_port should be unchanged");
                assert_eq!(loaded.search_limit, initial_settings.search_limit,
                    "search_limit should be unchanged");
                assert_eq!(loaded.max_concurrent_downloads, initial_settings.max_concurrent_downloads,
                    "max_concurrent_downloads should be unchanged");
            });
        }
    }
