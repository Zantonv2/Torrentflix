// POSIX path handling for Linux filesystem compatibility
// Handles Linux path separators, hidden files, and case-sensitive filenames

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// POSIX path handler for Linux filesystem operations
pub struct PosixPathHandler;

impl PosixPathHandler {
    /// Create a new POSIX path handler
    pub fn new() -> Self {
        Self
    }

    /// Normalize a path to use Linux path separators
    /// Ensures forward slashes are used consistently
    pub fn normalize_path(&self, path: &Path) -> Result<PathBuf> {
        // On Linux, Path already uses forward slashes, but we validate the path
        let path_str = path
            .to_str()
            .context("Path contains invalid UTF-8 characters")?;

        // Ensure no backslashes (Windows-style) are present
        if path_str.contains('\\') {
            anyhow::bail!("Path contains Windows-style backslashes: {}", path_str);
        }

        Ok(path.to_path_buf())
    }

    /// Check if a filename is hidden (dot-prefixed)
    /// In Linux, files starting with '.' are hidden
    pub fn is_hidden(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
    }

    /// Get the filename from a path, handling hidden files correctly
    pub fn get_filename(&self, path: &Path) -> Option<String> {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    }

    /// Check if two paths are equal, respecting case sensitivity
    /// Linux filesystems are case-sensitive by default
    pub fn paths_equal(&self, path1: &Path, path2: &Path) -> bool {
        // Direct comparison - case-sensitive
        path1 == path2
    }

    /// Validate that a path is a valid POSIX path
    /// Checks for invalid characters and proper formatting
    pub fn validate_path(&self, path: &Path) -> Result<()> {
        let path_str = path
            .to_str()
            .context("Path contains invalid UTF-8 characters")?;

        // Check for null bytes (invalid in POSIX paths)
        if path_str.contains('\0') {
            anyhow::bail!("Path contains null bytes");
        }

        // Check for Windows-style backslashes
        if path_str.contains('\\') {
            anyhow::bail!("Path contains Windows-style backslashes");
        }

        Ok(())
    }

    /// Join path components using POSIX separators
    pub fn join_paths(&self, base: &Path, relative: &Path) -> Result<PathBuf> {
        self.validate_path(base)?;
        self.validate_path(relative)?;

        Ok(base.join(relative))
    }

    /// Split a path into components, handling hidden files
    pub fn split_path(&self, path: &Path) -> Vec<String> {
        path.components()
            .filter_map(|comp| {
                comp.as_os_str()
                    .to_str()
                    .map(|s| s.to_string())
            })
            .collect()
    }

    /// Check if a path is absolute (starts with '/')
    pub fn is_absolute(&self, path: &Path) -> bool {
        path.is_absolute()
    }

    /// Convert a relative path to absolute, handling '.' and '..' correctly
    pub fn canonicalize(&self, path: &Path) -> Result<PathBuf> {
        path.canonicalize()
            .with_context(|| format!("Failed to canonicalize path: {}", path.display()))
    }

    /// Get the parent directory of a path
    pub fn parent(&self, path: &Path) -> Option<PathBuf> {
        path.parent().map(|p| p.to_path_buf())
    }

    /// Check if a path contains any hidden components
    pub fn contains_hidden_components(&self, path: &Path) -> bool {
        path.components().any(|comp| {
            comp.as_os_str()
                .to_str()
                .map(|s| s.starts_with('.') && s != "." && s != "..")
                .unwrap_or(false)
        })
    }

    /// Sanitize a filename for use in a path
    /// Removes or replaces characters that might cause issues
    pub fn sanitize_filename(&self, filename: &str) -> String {
        filename
            .chars()
            .map(|c| match c {
                '\0' => '_',  // Null byte
                '/' => '_',   // Path separator
                _ => c,
            })
            .collect()
    }

    /// Check if a path is case-sensitively different from another
    /// Returns true if paths differ only in case
    pub fn differs_only_in_case(&self, path1: &Path, path2: &Path) -> bool {
        let str1 = path1.to_str().unwrap_or("");
        let str2 = path2.to_str().unwrap_or("");
        
        str1.to_lowercase() == str2.to_lowercase() && str1 != str2
    }
}

impl Default for PosixPathHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_normalize_path() {
        let handler = PosixPathHandler::new();
        
        let path = PathBuf::from("/home/user/file.txt");
        let normalized = handler.normalize_path(&path).unwrap();
        assert_eq!(normalized, path);
    }

    #[test]
    fn test_is_hidden() {
        let handler = PosixPathHandler::new();
        
        assert!(handler.is_hidden(&PathBuf::from(".hidden")));
        assert!(handler.is_hidden(&PathBuf::from("/path/to/.hidden")));
        assert!(!handler.is_hidden(&PathBuf::from("visible")));
        assert!(!handler.is_hidden(&PathBuf::from("/path/to/visible")));
    }

    #[test]
    fn test_paths_equal_case_sensitive() {
        let handler = PosixPathHandler::new();
        
        let path1 = PathBuf::from("/home/user/File.txt");
        let path2 = PathBuf::from("/home/user/file.txt");
        
        // Should be different due to case sensitivity
        assert!(!handler.paths_equal(&path1, &path2));
        
        let path3 = PathBuf::from("/home/user/File.txt");
        assert!(handler.paths_equal(&path1, &path3));
    }

    #[test]
    fn test_validate_path() {
        let handler = PosixPathHandler::new();
        
        // Valid paths
        assert!(handler.validate_path(&PathBuf::from("/home/user/file.txt")).is_ok());
        assert!(handler.validate_path(&PathBuf::from("relative/path")).is_ok());
        assert!(handler.validate_path(&PathBuf::from(".hidden/file")).is_ok());
    }

    #[test]
    fn test_contains_hidden_components() {
        let handler = PosixPathHandler::new();
        
        assert!(handler.contains_hidden_components(&PathBuf::from("/home/.config/file")));
        assert!(handler.contains_hidden_components(&PathBuf::from(".hidden/file")));
        assert!(!handler.contains_hidden_components(&PathBuf::from("/home/user/file")));
        assert!(!handler.contains_hidden_components(&PathBuf::from("./relative/path"))); // . and .. don't count
    }

    #[test]
    fn test_sanitize_filename() {
        let handler = PosixPathHandler::new();
        
        assert_eq!(handler.sanitize_filename("normal.txt"), "normal.txt");
        assert_eq!(handler.sanitize_filename("file/with/slashes"), "file_with_slashes");
        assert_eq!(handler.sanitize_filename("file\0with\0nulls"), "file_with_nulls");
    }

    #[test]
    fn test_differs_only_in_case() {
        let handler = PosixPathHandler::new();
        
        let path1 = PathBuf::from("/home/user/File.txt");
        let path2 = PathBuf::from("/home/user/file.txt");
        
        assert!(handler.differs_only_in_case(&path1, &path2));
        assert!(!handler.differs_only_in_case(&path1, &path1));
    }
}
