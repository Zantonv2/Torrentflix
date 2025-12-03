// Property-based tests for database transaction wrapper
// Feature: database-race-condition-fix

use engine::database::transaction::{RetryConfig, is_retryable_error};
use engine::database::{Database, SettingsDatabase};
use engine::settings::{SettingsManager, Settings};
use proptest::prelude::*;
use std::sync::{Arc, Mutex};

// Generator for valid retry config values
fn valid_max_attempts() -> impl Strategy<Value = u32> {
    1u32..=20u32
}

fn valid_base_delay_ms() -> impl Strategy<Value = u64> {
    1u64..=1000u64
}

fn valid_max_delay_ms() -> impl Strategy<Value = u64> {
    100u64..=10000u64
}

fn valid_exponential_base() -> impl Strategy<Value = u32> {
    2u32..=5u32
}

fn attempt_number() -> impl Strategy<Value = u32> {
    0u32..=15u32
}

// Generator for RetryConfig
// Ensures max_delay_ms >= base_delay_ms for valid configs
fn retry_config_strategy() -> impl Strategy<Value = RetryConfig> {
    (
        valid_max_attempts(),
        valid_base_delay_ms(),
        valid_exponential_base(),
    ).prop_flat_map(|(max_attempts, base_delay_ms, exponential_base)| {
        // Generate max_delay_ms that is >= base_delay_ms
        let min_max_delay = base_delay_ms.max(100);
        (
            Just(max_attempts),
            Just(base_delay_ms),
            min_max_delay..=10000u64,
            Just(exponential_base),
        )
    }).prop_map(|(max_attempts, base_delay_ms, max_delay_ms, exponential_base)| {
        RetryConfig {
            max_attempts,
            base_delay_ms,
            max_delay_ms,
            exponential_base,
        }
    })
}

#[cfg(test)]
mod proptest_tests {
    use super::*;

    // Property 4: Retry with Exponential Backoff
    // For any database operation that encounters SQLITE_BUSY, the retry delay for 
    // attempt N should be min(base_delay * (exponential_base ^ N), max_delay).
    // Validates: Requirements 2.1, 2.3
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn test_exponential_backoff_calculation_property(
            config in retry_config_strategy(),
            attempt in attempt_number(),
        ) {
            // Calculate the expected delay using the formula
            let expected_delay_ms = config.base_delay_ms
                .saturating_mul(config.exponential_base.saturating_pow(attempt) as u64)
                .min(config.max_delay_ms);
            
            // Get the actual delay from the config
            let actual_delay = config.calculate_backoff_delay(attempt);
            
            // Assert they match
            assert_eq!(
                actual_delay.as_millis() as u64,
                expected_delay_ms,
                "Backoff delay mismatch for attempt {} with config {:?}",
                attempt,
                config
            );
            
            // Additional property: delay should never exceed max_delay_ms
            assert!(
                actual_delay.as_millis() as u64 <= config.max_delay_ms,
                "Delay {} exceeds max_delay_ms {} for attempt {}",
                actual_delay.as_millis(),
                config.max_delay_ms,
                attempt
            );
            
            // Additional property: delay should be at least base_delay_ms for attempt 0
            if attempt == 0 {
                assert_eq!(
                    actual_delay.as_millis() as u64,
                    config.base_delay_ms,
                    "Delay for attempt 0 should equal base_delay_ms"
                );
            }
        }
    }

    // Property 5: Retry Logging
    // For any database operation that succeeds after retrying, the logs should 
    // contain the retry count and the operation should eventually succeed if 
    // retries are not exhausted.
    // Validates: Requirements 2.4, 7.2
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]
        
        #[test]
        fn test_retry_logging_property(
            max_attempts in 2u32..=5u32,
            fail_count in 1u32..=4u32,
        ) {
            // Use tokio runtime for async test
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                // Ensure fail_count is less than max_attempts so we eventually succeed
                let actual_fail_count = fail_count.min(max_attempts - 1);
                
                let config = RetryConfig {
                    max_attempts,
                    base_delay_ms: 1, // Use very short delays for testing
                    max_delay_ms: 10,
                    exponential_base: 2,
                };
                
                // Counter to track how many times the operation has been called
                let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
                let call_count_clone = call_count.clone();
                
                // Operation that fails actual_fail_count times, then succeeds
                let operation = move || {
                    let count = call_count_clone.clone();
                    async move {
                        let current = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        
                        if current < actual_fail_count {
                            // Simulate SQLITE_BUSY error
                            let busy_error = sqlx::Error::Database(Box::new(SqliteError {
                                code: Some("SQLITE_BUSY".into()),
                                message: "database is locked".to_string(),
                            }));
                            Err(anyhow::Error::new(busy_error))
                        } else {
                            Ok(42)
                        }
                    }
                };
                
                // Execute with retry
                let result = engine::database::transaction::with_retry(operation, &config).await;
                
                // Should succeed after retries
                assert!(result.is_ok(), "Operation should succeed after {} retries", actual_fail_count);
                assert_eq!(result.unwrap(), 42);
                
                // Verify the operation was called the expected number of times
                let final_count = call_count.load(std::sync::atomic::Ordering::SeqCst);
                assert_eq!(
                    final_count, 
                    actual_fail_count + 1,
                    "Operation should be called {} times (fail {} times, then succeed)",
                    actual_fail_count + 1,
                    actual_fail_count
                );
            });
        }
    }

    // Property 10: Operation Logging
    // For any database operation, the logs should contain the operation type, 
    // parameters (at debug level), and duration (on completion).
    // Validates: Requirements 7.1, 7.3
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]
        
        #[test]
        fn test_operation_logging_property(
            success_after_attempts in 0u32..=3u32,
        ) {
            use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
            use std::sync::{Arc, Mutex};
            
            // Use tokio runtime for async test
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                // Create a custom layer to capture log messages
                let logs = Arc::new(Mutex::new(Vec::new()));
                let logs_clone = logs.clone();
                
                let layer = tracing_subscriber::fmt::layer()
                    .with_writer(move || {
                        let logs = logs_clone.clone();
                        LogWriter { logs }
                    })
                    .with_ansi(false);
                
                let subscriber = tracing_subscriber::registry().with(layer);
                
                // Set as global default for this test
                let _guard = tracing::subscriber::set_default(subscriber);
                
                let config = RetryConfig {
                    max_attempts: 5,
                    base_delay_ms: 1,
                    max_delay_ms: 10,
                    exponential_base: 2,
                };
                
                // Counter to track attempts
                let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
                let call_count_clone = call_count.clone();
                
                // Operation that fails success_after_attempts times, then succeeds
                let operation = move || {
                    let count = call_count_clone.clone();
                    async move {
                        let current = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        
                        if current < success_after_attempts {
                            // Simulate SQLITE_BUSY error
                            let busy_error = sqlx::Error::Database(Box::new(SqliteError {
                                code: Some("SQLITE_BUSY".into()),
                                message: "database is locked".to_string(),
                            }));
                            Err(anyhow::Error::new(busy_error))
                        } else {
                            Ok(42)
                        }
                    }
                };
                
                // Execute with retry
                let result = engine::database::transaction::with_retry(operation, &config).await;
                
                // Should succeed
                assert!(result.is_ok(), "Operation should succeed");
                
                // Check logs
                let captured_logs = logs.lock().unwrap();
                let log_text = String::from_utf8_lossy(&captured_logs);
                
                // Property: Logs should contain operation execution information
                assert!(
                    log_text.contains("Executing database operation") || 
                    log_text.contains("database operation"),
                    "Logs should contain operation execution information. Got: {}",
                    log_text
                );
                
                // Property: If retries occurred, logs should contain retry information
                if success_after_attempts > 0 {
                    assert!(
                        log_text.contains("retry") || log_text.contains("attempt"),
                        "Logs should contain retry/attempt information when retries occur. Got: {}",
                        log_text
                    );
                }
                
                // Property: If operation succeeded after retries, logs should indicate success
                if success_after_attempts > 0 {
                    assert!(
                        log_text.contains("succeeded") || log_text.contains("success"),
                        "Logs should indicate success after retries. Got: {}",
                        log_text
                    );
                }
            });
        }
    }
    
    // Property 11: Error Context Logging
    // For any database error, the logs should contain the full error context 
    // including error message, error code, and relevant operation details.
    // Validates: Requirements 7.4, 6.2
    #[test]
    fn test_error_context_logging_property() {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
        use std::sync::{Arc, Mutex};
        
        // Use tokio runtime for async test
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            // Create a custom layer to capture log messages
            let logs = Arc::new(Mutex::new(Vec::new()));
            let logs_clone = logs.clone();
            
            let layer = tracing_subscriber::fmt::layer()
                .with_writer(move || {
                    let logs = logs_clone.clone();
                    LogWriter { logs }
                })
                .with_ansi(false);
            
            let subscriber = tracing_subscriber::registry().with(layer);
            
            // Set as global default for this test
            let _guard = tracing::subscriber::set_default(subscriber);
            
            let config = RetryConfig {
                max_attempts: 2, // Only 2 attempts so we exhaust retries quickly
                base_delay_ms: 1,
                max_delay_ms: 10,
                exponential_base: 2,
            };
            
            // Test 1: Retryable error that exhausts retries
            {
                let operation = || async {
                    // Always fail with SQLITE_BUSY
                    let busy_error = sqlx::Error::Database(Box::new(SqliteError {
                        code: Some("SQLITE_BUSY".into()),
                        message: "database is locked".to_string(),
                    }));
                    Err(anyhow::Error::new(busy_error)) as anyhow::Result<i32>
                };
                
                let result: anyhow::Result<i32> = engine::database::transaction::with_retry(operation, &config).await;
                
                // Should fail after exhausting retries
                assert!(result.is_err(), "Operation should fail after exhausting retries");
                
                // Check logs
                let mut captured_logs = logs.lock().unwrap();
                let log_text = String::from_utf8_lossy(&captured_logs);
                
                // Property: Logs should contain error information
                assert!(
                    log_text.contains("error") || log_text.contains("failed"),
                    "Logs should contain error information. Got: {}",
                    log_text
                );
                
                // Property: Logs should contain information about exhausted retries
                assert!(
                    log_text.contains("exhausting") || log_text.contains("all retry attempts") || 
                    log_text.contains("failed after"),
                    "Logs should indicate retry exhaustion. Got: {}",
                    log_text
                );
                
                // Clear logs for next test
                captured_logs.clear();
            }
            
            // Test 2: Non-retryable error
            {
                let operation = || async {
                    // Fail with non-retryable error
                    let constraint_error = sqlx::Error::Database(Box::new(SqliteError {
                        code: Some("SQLITE_CONSTRAINT".into()),
                        message: "constraint violation".to_string(),
                    }));
                    Err(anyhow::Error::new(constraint_error)) as anyhow::Result<i32>
                };
                
                let result: anyhow::Result<i32> = engine::database::transaction::with_retry(operation, &config).await;
                
                // Should fail immediately
                assert!(result.is_err(), "Operation should fail with non-retryable error");
                
                // Check logs
                let captured_logs = logs.lock().unwrap();
                let log_text = String::from_utf8_lossy(&captured_logs);
                
                // Property: Logs should contain error information
                assert!(
                    log_text.contains("error") || log_text.contains("failed"),
                    "Logs should contain error information for non-retryable errors. Got: {}",
                    log_text
                );
                
                // Property: Logs should indicate non-retryable error
                assert!(
                    log_text.contains("non-retryable") || log_text.contains("constraint"),
                    "Logs should indicate non-retryable error type. Got: {}",
                    log_text
                );
            }
        });
    }

    // Property 9: Error Classification
    // For any database error, the system should correctly classify it as either 
    // transient (retryable) or permanent (non-retryable) based on the error code.
    // Validates: Requirements 6.4
    #[test]
    fn test_error_classification_property() {
        // Test SQLITE_BUSY error (should be retryable)
        let busy_error = sqlx::Error::Database(Box::new(SqliteError {
            code: Some("SQLITE_BUSY".into()),
            message: "database is locked".to_string(),
        }));
        let busy_anyhow = anyhow::Error::new(busy_error);
        assert!(
            is_retryable_error(&busy_anyhow),
            "SQLITE_BUSY should be classified as retryable"
        );
        
        // Test SQLITE_LOCKED error (should be retryable)
        let locked_error = sqlx::Error::Database(Box::new(SqliteError {
            code: Some("SQLITE_LOCKED".into()),
            message: "table is locked".to_string(),
        }));
        let locked_anyhow = anyhow::Error::new(locked_error);
        assert!(
            is_retryable_error(&locked_anyhow),
            "SQLITE_LOCKED should be classified as retryable"
        );
        
        // Test PoolTimedOut error (should be retryable)
        let pool_timeout_error = sqlx::Error::PoolTimedOut;
        let pool_timeout_anyhow = anyhow::Error::new(pool_timeout_error);
        assert!(
            is_retryable_error(&pool_timeout_anyhow),
            "PoolTimedOut should be classified as retryable"
        );
        
        // Test SQLITE_CONSTRAINT error (should NOT be retryable)
        let constraint_error = sqlx::Error::Database(Box::new(SqliteError {
            code: Some("SQLITE_CONSTRAINT".into()),
            message: "constraint violation".to_string(),
        }));
        let constraint_anyhow = anyhow::Error::new(constraint_error);
        assert!(
            !is_retryable_error(&constraint_anyhow),
            "SQLITE_CONSTRAINT should NOT be classified as retryable"
        );
        
        // Test SQLITE_CORRUPT error (should NOT be retryable)
        let corrupt_error = sqlx::Error::Database(Box::new(SqliteError {
            code: Some("SQLITE_CORRUPT".into()),
            message: "database corruption".to_string(),
        }));
        let corrupt_anyhow = anyhow::Error::new(corrupt_error);
        assert!(
            !is_retryable_error(&corrupt_anyhow),
            "SQLITE_CORRUPT should NOT be classified as retryable"
        );
        
        // Test generic error (should NOT be retryable)
        let generic_error = anyhow::anyhow!("some other error");
        assert!(
            !is_retryable_error(&generic_error),
            "Generic errors should NOT be classified as retryable"
        );
    }
}

// Mock SqliteError for testing
#[derive(Debug)]
struct SqliteError {
    code: Option<std::borrow::Cow<'static, str>>,
    message: String,
}

impl std::fmt::Display for SqliteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SqliteError {}

impl sqlx::error::DatabaseError for SqliteError {
    fn message(&self) -> &str {
        &self.message
    }

    fn code(&self) -> Option<std::borrow::Cow<'_, str>> {
        self.code.as_ref().map(|c| std::borrow::Cow::Borrowed(c.as_ref()))
    }

    fn kind(&self) -> sqlx::error::ErrorKind {
        sqlx::error::ErrorKind::Other
    }

    fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        self
    }

    fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
        self
    }

    fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
        self
    }
}

// Helper struct for capturing log output
struct LogWriter {
    logs: Arc<Mutex<Vec<u8>>>,
}

impl std::io::Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut logs = self.logs.lock().unwrap();
        logs.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// Property 2: Concurrent Settings Save Serialization
// For any two concurrent save_settings() calls with different Settings objects, 
// the final database state should match one of the two Settings objects completely 
// (no interleaving of values from both).
// Validates: Requirements 1.3, 4.3

// Generator for valid settings with random values
// We test a subset of fields to keep the test manageable
fn settings_strategy() -> impl Strategy<Value = Settings> {
    (
        any::<bool>(), // qbittorrent_enabled
        1024u16..=65535u16, // qbittorrent_port
        any::<bool>(), // qbittorrent_use_ssl
        1u64..=300u64, // qbittorrent_timeout
        1u32..=10000u32, // search_limit
        1u64..=300u64, // search_timeout
        1u32..=100u32, // max_concurrent_downloads
        any::<bool>(), // debug
        1u32..=100u32, // db_pool_size
        0u32..=100u32, // db_max_overflow
    ).prop_map(|(
        qbittorrent_enabled,
        qbittorrent_port,
        qbittorrent_use_ssl,
        qbittorrent_timeout,
        search_limit,
        search_timeout,
        max_concurrent_downloads,
        debug,
        db_pool_size,
        db_max_overflow,
    )| {
        let mut settings = Settings::default();
        settings.qbittorrent_enabled = qbittorrent_enabled;
        settings.qbittorrent_port = qbittorrent_port;
        settings.qbittorrent_use_ssl = qbittorrent_use_ssl;
        settings.qbittorrent_timeout = qbittorrent_timeout;
        settings.search_limit = search_limit;
        settings.search_timeout = search_timeout;
        settings.max_concurrent_downloads = max_concurrent_downloads;
        settings.debug = debug;
        settings.db_pool_size = db_pool_size;
        settings.db_max_overflow = db_max_overflow;
        settings
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    
    #[test]
    fn test_concurrent_settings_save_serialization_property(
        settings1 in settings_strategy(),
        settings2 in settings_strategy(),
    ) {
        // Use tokio runtime for async test
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            // Create a temporary database for this test
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test_concurrent.db");
            let db_url = format!("sqlite://{}", db_path.to_string_lossy());
            
            // Initialize database and settings manager
            let db = Database::new(&db_url).await.unwrap();
            let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
            settings_db.initialize().await.unwrap();
            
            let manager = Arc::new(SettingsManager::new(settings_db.clone()));
            
            // Set valid paths for both settings (required for validation)
            let mut settings1 = settings1;
            let mut settings2 = settings2;
            settings1.library_path = temp_dir.path().to_string_lossy().to_string();
            settings1.download_path = temp_dir.path().to_string_lossy().to_string();
            settings2.library_path = temp_dir.path().to_string_lossy().to_string();
            settings2.download_path = temp_dir.path().to_string_lossy().to_string();
            
            // Spawn two concurrent save operations
            let manager1 = manager.clone();
            let manager2 = manager.clone();
            let settings1_clone = settings1.clone();
            let settings2_clone = settings2.clone();
            
            let handle1 = tokio::spawn(async move {
                manager1.save_settings(&settings1_clone).await
            });
            
            let handle2 = tokio::spawn(async move {
                manager2.save_settings(&settings2_clone).await
            });
            
            // Wait for both to complete
            let result1 = handle1.await.unwrap();
            let result2 = handle2.await.unwrap();
            
            // Both should succeed (no errors)
            assert!(result1.is_ok(), "First save should succeed: {:?}", result1);
            assert!(result2.is_ok(), "Second save should succeed: {:?}", result2);
            
            // Load the final state
            let final_settings = manager.load_settings().await.unwrap();
            
            // The final state should match either settings1 or settings2 completely
            // (no interleaving of values from both)
            // We check the fields we randomized in the generator
            let matches_settings1 = 
                final_settings.qbittorrent_enabled == settings1.qbittorrent_enabled &&
                final_settings.qbittorrent_port == settings1.qbittorrent_port &&
                final_settings.qbittorrent_use_ssl == settings1.qbittorrent_use_ssl &&
                final_settings.qbittorrent_timeout == settings1.qbittorrent_timeout &&
                final_settings.search_limit == settings1.search_limit &&
                final_settings.search_timeout == settings1.search_timeout &&
                final_settings.max_concurrent_downloads == settings1.max_concurrent_downloads &&
                final_settings.debug == settings1.debug &&
                final_settings.db_pool_size == settings1.db_pool_size &&
                final_settings.db_max_overflow == settings1.db_max_overflow;
            
            let matches_settings2 = 
                final_settings.qbittorrent_enabled == settings2.qbittorrent_enabled &&
                final_settings.qbittorrent_port == settings2.qbittorrent_port &&
                final_settings.qbittorrent_use_ssl == settings2.qbittorrent_use_ssl &&
                final_settings.qbittorrent_timeout == settings2.qbittorrent_timeout &&
                final_settings.search_limit == settings2.search_limit &&
                final_settings.search_timeout == settings2.search_timeout &&
                final_settings.max_concurrent_downloads == settings2.max_concurrent_downloads &&
                final_settings.debug == settings2.debug &&
                final_settings.db_pool_size == settings2.db_pool_size &&
                final_settings.db_max_overflow == settings2.db_max_overflow;
            
            // Assert that the final state matches one of the two settings completely
            assert!(
                matches_settings1 || matches_settings2,
                "Final settings state should match either settings1 or settings2 completely. \
                 Got a mix of both, which indicates transaction interleaving.\n\
                 Settings1: enabled={}, port={}, ssl={}, timeout={}, search_limit={}, \
                 search_timeout={}, max_downloads={}, debug={}, pool={}, overflow={}\n\
                 Settings2: enabled={}, port={}, ssl={}, timeout={}, search_limit={}, \
                 search_timeout={}, max_downloads={}, debug={}, pool={}, overflow={}\n\
                 Final: enabled={}, port={}, ssl={}, timeout={}, search_limit={}, \
                 search_timeout={}, max_downloads={}, debug={}, pool={}, overflow={}",
                settings1.qbittorrent_enabled, settings1.qbittorrent_port, settings1.qbittorrent_use_ssl,
                settings1.qbittorrent_timeout, settings1.search_limit, settings1.search_timeout,
                settings1.max_concurrent_downloads, settings1.debug, settings1.db_pool_size,
                settings1.db_max_overflow,
                settings2.qbittorrent_enabled, settings2.qbittorrent_port, settings2.qbittorrent_use_ssl,
                settings2.qbittorrent_timeout, settings2.search_limit, settings2.search_timeout,
                settings2.max_concurrent_downloads, settings2.debug, settings2.db_pool_size,
                settings2.db_max_overflow,
                final_settings.qbittorrent_enabled, final_settings.qbittorrent_port, final_settings.qbittorrent_use_ssl,
                final_settings.qbittorrent_timeout, final_settings.search_limit, final_settings.search_timeout,
                final_settings.max_concurrent_downloads, final_settings.debug, final_settings.db_pool_size,
                final_settings.db_max_overflow
            );
        });
    }
}
