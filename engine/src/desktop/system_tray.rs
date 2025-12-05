/// System tray integration for Linux desktop environments
/// Provides quick access menu and job status display

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};
use crate::desktop::{DesktopError, DesktopResult};

/// Represents the current status to display in the system tray
#[derive(Debug, Clone)]
pub struct TrayStatus {
    pub is_running: bool,
    pub active_jobs: usize,
    pub status_message: String,
}

impl Default for TrayStatus {
    fn default() -> Self {
        Self {
            is_running: false,
            active_jobs: 0,
            status_message: "Ready".to_string(),
        }
    }
}

/// Manages system tray integration
pub struct SystemTrayManager {
    status: Arc<Mutex<TrayStatus>>,
    #[allow(dead_code)]
    icon_path: Option<String>,
}

impl SystemTrayManager {
    /// Create a new system tray manager
    pub fn new(icon_path: Option<String>) -> Self {
        Self {
            status: Arc::new(Mutex::new(TrayStatus::default())),
            icon_path,
        }
    }

    /// Initialize the system tray
    pub async fn initialize(&self) -> DesktopResult<()> {
        info!("Initializing system tray");

        // Check if we're in a desktop environment that supports system tray
        if !self.is_tray_supported().await {
            warn!("System tray not supported in this environment");
            return Err(DesktopError::SystemTrayError(
                "System tray not supported".to_string(),
            ));
        }

        info!("System tray initialized successfully");
        Ok(())
    }

    /// Check if the system tray is supported
    async fn is_tray_supported(&self) -> bool {
        // Check for common desktop environment variables
        let desktop_env = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
        let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();

        // Support for common Linux desktop environments
        let supported_desktops = ["GNOME", "KDE", "XFCE", "LXDE", "Cinnamon", "MATE"];
        let is_desktop_supported = supported_desktops
            .iter()
            .any(|&d| desktop_env.contains(d));

        // Also check if we have a display server
        let has_display = std::env::var("DISPLAY").is_ok() || session_type == "wayland";

        is_desktop_supported && has_display
    }

    /// Update the tray status
    pub async fn update_status(&self, status: TrayStatus) -> DesktopResult<()> {
        let mut current_status = self.status.lock().await;
        *current_status = status;
        Ok(())
    }

    /// Get the current tray status
    pub async fn get_status(&self) -> TrayStatus {
        self.status.lock().await.clone()
    }

    /// Update job count in tray
    pub async fn set_active_jobs(&self, count: usize) -> DesktopResult<()> {
        let mut status = self.status.lock().await;
        status.active_jobs = count;
        
        if count > 0 {
            status.status_message = format!("{} job(s) running", count);
        } else {
            status.status_message = "Ready".to_string();
        }
        
        Ok(())
    }

    /// Set the running state
    pub async fn set_running(&self, running: bool) -> DesktopResult<()> {
        let mut status = self.status.lock().await;
        status.is_running = running;
        status.status_message = if running {
            "Running".to_string()
        } else {
            "Idle".to_string()
        };
        Ok(())
    }

    /// Get the tooltip text for the tray icon
    pub async fn get_tooltip(&self) -> String {
        let status = self.status.lock().await;
        format!(
            "TorrentFlix\n{}\nActive jobs: {}",
            status.status_message, status.active_jobs
        )
    }

    /// Get the menu items for the tray context menu
    pub fn get_menu_items(&self) -> Vec<TrayMenuItem> {
        vec![
            TrayMenuItem {
                label: "Show Library".to_string(),
                action: "show_library".to_string(),
            },
            TrayMenuItem {
                label: "Start Rescan".to_string(),
                action: "start_rescan".to_string(),
            },
            TrayMenuItem {
                label: "Check Integrity".to_string(),
                action: "check_integrity".to_string(),
            },
            TrayMenuItem {
                label: "Settings".to_string(),
                action: "open_settings".to_string(),
            },
            TrayMenuItem {
                label: "Quit".to_string(),
                action: "quit".to_string(),
            },
        ]
    }

    /// Handle a menu item click
    pub async fn handle_menu_click(&self, action: &str) -> DesktopResult<()> {
        info!("Tray menu action: {}", action);
        // This would be handled by the Tauri app
        Ok(())
    }
}

/// Represents a menu item in the tray context menu
#[derive(Debug, Clone)]
pub struct TrayMenuItem {
    pub label: String,
    pub action: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_tray_manager_creation() {
        let manager = SystemTrayManager::new(None);
        let status = manager.get_status().await;
        assert!(!status.is_running);
        assert_eq!(status.active_jobs, 0);
    }

    #[tokio::test]
    async fn test_update_status() {
        let manager = SystemTrayManager::new(None);
        let new_status = TrayStatus {
            is_running: true,
            active_jobs: 2,
            status_message: "Processing".to_string(),
        };

        manager.update_status(new_status.clone()).await.unwrap();
        let status = manager.get_status().await;
        assert!(status.is_running);
        assert_eq!(status.active_jobs, 2);
    }

    #[tokio::test]
    async fn test_set_active_jobs() {
        let manager = SystemTrayManager::new(None);
        manager.set_active_jobs(3).await.unwrap();
        let status = manager.get_status().await;
        assert_eq!(status.active_jobs, 3);
        assert!(status.status_message.contains("3 job(s)"));
    }

    #[tokio::test]
    async fn test_get_tooltip() {
        let manager = SystemTrayManager::new(None);
        manager.set_active_jobs(1).await.unwrap();
        let tooltip = manager.get_tooltip().await;
        assert!(tooltip.contains("TorrentFlix"));
        assert!(tooltip.contains("Active jobs: 1"));
    }

    #[test]
    fn test_get_menu_items() {
        let manager = SystemTrayManager::new(None);
        let items = manager.get_menu_items();
        assert!(!items.is_empty());
        assert!(items.iter().any(|item| item.action == "show_library"));
        assert!(items.iter().any(|item| item.action == "quit"));
    }
}
