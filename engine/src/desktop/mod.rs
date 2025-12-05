/// Desktop integration module for Linux desktop environments
/// Handles file associations, MIME types, system tray, and power management

pub mod file_associations;
pub mod drag_drop;
pub mod system_tray;
pub mod power_management;

pub use file_associations::FileAssociationManager;
pub use drag_drop::DragDropHandler;
pub use system_tray::SystemTrayManager;
pub use power_management::PowerManagementHandler;

/// Desktop integration error types
#[derive(Debug, thiserror::Error)]
pub enum DesktopError {
    #[error("Failed to register file association: {0}")]
    FileAssociationFailed(String),
    
    #[error("Failed to create desktop file: {0}")]
    DesktopFileError(String),
    
    #[error("Failed to register MIME type: {0}")]
    MimeTypeError(String),
    
    #[error("Failed to initialize system tray: {0}")]
    SystemTrayError(String),
    
    #[error("Failed to initialize power management: {0}")]
    PowerManagementError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type DesktopResult<T> = Result<T, DesktopError>;
