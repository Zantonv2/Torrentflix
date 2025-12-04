use anyhow::{Result, Context};
use std::sync::Arc;
use tracing::{info, debug};

use crate::database::settings::SettingsDatabase;
use super::settings::{LibraryConfig, QualityPreference, VersionRetention};
use super::validation::ConfigValidator;

/// Manager for library configuration persistence and retrieval
pub struct LibraryConfigManager {
    settings_db: Arc<SettingsDatabase>,
}

impl LibraryConfigManager {
    /// Create a new library configuration manager
    pub fn new(settings_db: Arc<SettingsDatabase>) -> Self {
        Self { settings_db }
    }

    /// Load library configuration from database, or return defaults if not found
    pub async fn load_config(&self) -> Result<LibraryConfig> {
        debug!("Loading library configuration from database");

        let trash_grace_period = self.get_u64("trash_grace_period_secs")
            .await?
            .unwrap_or(30 * 24 * 60 * 60); // 30 days default

        let hashing_concurrency = self.get_u32("hashing_concurrency_limit")
            .await?
            .unwrap_or(1);

        let rescan_batch_size = self.get_u32("rescan_batch_size")
            .await?
            .unwrap_or(100);

        let cleanup_threshold = self.get_u64("cleanup_threshold_bytes")
            .await?
            .unwrap_or(10 * 1024 * 1024 * 1024); // 10 GB default

        let quality_pref = self.get_string("quality_preference")
            .await?
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(QualityPreference::Balanced);

        let version_retention = self.get_string("version_retention")
            .await?
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(VersionRetention::BestPlusBakup);

        let config = LibraryConfig {
            trash_grace_period_secs: trash_grace_period,
            hashing_concurrency_limit: hashing_concurrency,
            rescan_batch_size,
            cleanup_threshold_bytes: cleanup_threshold,
            quality_preference: quality_pref,
            version_retention,
        };

        info!("Library configuration loaded successfully");
        Ok(config)
    }

    /// Save library configuration to database with validation
    pub async fn save_config(&self, config: &LibraryConfig) -> Result<()> {
        debug!("Validating and saving library configuration to database");
        
        // Validate configuration before saving
        ConfigValidator::validate_config(config)?;

        let settings = vec![
            (
                "trash_grace_period_secs".to_string(),
                config.trash_grace_period_secs.to_string(),
                "u64".to_string(),
            ),
            (
                "hashing_concurrency_limit".to_string(),
                config.hashing_concurrency_limit.to_string(),
                "u32".to_string(),
            ),
            (
                "rescan_batch_size".to_string(),
                config.rescan_batch_size.to_string(),
                "u32".to_string(),
            ),
            (
                "cleanup_threshold_bytes".to_string(),
                config.cleanup_threshold_bytes.to_string(),
                "u64".to_string(),
            ),
            (
                "quality_preference".to_string(),
                serde_json::to_string(&config.quality_preference)?,
                "enum".to_string(),
            ),
            (
                "version_retention".to_string(),
                serde_json::to_string(&config.version_retention)?,
                "enum".to_string(),
            ),
        ];

        self.settings_db.set_many(settings).await?;
        info!("Library configuration saved successfully");
        Ok(())
    }

    /// Update a single configuration value with validation
    pub async fn update_trash_grace_period(&self, secs: u64) -> Result<()> {
        ConfigValidator::validate_trash_grace_period(secs)?;
        self.set_u64("trash_grace_period_secs", secs).await
    }

    /// Update hashing concurrency limit with validation
    pub async fn update_hashing_concurrency(&self, limit: u32) -> Result<()> {
        ConfigValidator::validate_hashing_concurrency(limit)?;
        self.set_u32("hashing_concurrency_limit", limit).await
    }

    /// Update rescan batch size with validation
    pub async fn update_rescan_batch_size(&self, size: u32) -> Result<()> {
        ConfigValidator::validate_rescan_batch_size(size)?;
        self.set_u32("rescan_batch_size", size).await
    }

    /// Update cleanup threshold with validation
    pub async fn update_cleanup_threshold(&self, bytes: u64) -> Result<()> {
        ConfigValidator::validate_cleanup_threshold(bytes)?;
        self.set_u64("cleanup_threshold_bytes", bytes).await
    }

    /// Update quality preference policy
    pub async fn update_quality_preference(&self, pref: QualityPreference) -> Result<()> {
        let value = serde_json::to_string(&pref)?;
        self.settings_db.set("quality_preference", &value, "enum").await
    }

    /// Update version retention policy
    pub async fn update_version_retention(&self, retention: VersionRetention) -> Result<()> {
        let value = serde_json::to_string(&retention)?;
        self.settings_db.set("version_retention", &value, "enum").await
    }

    /// Get a u64 setting value
    async fn get_u64(&self, key: &str) -> Result<Option<u64>> {
        if let Some(record) = self.settings_db.get(key).await? {
            let value = record.value.parse::<u64>()
                .context(format!("Failed to parse {} as u64", key))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Get a u32 setting value
    async fn get_u32(&self, key: &str) -> Result<Option<u32>> {
        if let Some(record) = self.settings_db.get(key).await? {
            let value = record.value.parse::<u32>()
                .context(format!("Failed to parse {} as u32", key))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Get a string setting value
    async fn get_string(&self, key: &str) -> Result<Option<String>> {
        if let Some(record) = self.settings_db.get(key).await? {
            Ok(Some(record.value))
        } else {
            Ok(None)
        }
    }

    /// Set a u64 value
    async fn set_u64(&self, key: &str, value: u64) -> Result<()> {
        self.settings_db.set(key, &value.to_string(), "u64").await
    }

    /// Set a u32 value
    async fn set_u32(&self, key: &str, value: u32) -> Result<()> {
        self.settings_db.set(key, &value.to_string(), "u32").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_config_defaults() {
        let config = LibraryConfig::new();
        assert_eq!(config.trash_grace_period_secs, 30 * 24 * 60 * 60);
        assert_eq!(config.hashing_concurrency_limit, 1);
        assert_eq!(config.rescan_batch_size, 100);
        assert_eq!(config.cleanup_threshold_bytes, 10 * 1024 * 1024 * 1024);
        assert_eq!(config.quality_preference, QualityPreference::Balanced);
        assert_eq!(config.version_retention, VersionRetention::BestPlusBakup);
    }

    #[test]
    fn test_quality_preference_serialization() {
        let pref = QualityPreference::HighestResolution;
        let json = serde_json::to_string(&pref).unwrap();
        let deserialized: QualityPreference = serde_json::from_str(&json).unwrap();
        assert_eq!(pref, deserialized);
    }

    #[test]
    fn test_version_retention_serialization() {
        let retention = VersionRetention::BestOnly;
        let json = serde_json::to_string(&retention).unwrap();
        let deserialized: VersionRetention = serde_json::from_str(&json).unwrap();
        assert_eq!(retention, deserialized);
    }
}
