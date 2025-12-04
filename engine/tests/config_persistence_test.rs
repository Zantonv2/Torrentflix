use engine::config::{LibraryConfig, LibraryConfigManager, QualityPreference, VersionRetention, ConfigValidator};
use engine::database::SettingsDatabase;
use std::sync::Arc;

/// Helper to create a test database
async fn create_test_db() -> Arc<SettingsDatabase> {
    let db_url = "sqlite://:memory:";
    let database = engine::database::Database::new(db_url)
        .await
        .expect("Failed to create test database");
    
    let settings_db = Arc::new(SettingsDatabase::new(Arc::new(database)));
    settings_db.initialize().await.expect("Failed to initialize settings database");
    settings_db
}

#[tokio::test]
async fn test_config_persistence_save_and_load() {
    let settings_db = create_test_db().await;
    let config_manager = LibraryConfigManager::new(settings_db);

    // Create a custom configuration
    let mut config = LibraryConfig::new();
    config.trash_grace_period_secs = 7 * 24 * 60 * 60; // 7 days
    config.hashing_concurrency_limit = 2;
    config.rescan_batch_size = 50;
    config.cleanup_threshold_bytes = 5 * 1024 * 1024 * 1024; // 5 GB
    config.quality_preference = QualityPreference::HighestResolution;
    config.version_retention = VersionRetention::BestOnly;

    // Save configuration
    config_manager.save_config(&config).await.expect("Failed to save config");

    // Load configuration
    let loaded_config = config_manager.load_config().await.expect("Failed to load config");

    // Verify all values were persisted
    assert_eq!(loaded_config.trash_grace_period_secs, 7 * 24 * 60 * 60);
    assert_eq!(loaded_config.hashing_concurrency_limit, 2);
    assert_eq!(loaded_config.rescan_batch_size, 50);
    assert_eq!(loaded_config.cleanup_threshold_bytes, 5 * 1024 * 1024 * 1024);
    assert_eq!(loaded_config.quality_preference, QualityPreference::HighestResolution);
    assert_eq!(loaded_config.version_retention, VersionRetention::BestOnly);
}

#[tokio::test]
async fn test_config_persistence_defaults() {
    let settings_db = create_test_db().await;
    let config_manager = LibraryConfigManager::new(settings_db);

    // Load configuration without saving anything (should get defaults)
    let loaded_config = config_manager.load_config().await.expect("Failed to load config");

    // Verify defaults
    assert_eq!(loaded_config.trash_grace_period_secs, 30 * 24 * 60 * 60);
    assert_eq!(loaded_config.hashing_concurrency_limit, 1);
    assert_eq!(loaded_config.rescan_batch_size, 100);
    assert_eq!(loaded_config.cleanup_threshold_bytes, 10 * 1024 * 1024 * 1024);
    assert_eq!(loaded_config.quality_preference, QualityPreference::Balanced);
    assert_eq!(loaded_config.version_retention, VersionRetention::BestPlusBakup);
}

#[tokio::test]
async fn test_config_persistence_update_individual_values() {
    let settings_db = create_test_db().await;
    let config_manager = LibraryConfigManager::new(settings_db);

    // Update individual values
    config_manager.update_trash_grace_period(14 * 24 * 60 * 60).await.expect("Failed to update trash grace period");
    config_manager.update_hashing_concurrency(4).await.expect("Failed to update hashing concurrency");
    config_manager.update_rescan_batch_size(200).await.expect("Failed to update rescan batch size");
    config_manager.update_cleanup_threshold(20 * 1024 * 1024 * 1024).await.expect("Failed to update cleanup threshold");

    // Load and verify
    let loaded_config = config_manager.load_config().await.expect("Failed to load config");
    assert_eq!(loaded_config.trash_grace_period_secs, 14 * 24 * 60 * 60);
    assert_eq!(loaded_config.hashing_concurrency_limit, 4);
    assert_eq!(loaded_config.rescan_batch_size, 200);
    assert_eq!(loaded_config.cleanup_threshold_bytes, 20 * 1024 * 1024 * 1024);
}

#[tokio::test]
async fn test_config_persistence_validation_on_save() {
    let settings_db = create_test_db().await;
    let config_manager = LibraryConfigManager::new(settings_db);

    // Create an invalid configuration
    let mut config = LibraryConfig::new();
    config.trash_grace_period_secs = 0; // Invalid: must be > 0

    // Attempt to save should fail
    let result = config_manager.save_config(&config).await;
    assert!(result.is_err(), "Should reject invalid trash grace period");
}

#[tokio::test]
async fn test_config_persistence_validation_on_update() {
    let settings_db = create_test_db().await;
    let config_manager = LibraryConfigManager::new(settings_db);

    // Attempt to update with invalid value
    let result = config_manager.update_hashing_concurrency(0).await;
    assert!(result.is_err(), "Should reject hashing concurrency of 0");

    let result = config_manager.update_hashing_concurrency(17).await;
    assert!(result.is_err(), "Should reject hashing concurrency > 16");

    let result = config_manager.update_rescan_batch_size(0).await;
    assert!(result.is_err(), "Should reject rescan batch size of 0");

    let result = config_manager.update_cleanup_threshold(0).await;
    assert!(result.is_err(), "Should reject cleanup threshold of 0");
}

#[tokio::test]
async fn test_config_persistence_quality_preference_update() {
    let settings_db = create_test_db().await;
    let config_manager = LibraryConfigManager::new(settings_db);

    // Update quality preference
    config_manager.update_quality_preference(QualityPreference::SmallestSize).await.expect("Failed to update quality preference");

    // Load and verify
    let loaded_config = config_manager.load_config().await.expect("Failed to load config");
    assert_eq!(loaded_config.quality_preference, QualityPreference::SmallestSize);
}

#[tokio::test]
async fn test_config_persistence_version_retention_update() {
    let settings_db = create_test_db().await;
    let config_manager = LibraryConfigManager::new(settings_db);

    // Update version retention
    config_manager.update_version_retention(VersionRetention::KeepAll).await.expect("Failed to update version retention");

    // Load and verify
    let loaded_config = config_manager.load_config().await.expect("Failed to load config");
    assert_eq!(loaded_config.version_retention, VersionRetention::KeepAll);
}

#[tokio::test]
async fn test_config_persistence_multiple_updates() {
    let settings_db = create_test_db().await;
    let config_manager = LibraryConfigManager::new(settings_db);

    // Perform multiple updates
    config_manager.update_trash_grace_period(3 * 24 * 60 * 60).await.expect("Failed to update");
    config_manager.update_hashing_concurrency(2).await.expect("Failed to update");
    config_manager.update_rescan_batch_size(75).await.expect("Failed to update");
    config_manager.update_cleanup_threshold(15 * 1024 * 1024 * 1024).await.expect("Failed to update");
    config_manager.update_quality_preference(QualityPreference::PreferCodec).await.expect("Failed to update");
    config_manager.update_version_retention(VersionRetention::BestOnly).await.expect("Failed to update");

    // Load and verify all updates persisted
    let loaded_config = config_manager.load_config().await.expect("Failed to load config");
    assert_eq!(loaded_config.trash_grace_period_secs, 3 * 24 * 60 * 60);
    assert_eq!(loaded_config.hashing_concurrency_limit, 2);
    assert_eq!(loaded_config.rescan_batch_size, 75);
    assert_eq!(loaded_config.cleanup_threshold_bytes, 15 * 1024 * 1024 * 1024);
    assert_eq!(loaded_config.quality_preference, QualityPreference::PreferCodec);
    assert_eq!(loaded_config.version_retention, VersionRetention::BestOnly);
}

#[test]
fn test_config_validator_all_valid_values() {
    let config = LibraryConfig::new();
    assert!(ConfigValidator::validate_config(&config).is_ok());
}

#[test]
fn test_config_validator_trash_grace_period_boundaries() {
    // Valid: 1 second
    assert!(ConfigValidator::validate_trash_grace_period(1).is_ok());
    
    // Valid: 365 days
    assert!(ConfigValidator::validate_trash_grace_period(365 * 24 * 60 * 60).is_ok());
    
    // Invalid: 0 seconds
    assert!(ConfigValidator::validate_trash_grace_period(0).is_err());
    
    // Invalid: > 365 days
    assert!(ConfigValidator::validate_trash_grace_period(366 * 24 * 60 * 60).is_err());
}

#[test]
fn test_config_validator_hashing_concurrency_boundaries() {
    // Valid: 1
    assert!(ConfigValidator::validate_hashing_concurrency(1).is_ok());
    
    // Valid: 16
    assert!(ConfigValidator::validate_hashing_concurrency(16).is_ok());
    
    // Invalid: 0
    assert!(ConfigValidator::validate_hashing_concurrency(0).is_err());
    
    // Invalid: > 16
    assert!(ConfigValidator::validate_hashing_concurrency(17).is_err());
}

#[test]
fn test_config_validator_rescan_batch_size_boundaries() {
    // Valid: 1
    assert!(ConfigValidator::validate_rescan_batch_size(1).is_ok());
    
    // Valid: 10000
    assert!(ConfigValidator::validate_rescan_batch_size(10000).is_ok());
    
    // Invalid: 0
    assert!(ConfigValidator::validate_rescan_batch_size(0).is_err());
    
    // Invalid: > 10000
    assert!(ConfigValidator::validate_rescan_batch_size(10001).is_err());
}

#[test]
fn test_config_validator_cleanup_threshold_boundaries() {
    // Valid: 1 byte
    assert!(ConfigValidator::validate_cleanup_threshold(1).is_ok());
    
    // Valid: 10 TB
    assert!(ConfigValidator::validate_cleanup_threshold(10 * 1024 * 1024 * 1024 * 1024).is_ok());
    
    // Invalid: 0 bytes
    assert!(ConfigValidator::validate_cleanup_threshold(0).is_err());
    
    // Invalid: > 10 TB
    assert!(ConfigValidator::validate_cleanup_threshold(11 * 1024 * 1024 * 1024 * 1024).is_err());
}
