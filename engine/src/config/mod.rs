// Configuration management module
// Handles library settings, validation, and persistence

pub mod loader;
pub mod settings;
pub mod migration;
pub mod library_config;
pub mod validation;

pub use loader::ConfigLoader;
pub use settings::{AppSettings, LibraryConfig, QualityPreference, VersionRetention};
pub use migration::ConfigMigration;
pub use library_config::LibraryConfigManager;
pub use validation::ConfigValidator;
