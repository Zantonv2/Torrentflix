// Atomic file operations for Linux filesystem
// Handles atomic moves within same filesystem and cross-mount operations

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::os::unix::fs::MetadataExt;
use tokio::fs;

/// Atomic file mover that handles same-filesystem and cross-mount operations
pub struct AtomicMover;

impl AtomicMover {
    pub fn new() -> Self {
        Self
    }

    /// Perform an atomic move operation
    /// Uses rename() for same-filesystem moves, copy-then-delete for cross-mount
    pub async fn atomic_move(&self, from: &Path, to: &Path) -> Result<()> {
        // Check if source and destination are on the same filesystem
        if self.same_filesystem(from, to).await? {
            // Use atomic rename for same-filesystem moves
            self.atomic_rename(from, to).await
        } else {
            // Use copy-then-delete for cross-mount moves
            self.copy_then_delete(from, to).await
        }
    }

    /// Atomic rename within the same filesystem
    /// This is truly atomic on POSIX systems
    async fn atomic_rename(&self, from: &Path, to: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
        }

        // Perform atomic rename
        fs::rename(from, to)
            .await
            .with_context(|| format!("Failed to rename {} to {}", from.display(), to.display()))?;

        Ok(())
    }

    /// Copy file then delete original (for cross-mount moves)
    /// Not atomic, but ensures data integrity
    async fn copy_then_delete(&self, from: &Path, to: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
        }

        // Copy the file
        fs::copy(from, to)
            .await
            .with_context(|| format!("Failed to copy {} to {}", from.display(), to.display()))?;

        // Verify the copy succeeded by checking file sizes
        let from_metadata = fs::metadata(from).await?;
        let to_metadata = fs::metadata(to).await?;
        
        if from_metadata.len() != to_metadata.len() {
            // Copy verification failed, remove the incomplete copy
            let _ = fs::remove_file(to).await;
            anyhow::bail!("Copy verification failed: file sizes don't match");
        }

        // Delete the original file
        fs::remove_file(from)
            .await
            .with_context(|| format!("Failed to remove original file: {}", from.display()))?;

        Ok(())
    }

    /// Check if two paths are on the same filesystem
    /// Compares device IDs from filesystem metadata
    pub async fn same_filesystem(&self, path1: &Path, path2: &Path) -> Result<bool> {
        // Get the parent directory for the destination if it doesn't exist yet
        let path2_check = if path2.exists() {
            path2.to_path_buf()
        } else {
            path2.parent()
                .ok_or_else(|| anyhow::anyhow!("Path has no parent: {}", path2.display()))?
                .to_path_buf()
        };

        let metadata1 = fs::metadata(path1)
            .await
            .with_context(|| format!("Failed to get metadata for: {}", path1.display()))?;
        
        let metadata2 = fs::metadata(&path2_check)
            .await
            .with_context(|| format!("Failed to get metadata for: {}", path2_check.display()))?;

        // Compare device IDs (st_dev)
        Ok(metadata1.dev() == metadata2.dev())
    }

    /// Detect the mount point for a given path
    pub async fn get_mount_point(&self, path: &Path) -> Result<PathBuf> {
        let mut current = path.to_path_buf();
        let mut current_dev = fs::metadata(&current).await?.dev();

        // Walk up the directory tree until we find a different device
        loop {
            let parent = match current.parent() {
                Some(p) => p.to_path_buf(),
                None => break, // Reached root
            };

            let parent_dev = fs::metadata(&parent).await?.dev();
            
            if parent_dev != current_dev {
                // Found the mount point
                break;
            }

            current = parent;
            current_dev = parent_dev;
        }

        Ok(current)
    }

    /// Atomic write operation
    /// Writes to a temporary file then atomically renames it
    pub async fn atomic_write(&self, path: &Path, data: &[u8]) -> Result<()> {
        // Create a temporary file in the same directory
        let temp_path = self.temp_path(path)?;

        // Write data to temporary file
        fs::write(&temp_path, data)
            .await
            .with_context(|| format!("Failed to write to temporary file: {}", temp_path.display()))?;

        // Atomically rename temporary file to target
        fs::rename(&temp_path, path)
            .await
            .with_context(|| format!("Failed to rename {} to {}", temp_path.display(), path.display()))?;

        Ok(())
    }

    /// Generate a temporary file path in the same directory
    fn temp_path(&self, path: &Path) -> Result<PathBuf> {
        let parent = path.parent()
            .ok_or_else(|| anyhow::anyhow!("Path has no parent: {}", path.display()))?;
        
        let filename = path.file_name()
            .ok_or_else(|| anyhow::anyhow!("Path has no filename: {}", path.display()))?;
        
        let temp_filename = format!(".tmp.{}.{}", 
            filename.to_string_lossy(),
            std::process::id()
        );

        Ok(parent.join(temp_filename))
    }
}

impl Default for AtomicMover {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs as std_fs;

    #[tokio::test]
    async fn test_atomic_rename_same_filesystem() {
        let temp_dir = TempDir::new().unwrap();
        let from = temp_dir.path().join("source.txt");
        let to = temp_dir.path().join("dest.txt");

        // Create source file
        std_fs::write(&from, b"test content").unwrap();

        let mover = AtomicMover::new();
        mover.atomic_move(&from, &to).await.unwrap();

        // Verify move succeeded
        assert!(!from.exists());
        assert!(to.exists());
        assert_eq!(std_fs::read(&to).unwrap(), b"test content");
    }

    #[tokio::test]
    async fn test_same_filesystem_detection() {
        let temp_dir = TempDir::new().unwrap();
        let path1 = temp_dir.path().join("file1.txt");
        let path2 = temp_dir.path().join("file2.txt");

        std_fs::write(&path1, b"test").unwrap();

        let mover = AtomicMover::new();
        let same = mover.same_filesystem(&path1, &path2).await.unwrap();

        // Both paths are in the same temp directory, so same filesystem
        assert!(same);
    }

    #[tokio::test]
    async fn test_atomic_write() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.txt");

        let mover = AtomicMover::new();
        mover.atomic_write(&path, b"atomic content").await.unwrap();

        assert!(path.exists());
        assert_eq!(std_fs::read(&path).unwrap(), b"atomic content");
    }

    #[tokio::test]
    async fn test_get_mount_point() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("subdir").join("file.txt");

        std_fs::create_dir_all(path.parent().unwrap()).unwrap();
        std_fs::write(&path, b"test").unwrap();

        let mover = AtomicMover::new();
        let mount_point = mover.get_mount_point(&path).await.unwrap();

        // Mount point should be a valid path
        assert!(mount_point.exists());
    }
}
