// Placeholder module for configuration management
// TODO: Implement config loader and migrations

pub mod loader;
pub mod settings;
pub mod migration;

pub use loader::ConfigLoader;
pub use settings::AppSettings;
pub use migration::ConfigMigration;
