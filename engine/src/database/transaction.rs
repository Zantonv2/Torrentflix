use anyhow::{Result, Context};
use sqlx::{SqlitePool, Sqlite, Transaction};
use std::future::Future;
use std::time::Duration;
use tracing::{debug, warn, error, info};

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_base: u32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            exponential_base: 2,
        }
    }
}

impl RetryConfig {
    /// Calculate the backoff delay for a given attempt number
    pub fn calculate_backoff_delay(&self, attempt: u32) -> Duration {
        let delay_ms = self.base_delay_ms
            .saturating_mul(self.exponential_base.saturating_pow(attempt) as u64);
        let capped_delay_ms = delay_ms.min(self.max_delay_ms);
        Duration::from_millis(capped_delay_ms)
    }
}

/// Classify database errors as transient (retryable) or permanent
pub fn is_retryable_error(error: &anyhow::Error) -> bool {
    // Check if this is a sqlx error
    if let Some(sqlx_err) = error.downcast_ref::<sqlx::Error>() {
        match sqlx_err {
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    let code_str = code.as_ref();
                    code_str == "SQLITE_BUSY" || code_str == "SQLITE_LOCKED"
                } else {
                    false
                }
            }
            sqlx::Error::PoolTimedOut => true,
            _ => false,
        }
    } else {
        false
    }
}

/// Execute a database operation with automatic retry on transient errors
pub async fn with_retry<F, T, Fut>(
    operation: F,
    config: &RetryConfig,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut last_error = None;
    
    for attempt in 0..config.max_attempts {
        debug!(
            attempt = attempt + 1,
            max_attempts = config.max_attempts,
            "Executing database operation"
        );
        
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    info!(
                        retry_count = attempt,
                        "Database operation succeeded after retries"
                    );
                }
                return Ok(result);
            }
            Err(err) => {
                last_error = Some(err);
                let err_ref = last_error.as_ref().unwrap();
                
                if !is_retryable_error(err_ref) {
                    error!(
                        error = %err_ref,
                        "Database operation failed with non-retryable error"
                    );
                    return Err(last_error.unwrap());
                }
                
                if attempt + 1 < config.max_attempts {
                    let delay = config.calculate_backoff_delay(attempt);
                    warn!(
                        attempt = attempt + 1,
                        max_attempts = config.max_attempts,
                        delay_ms = delay.as_millis(),
                        error = %err_ref,
                        "Database operation failed, retrying after delay"
                    );
                    tokio::time::sleep(delay).await;
                } else {
                    error!(
                        attempts = config.max_attempts,
                        error = %err_ref,
                        "Database operation failed after exhausting all retry attempts"
                    );
                }
            }
        }
    }
    
    Err(last_error.unwrap().context("Operation failed after all retry attempts"))
}

/// Execute multiple operations in a single transaction
pub async fn with_transaction<F, T, Fut>(
    pool: &SqlitePool,
    operation: F,
) -> Result<T>
where
    F: FnOnce(&mut Transaction<'_, Sqlite>) -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let start = std::time::Instant::now();
    debug!("Starting database transaction");
    
    let mut tx = pool.begin().await
        .context("Failed to begin transaction")?;
    
    match operation(&mut tx).await {
        Ok(result) => {
            tx.commit().await
                .context("Failed to commit transaction")?;
            
            let duration = start.elapsed();
            debug!(
                duration_ms = duration.as_millis(),
                "Transaction committed successfully"
            );
            
            Ok(result)
        }
        Err(err) => {
            warn!(
                error = %err,
                "Transaction failed, rolling back"
            );
            
            tx.rollback().await
                .context("Failed to rollback transaction")?;
            
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_retry_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.base_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 5000);
        assert_eq!(config.exponential_base, 2);
    }

    #[test]
    fn test_backoff_calculation() {
        let config = RetryConfig::default();
        
        // Attempt 0: 100 * 2^0 = 100ms
        assert_eq!(config.calculate_backoff_delay(0).as_millis(), 100);
        
        // Attempt 1: 100 * 2^1 = 200ms
        assert_eq!(config.calculate_backoff_delay(1).as_millis(), 200);
        
        // Attempt 2: 100 * 2^2 = 400ms
        assert_eq!(config.calculate_backoff_delay(2).as_millis(), 400);
        
        // Attempt 3: 100 * 2^3 = 800ms
        assert_eq!(config.calculate_backoff_delay(3).as_millis(), 800);
        
        // Attempt 4: 100 * 2^4 = 1600ms
        assert_eq!(config.calculate_backoff_delay(4).as_millis(), 1600);
        
        // Attempt 10: Should be capped at max_delay_ms (5000ms)
        assert_eq!(config.calculate_backoff_delay(10).as_millis(), 5000);
    }

    #[test]
    fn test_backoff_with_custom_config() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay_ms: 50,
            max_delay_ms: 1000,
            exponential_base: 3,
        };
        
        // Attempt 0: 50 * 3^0 = 50ms
        assert_eq!(config.calculate_backoff_delay(0).as_millis(), 50);
        
        // Attempt 1: 50 * 3^1 = 150ms
        assert_eq!(config.calculate_backoff_delay(1).as_millis(), 150);
        
        // Attempt 2: 50 * 3^2 = 450ms
        assert_eq!(config.calculate_backoff_delay(2).as_millis(), 450);
        
        // Attempt 5: Should be capped at 1000ms
        assert_eq!(config.calculate_backoff_delay(5).as_millis(), 1000);
    }
}
