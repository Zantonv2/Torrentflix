use anyhow::{Result, anyhow};
use super::settings::LibraryConfig;

/// Configuration validator for library settings
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate a complete library configuration
    pub fn validate_config(config: &LibraryConfig) -> Result<()> {
        Self::validate_trash_grace_period(config.trash_grace_period_secs)?;
        Self::validate_hashing_concurrency(config.hashing_concurrency_limit)?;
        Self::validate_rescan_batch_size(config.rescan_batch_size)?;
        Self::validate_cleanup_threshold(config.cleanup_threshold_bytes)?;
        Ok(())
    }

    /// Validate trash grace period (must be positive)
    pub fn validate_trash_grace_period(secs: u64) -> Result<()> {
        if secs == 0 {
            return Err(anyhow!("Trash grace period must be greater than 0 seconds"));
        }
        if secs > 365 * 24 * 60 * 60 {
            return Err(anyhow!("Trash grace period cannot exceed 365 days"));
        }
        Ok(())
    }

    /// Validate hashing concurrency limit (must be positive and reasonable)
    pub fn validate_hashing_concurrency(limit: u32) -> Result<()> {
        if limit == 0 {
            return Err(anyhow!("Hashing concurrency limit must be at least 1"));
        }
        if limit > 16 {
            return Err(anyhow!("Hashing concurrency limit cannot exceed 16"));
        }
        Ok(())
    }

    /// Validate rescan batch size (must be positive and reasonable)
    pub fn validate_rescan_batch_size(size: u32) -> Result<()> {
        if size == 0 {
            return Err(anyhow!("Rescan batch size must be at least 1"));
        }
        if size > 10000 {
            return Err(anyhow!("Rescan batch size cannot exceed 10000"));
        }
        Ok(())
    }

    /// Validate cleanup threshold (must be positive)
    pub fn validate_cleanup_threshold(bytes: u64) -> Result<()> {
        if bytes == 0 {
            return Err(anyhow!("Cleanup threshold must be greater than 0 bytes"));
        }
        if bytes > 10 * 1024 * 1024 * 1024 * 1024 {
            // 10 TB max
            return Err(anyhow!("Cleanup threshold cannot exceed 10 TB"));
        }
        Ok(())
    }

    /// Validate trash grace period as u64
    pub fn validate_trash_grace_period_value(secs: u64) -> Result<()> {
        Self::validate_trash_grace_period(secs)
    }

    /// Validate hashing concurrency as u32
    pub fn validate_hashing_concurrency_value(limit: u32) -> Result<()> {
        Self::validate_hashing_concurrency(limit)
    }

    /// Validate rescan batch size as u32
    pub fn validate_rescan_batch_size_value(size: u32) -> Result<()> {
        Self::validate_rescan_batch_size(size)
    }

    /// Validate cleanup threshold as u64
    pub fn validate_cleanup_threshold_value(bytes: u64) -> Result<()> {
        Self::validate_cleanup_threshold(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_trash_grace_period_valid() {
        assert!(ConfigValidator::validate_trash_grace_period(86400).is_ok());
        assert!(ConfigValidator::validate_trash_grace_period(30 * 24 * 60 * 60).is_ok());
    }

    #[test]
    fn test_validate_trash_grace_period_zero() {
        assert!(ConfigValidator::validate_trash_grace_period(0).is_err());
    }

    #[test]
    fn test_validate_trash_grace_period_too_large() {
        assert!(ConfigValidator::validate_trash_grace_period(366 * 24 * 60 * 60).is_err());
    }

    #[test]
    fn test_validate_hashing_concurrency_valid() {
        assert!(ConfigValidator::validate_hashing_concurrency(1).is_ok());
        assert!(ConfigValidator::validate_hashing_concurrency(4).is_ok());
        assert!(ConfigValidator::validate_hashing_concurrency(16).is_ok());
    }

    #[test]
    fn test_validate_hashing_concurrency_zero() {
        assert!(ConfigValidator::validate_hashing_concurrency(0).is_err());
    }

    #[test]
    fn test_validate_hashing_concurrency_too_large() {
        assert!(ConfigValidator::validate_hashing_concurrency(17).is_err());
    }

    #[test]
    fn test_validate_rescan_batch_size_valid() {
        assert!(ConfigValidator::validate_rescan_batch_size(1).is_ok());
        assert!(ConfigValidator::validate_rescan_batch_size(100).is_ok());
        assert!(ConfigValidator::validate_rescan_batch_size(10000).is_ok());
    }

    #[test]
    fn test_validate_rescan_batch_size_zero() {
        assert!(ConfigValidator::validate_rescan_batch_size(0).is_err());
    }

    #[test]
    fn test_validate_rescan_batch_size_too_large() {
        assert!(ConfigValidator::validate_rescan_batch_size(10001).is_err());
    }

    #[test]
    fn test_validate_cleanup_threshold_valid() {
        assert!(ConfigValidator::validate_cleanup_threshold(1024).is_ok());
        assert!(ConfigValidator::validate_cleanup_threshold(10 * 1024 * 1024 * 1024).is_ok());
    }

    #[test]
    fn test_validate_cleanup_threshold_zero() {
        assert!(ConfigValidator::validate_cleanup_threshold(0).is_err());
    }

    #[test]
    fn test_validate_cleanup_threshold_too_large() {
        assert!(ConfigValidator::validate_cleanup_threshold(11 * 1024 * 1024 * 1024 * 1024).is_err());
    }

    #[test]
    fn test_validate_complete_config() {
        let config = LibraryConfig::new();
        assert!(ConfigValidator::validate_config(&config).is_ok());
    }
}
