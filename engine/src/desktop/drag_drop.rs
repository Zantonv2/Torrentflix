/// Drag-and-drop support for importing files
/// Handles file drops from file managers and other applications

use std::path::PathBuf;
use tracing::{info, warn};
use crate::desktop::{DesktopError, DesktopResult};

/// Handles drag-and-drop operations
pub struct DragDropHandler;

impl DragDropHandler {
    /// Process dropped files for import
    /// Validates files and returns paths ready for import
    pub async fn process_dropped_files(
        files: Vec<PathBuf>,
    ) -> DesktopResult<Vec<PathBuf>> {
        info!("Processing {} dropped files", files.len());

        let mut valid_files = Vec::new();

        for file_path in files {
            match Self::validate_dropped_file(&file_path).await {
                Ok(true) => {
                    info!("Valid file dropped: {}", file_path.display());
                    valid_files.push(file_path);
                }
                Ok(false) => {
                    warn!("Dropped file is not a valid media file: {}", file_path.display());
                }
                Err(e) => {
                    warn!("Error validating dropped file: {}", e);
                }
            }
        }

        if valid_files.is_empty() {
            return Err(DesktopError::ConfigError(
                "No valid media files were dropped".to_string(),
            ));
        }

        info!("Accepted {} valid files from drag-and-drop", valid_files.len());
        Ok(valid_files)
    }

    /// Validate a dropped file
    /// Checks if the file exists and has a supported media extension
    async fn validate_dropped_file(path: &PathBuf) -> DesktopResult<bool> {
        // Check if file exists
        if !path.exists() {
            return Ok(false);
        }

        // Check if it's a file (not a directory)
        if !path.is_file() {
            return Ok(false);
        }

        // Check file extension
        let supported_extensions = vec![
            "mkv", "mp4", "avi", "mov", "flv", "wmv", "webm", "m4v", "ts", "m2ts",
        ];

        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return Ok(supported_extensions.contains(&ext_str.to_lowercase().as_str()));
            }
        }

        Ok(false)
    }

    /// Check if a file is a directory (for recursive import)
    pub async fn is_directory(path: &PathBuf) -> bool {
        path.is_dir()
    }

    /// Get all media files from a directory recursively
    pub async fn get_media_files_from_directory(
        dir: &PathBuf,
    ) -> DesktopResult<Vec<PathBuf>> {
        info!("Scanning directory for media files: {}", dir.display());

        let mut media_files = Vec::new();
        let supported_extensions = vec![
            "mkv", "mp4", "avi", "mov", "flv", "wmv", "webm", "m4v", "ts", "m2ts",
        ];

        Self::scan_directory_recursive(dir, &mut media_files, &supported_extensions)?;

        info!("Found {} media files in directory", media_files.len());
        Ok(media_files)
    }

    /// Recursively scan directory for media files
    fn scan_directory_recursive(
        dir: &PathBuf,
        results: &mut Vec<PathBuf>,
        supported_extensions: &[&str],
    ) -> DesktopResult<()> {
        match std::fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            if path.is_dir() {
                                // Recursively scan subdirectories
                                let _ = Self::scan_directory_recursive(&path, results, supported_extensions);
                            } else if path.is_file() {
                                // Check if file has supported extension
                                if let Some(ext) = path.extension() {
                                    if let Some(ext_str) = ext.to_str() {
                                        if supported_extensions.contains(&ext_str.to_lowercase().as_str()) {
                                            results.push(path);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Error reading directory entry: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                return Err(DesktopError::IoError(e));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_validate_dropped_file_with_valid_extension() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.mkv");
        fs::write(&file_path, "test").unwrap();

        let result = DragDropHandler::validate_dropped_file(&file_path).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_validate_dropped_file_with_invalid_extension() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        let result = DragDropHandler::validate_dropped_file(&file_path).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_validate_dropped_file_nonexistent() {
        let file_path = PathBuf::from("/nonexistent/file.mkv");
        let result = DragDropHandler::validate_dropped_file(&file_path).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_is_directory() {
        let temp_dir = TempDir::new().unwrap();
        let result = DragDropHandler::is_directory(&temp_dir.path().to_path_buf()).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_get_media_files_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create some test files
        fs::write(temp_dir.path().join("video1.mkv"), "test").unwrap();
        fs::write(temp_dir.path().join("video2.mp4"), "test").unwrap();
        fs::write(temp_dir.path().join("document.txt"), "test").unwrap();

        let result = DragDropHandler::get_media_files_from_directory(&temp_dir.path().to_path_buf()).await;
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 2);
    }
}
