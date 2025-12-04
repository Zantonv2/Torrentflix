// Filesystem-specific optimizations for Linux
// Detects filesystem type and uses optimized operations when available

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use tokio::fs;

/// Filesystem types supported for optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemType {
    Ext4,
    Btrfs,
    Xfs,
    Ext3,
    Ext2,
    F2fs,
    Ntfs,
    Vfat,
    Unknown,
}

impl FilesystemType {
    /// Check if this filesystem supports reflinks (copy-on-write)
    pub fn supports_reflinks(&self) -> bool {
        matches!(self, FilesystemType::Btrfs | FilesystemType::Xfs)
    }

    /// Check if this filesystem supports extended attributes
    pub fn supports_xattrs(&self) -> bool {
        matches!(
            self,
            FilesystemType::Ext4
                | FilesystemType::Ext3
                | FilesystemType::Ext2
                | FilesystemType::Btrfs
                | FilesystemType::Xfs
                | FilesystemType::F2fs
        )
    }
}

impl From<&str> for FilesystemType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ext4" => FilesystemType::Ext4,
            "btrfs" => FilesystemType::Btrfs,
            "xfs" => FilesystemType::Xfs,
            "ext3" => FilesystemType::Ext3,
            "ext2" => FilesystemType::Ext2,
            "f2fs" => FilesystemType::F2fs,
            "ntfs" => FilesystemType::Ntfs,
            "vfat" | "fat32" | "fat16" => FilesystemType::Vfat,
            _ => FilesystemType::Unknown,
        }
    }
}

/// Filesystem optimizer that detects and uses filesystem-specific features
pub struct FilesystemOptimizer;

impl FilesystemOptimizer {
    pub fn new() -> Self {
        Self
    }

    /// Detect the filesystem type for a given path
    pub async fn detect_filesystem_type(&self, path: &Path) -> Result<FilesystemType> {
        // Use df command to get filesystem type
        let output = Command::new("df")
            .arg("-T")
            .arg(path)
            .output()
            .context("Failed to execute df command")?;

        if !output.status.success() {
            return Ok(FilesystemType::Unknown);
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse df output (format: Filesystem Type ...)
        // Skip header line and get the second column
        if let Some(line) = output_str.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return Ok(FilesystemType::from(parts[1]));
            }
        }

        Ok(FilesystemType::Unknown)
    }

    /// Attempt to use reflink for efficient copying on supported filesystems
    /// Falls back to regular copy if reflink is not supported
    pub async fn reflink_copy(&self, from: &Path, to: &Path) -> Result<bool> {
        // Check if source filesystem supports reflinks
        let fs_type = self.detect_filesystem_type(from).await?;
        
        if !fs_type.supports_reflinks() {
            return Ok(false);
        }

        // Try to use cp with --reflink=auto
        let output = Command::new("cp")
            .arg("--reflink=auto")
            .arg(from)
            .arg(to)
            .output()
            .context("Failed to execute cp with reflink")?;

        Ok(output.status.success())
    }

    /// Set extended attribute on a file
    /// Only works on filesystems that support xattrs
    pub async fn set_xattr(&self, path: &Path, name: &str, value: &[u8]) -> Result<()> {
        let fs_type = self.detect_filesystem_type(path).await?;
        
        if !fs_type.supports_xattrs() {
            anyhow::bail!("Filesystem does not support extended attributes");
        }

        // Use setfattr command
        let value_str = String::from_utf8_lossy(value);
        let output = Command::new("setfattr")
            .arg("-n")
            .arg(name)
            .arg("-v")
            .arg(value_str.as_ref())
            .arg(path)
            .output()
            .context("Failed to execute setfattr")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to set xattr: {}", error);
        }

        Ok(())
    }

    /// Get extended attribute from a file
    pub async fn get_xattr(&self, path: &Path, name: &str) -> Result<Vec<u8>> {
        let fs_type = self.detect_filesystem_type(path).await?;
        
        if !fs_type.supports_xattrs() {
            anyhow::bail!("Filesystem does not support extended attributes");
        }

        // Use getfattr command
        let output = Command::new("getfattr")
            .arg("-n")
            .arg(name)
            .arg("--only-values")
            .arg(path)
            .output()
            .context("Failed to execute getfattr")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to get xattr: {}", error);
        }

        Ok(output.stdout)
    }

    /// Remove extended attribute from a file
    pub async fn remove_xattr(&self, path: &Path, name: &str) -> Result<()> {
        let fs_type = self.detect_filesystem_type(path).await?;
        
        if !fs_type.supports_xattrs() {
            anyhow::bail!("Filesystem does not support extended attributes");
        }

        // Use setfattr command with -x flag
        let output = Command::new("setfattr")
            .arg("-x")
            .arg(name)
            .arg(path)
            .output()
            .context("Failed to execute setfattr")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to remove xattr: {}", error);
        }

        Ok(())
    }

    /// Optimized copy operation that uses reflinks when available
    pub async fn optimized_copy(&self, from: &Path, to: &Path) -> Result<()> {
        // Try reflink first
        if self.reflink_copy(from, to).await? {
            return Ok(());
        }

        // Fall back to regular copy
        fs::copy(from, to)
            .await
            .with_context(|| format!("Failed to copy {} to {}", from.display(), to.display()))?;

        Ok(())
    }

    /// Get filesystem information for a path
    pub async fn get_filesystem_info(&self, path: &Path) -> Result<FilesystemInfo> {
        let fs_type = self.detect_filesystem_type(path).await?;
        
        Ok(FilesystemInfo {
            fs_type,
            supports_reflinks: fs_type.supports_reflinks(),
            supports_xattrs: fs_type.supports_xattrs(),
        })
    }
}

impl Default for FilesystemOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a filesystem
#[derive(Debug, Clone)]
pub struct FilesystemInfo {
    pub fs_type: FilesystemType,
    pub supports_reflinks: bool,
    pub supports_xattrs: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs as std_fs;

    #[tokio::test]
    async fn test_detect_filesystem_type() {
        let temp_dir = TempDir::new().unwrap();
        let optimizer = FilesystemOptimizer::new();
        
        let fs_type = optimizer.detect_filesystem_type(temp_dir.path()).await.unwrap();
        
        // Should detect some filesystem type (not Unknown in most cases)
        println!("Detected filesystem: {:?}", fs_type);
    }

    #[tokio::test]
    async fn test_filesystem_type_from_string() {
        assert_eq!(FilesystemType::from("ext4"), FilesystemType::Ext4);
        assert_eq!(FilesystemType::from("btrfs"), FilesystemType::Btrfs);
        assert_eq!(FilesystemType::from("xfs"), FilesystemType::Xfs);
        assert_eq!(FilesystemType::from("unknown"), FilesystemType::Unknown);
    }

    #[tokio::test]
    async fn test_filesystem_capabilities() {
        assert!(FilesystemType::Btrfs.supports_reflinks());
        assert!(FilesystemType::Xfs.supports_reflinks());
        assert!(!FilesystemType::Ext4.supports_reflinks());
        
        assert!(FilesystemType::Ext4.supports_xattrs());
        assert!(FilesystemType::Btrfs.supports_xattrs());
        assert!(FilesystemType::Xfs.supports_xattrs());
    }

    #[tokio::test]
    async fn test_optimized_copy() {
        let temp_dir = TempDir::new().unwrap();
        let from = temp_dir.path().join("source.txt");
        let to = temp_dir.path().join("dest.txt");
        
        std_fs::write(&from, b"test content").unwrap();
        
        let optimizer = FilesystemOptimizer::new();
        optimizer.optimized_copy(&from, &to).await.unwrap();
        
        assert!(to.exists());
        assert_eq!(std_fs::read(&to).unwrap(), b"test content");
    }

    #[tokio::test]
    async fn test_get_filesystem_info() {
        let temp_dir = TempDir::new().unwrap();
        let optimizer = FilesystemOptimizer::new();
        
        let info = optimizer.get_filesystem_info(temp_dir.path()).await.unwrap();
        
        println!("Filesystem info: {:?}", info);
        // Just verify we can get the info without errors
    }
}
