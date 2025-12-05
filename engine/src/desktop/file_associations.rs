/// File association and MIME type registration for Linux desktop environments
/// Registers the application with the desktop environment for file handling

use std::path::PathBuf;
use std::fs;
use tracing::{info, warn};
use crate::desktop::{DesktopError, DesktopResult};

/// Manages file associations and MIME type registration
pub struct FileAssociationManager {
    app_name: String,
    app_exec: PathBuf,
    app_icon: Option<PathBuf>,
    supported_extensions: Vec<String>,
}

impl FileAssociationManager {
    /// Create a new file association manager
    pub fn new(
        app_name: String,
        app_exec: PathBuf,
        app_icon: Option<PathBuf>,
    ) -> Self {
        Self {
            app_name,
            app_exec,
            app_icon,
            supported_extensions: vec![
                "mkv".to_string(),
                "mp4".to_string(),
                "avi".to_string(),
                "mov".to_string(),
                "flv".to_string(),
                "wmv".to_string(),
                "webm".to_string(),
                "m4v".to_string(),
                "ts".to_string(),
                "m2ts".to_string(),
            ],
        }
    }

    /// Register the application with the desktop environment
    /// This creates a .desktop file and registers MIME types
    pub async fn register_with_desktop(&self) -> DesktopResult<()> {
        info!("Registering {} with desktop environment", self.app_name);

        // Create desktop file
        self.create_desktop_file().await?;

        // Register MIME types
        self.register_mime_types().await?;

        // Update desktop database
        self.update_desktop_database().await?;

        info!("Successfully registered {} with desktop environment", self.app_name);
        Ok(())
    }

    /// Create a .desktop file for the application
    async fn create_desktop_file(&self) -> DesktopResult<()> {
        let desktop_dir = self.get_desktop_dir()?;
        fs::create_dir_all(&desktop_dir)
            .map_err(|e| DesktopError::DesktopFileError(format!("Failed to create desktop dir: {}", e)))?;

        let desktop_file = desktop_dir.join(format!("{}.desktop", self.app_name.to_lowercase()));

        // Build MIME types string
        let mime_types = self.supported_extensions
            .iter()
            .map(|ext| format!("video/{}", ext))
            .collect::<Vec<_>>()
            .join(";");

        // Build icon path
        let icon_str = self.app_icon
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "application-x-executable".to_string());

        let desktop_content = format!(
            "[Desktop Entry]\n\
            Version=1.0\n\
            Type=Application\n\
            Name={}\n\
            Comment=Media library management and playback\n\
            Exec={} %F\n\
            Icon={}\n\
            Terminal=false\n\
            Categories=Video;AudioVideo;Utility;\n\
            MimeType={}\n\
            StartupNotify=true\n",
            self.app_name,
            self.app_exec.to_string_lossy(),
            icon_str,
            mime_types
        );

        fs::write(&desktop_file, desktop_content)
            .map_err(|e| DesktopError::DesktopFileError(format!("Failed to write desktop file: {}", e)))?;

        info!("Created desktop file at: {}", desktop_file.display());
        Ok(())
    }

    /// Register MIME types for supported video formats
    async fn register_mime_types(&self) -> DesktopResult<()> {
        // Build MIME types XML
        let mime_types_xml = self.build_mime_types_xml();

        let mime_dir = self.get_mime_dir()?;
        fs::create_dir_all(&mime_dir)
            .map_err(|e| DesktopError::MimeTypeError(format!("Failed to create MIME dir: {}", e)))?;

        let mime_file = mime_dir.join("application-x-moviedownloader.xml");
        fs::write(&mime_file, mime_types_xml)
            .map_err(|e| DesktopError::MimeTypeError(format!("Failed to write MIME file: {}", e)))?;

        info!("Registered MIME types at: {}", mime_file.display());
        Ok(())
    }

    /// Update the desktop database to recognize new MIME types
    async fn update_desktop_database(&self) -> DesktopResult<()> {
        // Try to run update-mime-database if available
        match std::process::Command::new("update-mime-database")
            .arg(self.get_mime_dir()?)
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    info!("Successfully updated MIME database");
                } else {
                    warn!("update-mime-database returned non-zero status");
                }
            }
            Err(e) => {
                warn!("update-mime-database not available: {}", e);
                // This is not fatal - the system may still recognize the MIME types
            }
        }

        // Try to run update-desktop-database if available
        match std::process::Command::new("update-desktop-database")
            .arg(self.get_desktop_dir()?)
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    info!("Successfully updated desktop database");
                } else {
                    warn!("update-desktop-database returned non-zero status");
                }
            }
            Err(e) => {
                warn!("update-desktop-database not available: {}", e);
                // This is not fatal - the system may still recognize the desktop file
            }
        }

        Ok(())
    }

    /// Build XML content for MIME type registration
    fn build_mime_types_xml(&self) -> String {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<mime-info xmlns=\"http://www.freedesktop.org/standards/shared-mime-info\">\n");

        for ext in &self.supported_extensions {
            xml.push_str(&format!(
                "  <mime-type type=\"video/{}\">\n\
                 <comment>Video file ({})</comment>\n\
                 <glob pattern=\"*.{}\"/>\n\
                 </mime-type>\n",
                ext, ext, ext
            ));
        }

        xml.push_str("</mime-info>\n");
        xml
    }

    /// Get the desktop files directory
    fn get_desktop_dir(&self) -> DesktopResult<PathBuf> {
        let home = std::env::var("HOME")
            .map_err(|_| DesktopError::ConfigError("HOME environment variable not set".to_string()))?;
        Ok(PathBuf::from(home).join(".local/share/applications"))
    }

    /// Get the MIME types directory
    fn get_mime_dir(&self) -> DesktopResult<PathBuf> {
        let home = std::env::var("HOME")
            .map_err(|_| DesktopError::ConfigError("HOME environment variable not set".to_string()))?;
        Ok(PathBuf::from(home).join(".local/share/mime/packages"))
    }

    /// Check if the application is registered with the desktop environment
    pub async fn is_registered(&self) -> bool {
        match self.get_desktop_dir() {
            Ok(desktop_dir) => {
                let desktop_file = desktop_dir.join(format!("{}.desktop", self.app_name.to_lowercase()));
                desktop_file.exists()
            }
            Err(_) => false,
        }
    }

    /// Unregister the application from the desktop environment
    pub async fn unregister_from_desktop(&self) -> DesktopResult<()> {
        info!("Unregistering {} from desktop environment", self.app_name);

        // Remove desktop file
        if let Ok(desktop_dir) = self.get_desktop_dir() {
            let desktop_file = desktop_dir.join(format!("{}.desktop", self.app_name.to_lowercase()));
            if desktop_file.exists() {
                fs::remove_file(&desktop_file)
                    .map_err(|e| DesktopError::DesktopFileError(format!("Failed to remove desktop file: {}", e)))?;
                info!("Removed desktop file: {}", desktop_file.display());
            }
        }

        // Remove MIME type file
        if let Ok(mime_dir) = self.get_mime_dir() {
            let mime_file = mime_dir.join("application-x-moviedownloader.xml");
            if mime_file.exists() {
                fs::remove_file(&mime_file)
                    .map_err(|e| DesktopError::MimeTypeError(format!("Failed to remove MIME file: {}", e)))?;
                info!("Removed MIME file: {}", mime_file.display());
            }
        }

        // Update desktop database
        self.update_desktop_database().await?;

        info!("Successfully unregistered {} from desktop environment", self.app_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_association_manager_creation() {
        let manager = FileAssociationManager::new(
            "MovieDownloader".to_string(),
            PathBuf::from("/usr/bin/moviedownloader"),
            None,
        );
        assert_eq!(manager.app_name, "MovieDownloader");
        assert!(!manager.supported_extensions.is_empty());
    }

    #[test]
    fn test_mime_types_xml_generation() {
        let manager = FileAssociationManager::new(
            "MovieDownloader".to_string(),
            PathBuf::from("/usr/bin/moviedownloader"),
            None,
        );
        let xml = manager.build_mime_types_xml();
        assert!(xml.contains("<?xml version"));
        assert!(xml.contains("mime-info"));
        assert!(xml.contains("video/mkv"));
    }
}
