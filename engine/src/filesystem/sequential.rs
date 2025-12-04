// Sequential Access Optimization for HDD
// Reads directory entries in filesystem order to leverage HDD read-ahead caching
// Optimizes for sequential access patterns

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info};

// ============================================================================
// Constants
// ============================================================================

/// Default read-ahead buffer size for sequential access (1MB)
const DEFAULT_READAHEAD_SIZE: usize = 1024 * 1024; // 1MB

// ============================================================================
// Types
// ============================================================================

/// Sequential directory entry with metadata
#[derive(Debug, Clone)]
pub struct SequentialEntry {
    /// Full path to the entry
    pub path: PathBuf,
    /// File name
    pub name: String,
    /// Whether this is a directory
    pub is_dir: bool,
    /// File size in bytes
    pub size: u64,
    /// Estimated physical location (inode number)
    pub inode: u64,
}

impl SequentialEntry {
    /// Create a new sequential entry
    pub fn new(path: PathBuf, name: String, is_dir: bool, size: u64, inode: u64) -> Self {
        Self {
            path,
            name,
            is_dir,
            size,
            inode,
        }
    }
}

/// Sequential Directory Reader
///
/// Reads directory entries in filesystem order to leverage HDD read-ahead caching.
/// Entries are returned in the order they appear on disk, which optimizes for
/// sequential access patterns on mechanical drives.
///
/// # Requirements
/// - Requirements: 27.2
pub struct SequentialDirectoryReader {
    /// Read-ahead buffer size
    readahead_size: usize,
}

impl SequentialDirectoryReader {
    /// Create a new sequential directory reader with default settings
    pub fn new() -> Self {
        Self {
            readahead_size: DEFAULT_READAHEAD_SIZE,
        }
    }

    /// Create a new sequential directory reader with custom read-ahead size
    pub fn with_readahead_size(readahead_size: usize) -> Self {
        Self { readahead_size }
    }

    /// Read directory entries in filesystem order
    ///
    /// Returns entries in the order they appear on disk, which is optimal
    /// for sequential access on HDD storage.
    ///
    /// # Arguments
    /// * `path` - Directory path to read
    /// * `recursive` - Whether to recursively read subdirectories
    ///
    /// # Returns
    /// A vector of SequentialEntry in filesystem order
    ///
    /// # Errors
    /// Returns an error if the directory cannot be read
    pub async fn read_directory(&self, path: &Path, recursive: bool) -> Result<Vec<SequentialEntry>> {
        debug!("Reading directory in sequential order: {:?}", path);

        let mut entries = Vec::new();
        
        if recursive {
            // Use iterative approach with a stack to avoid async recursion
            let mut dirs_to_process = vec![(path.to_path_buf(), 0)];
            
            while let Some((current_path, depth)) = dirs_to_process.pop() {
                if depth >= 100 {
                    // Limit recursion depth to prevent stack overflow
                    continue;
                }
                
                self.read_directory_non_recursive(&current_path, &mut entries, &mut dirs_to_process, depth).await?;
            }
        } else {
            self.read_directory_non_recursive(path, &mut entries, &mut Vec::new(), 0).await?;
        }

        info!("Read {} entries from directory: {:?}", entries.len(), path);
        Ok(entries)
    }

    /// Read a single directory without recursion
    async fn read_directory_non_recursive(
        &self,
        path: &Path,
        entries: &mut Vec<SequentialEntry>,
        dirs_to_process: &mut Vec<(PathBuf, usize)>,
        depth: usize,
    ) -> Result<()> {
        let mut dir = fs::read_dir(path)
            .await
            .with_context(|| format!("Failed to read directory: {}", path.display()))?;

        // Collect entries first to maintain filesystem order
        let mut dir_entries = Vec::new();
        while let Some(entry) = dir.next_entry().await? {
            dir_entries.push(entry);
        }

        // Process entries in the order they were returned (filesystem order)
        for entry in dir_entries {
            let entry_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            let metadata = entry.metadata()
                .await
                .with_context(|| format!("Failed to get metadata: {}", entry_path.display()))?;

            let is_dir = metadata.is_dir();
            let size = metadata.len();

            // Get inode number for physical location estimation
            #[cfg(unix)]
            let inode = {
                use std::os::unix::fs::MetadataExt;
                metadata.ino()
            };
            #[cfg(not(unix))]
            let inode = 0u64;

            let seq_entry = SequentialEntry::new(entry_path.clone(), name, is_dir, size, inode);
            entries.push(seq_entry);

            // Queue subdirectories for processing if recursive
            if is_dir && depth < 100 {
                dirs_to_process.push((entry_path, depth + 1));
            }
        }

        Ok(())
    }

    /// Read directory entries and sort by inode (physical location)
    ///
    /// This attempts to optimize access by reading files in the order
    /// they appear on disk (by inode number).
    ///
    /// # Arguments
    /// * `path` - Directory path to read
    ///
    /// # Returns
    /// A vector of SequentialEntry sorted by inode
    pub async fn read_directory_by_inode(&self, path: &Path) -> Result<Vec<SequentialEntry>> {
        debug!("Reading directory sorted by inode: {:?}", path);

        let mut entries = self.read_directory(path, false).await?;

        // Sort by inode to optimize physical access order
        entries.sort_by_key(|e| e.inode);

        info!("Read {} entries sorted by inode from: {:?}", entries.len(), path);
        Ok(entries)
    }

    /// Read directory entries and group by parent directory
    ///
    /// Groups entries by their parent directory to optimize access patterns
    /// when processing multiple directories.
    ///
    /// # Arguments
    /// * `path` - Root directory path to read
    ///
    /// # Returns
    /// A vector of SequentialEntry grouped by parent directory
    pub async fn read_directory_grouped(&self, path: &Path) -> Result<Vec<SequentialEntry>> {
        debug!("Reading directory grouped by parent: {:?}", path);

        let mut entries = self.read_directory(path, true).await?;

        // Sort by parent directory first, then by name within each directory
        entries.sort_by(|a, b| {
            let a_parent = a.path.parent().unwrap_or_else(|| Path::new(""));
            let b_parent = b.path.parent().unwrap_or_else(|| Path::new(""));

            match a_parent.cmp(b_parent) {
                std::cmp::Ordering::Equal => a.path.cmp(&b.path),
                other => other,
            }
        });

        info!("Read {} entries grouped by parent from: {:?}", entries.len(), path);
        Ok(entries)
    }

    /// Estimate physical location of a file
    ///
    /// Returns the inode number, which can be used to estimate
    /// physical location on disk.
    ///
    /// # Arguments
    /// * `path` - File path
    ///
    /// # Returns
    /// The inode number (0 on non-Unix systems)
    pub async fn get_inode(&self, path: &Path) -> Result<u64> {
        let metadata = fs::metadata(path)
            .await
            .with_context(|| format!("Failed to get metadata: {}", path.display()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            Ok(metadata.ino())
        }

        #[cfg(not(unix))]
        Ok(0u64)
    }

    /// Prefetch directory entries into cache
    ///
    /// Reads directory entries to warm up the filesystem cache,
    /// improving performance for subsequent operations.
    ///
    /// # Arguments
    /// * `path` - Directory path to prefetch
    ///
    /// # Errors
    /// Returns an error if the directory cannot be read
    pub async fn prefetch_directory(&self, path: &Path) -> Result<usize> {
        debug!("Prefetching directory: {:?}", path);

        let entries = self.read_directory(path, false).await?;
        let count = entries.len();

        info!("Prefetched {} entries from: {:?}", count, path);
        Ok(count)
    }
}

impl Default for SequentialDirectoryReader {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs as std_fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_sequential_entry_creation() {
        let entry = SequentialEntry::new(
            PathBuf::from("/tmp/test.txt"),
            "test.txt".to_string(),
            false,
            1024,
            12345,
        );

        assert_eq!(entry.name, "test.txt");
        assert!(!entry.is_dir);
        assert_eq!(entry.size, 1024);
        assert_eq!(entry.inode, 12345);
    }

    #[tokio::test]
    async fn test_sequential_reader_creation() {
        let reader = SequentialDirectoryReader::new();
        assert_eq!(reader.readahead_size, DEFAULT_READAHEAD_SIZE);

        let reader = SequentialDirectoryReader::with_readahead_size(2048);
        assert_eq!(reader.readahead_size, 2048);
    }

    #[tokio::test]
    async fn test_read_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create some test files
        std_fs::write(temp_dir.path().join("file1.txt"), b"content1").unwrap();
        std_fs::write(temp_dir.path().join("file2.txt"), b"content2").unwrap();
        std_fs::write(temp_dir.path().join("file3.txt"), b"content3").unwrap();

        let reader = SequentialDirectoryReader::new();
        let entries = reader.read_directory(temp_dir.path(), false).await.unwrap();

        assert_eq!(entries.len(), 3);
        assert!(entries.iter().all(|e| !e.is_dir));
    }

    #[tokio::test]
    async fn test_read_directory_recursive() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested structure
        std_fs::write(temp_dir.path().join("file1.txt"), b"content1").unwrap();
        std_fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        std_fs::write(temp_dir.path().join("subdir/file2.txt"), b"content2").unwrap();

        let reader = SequentialDirectoryReader::new();
        let entries = reader.read_directory(temp_dir.path(), true).await.unwrap();

        // Should have 3 entries: file1.txt, subdir, subdir/file2.txt
        assert_eq!(entries.len(), 3);
    }

    #[tokio::test]
    async fn test_read_directory_by_inode() {
        let temp_dir = TempDir::new().unwrap();

        // Create some test files
        std_fs::write(temp_dir.path().join("file1.txt"), b"content1").unwrap();
        std_fs::write(temp_dir.path().join("file2.txt"), b"content2").unwrap();
        std_fs::write(temp_dir.path().join("file3.txt"), b"content3").unwrap();

        let reader = SequentialDirectoryReader::new();
        let entries = reader.read_directory_by_inode(temp_dir.path()).await.unwrap();

        assert_eq!(entries.len(), 3);

        // Entries should be sorted by inode
        for i in 1..entries.len() {
            assert!(entries[i - 1].inode <= entries[i].inode);
        }
    }

    #[tokio::test]
    async fn test_read_directory_grouped() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested structure
        std_fs::write(temp_dir.path().join("file1.txt"), b"content1").unwrap();
        std_fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        std_fs::write(temp_dir.path().join("subdir/file2.txt"), b"content2").unwrap();

        let reader = SequentialDirectoryReader::new();
        let entries = reader.read_directory_grouped(temp_dir.path()).await.unwrap();

        assert_eq!(entries.len(), 3);

        // Entries should be grouped by parent directory
        let mut prev_parent = entries[0].path.parent();
        for entry in &entries[1..] {
            let curr_parent = entry.path.parent();
            // Parent should be same or greater (sorted)
            assert!(prev_parent <= curr_parent);
            prev_parent = curr_parent;
        }
    }

    #[tokio::test]
    async fn test_get_inode() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std_fs::write(&file_path, b"test").unwrap();

        let reader = SequentialDirectoryReader::new();
        let inode = reader.get_inode(&file_path).await.unwrap();

        // On Unix systems, inode should be > 0
        #[cfg(unix)]
        assert!(inode > 0);

        #[cfg(not(unix))]
        assert_eq!(inode, 0);
    }

    #[tokio::test]
    async fn test_prefetch_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create some test files
        std_fs::write(temp_dir.path().join("file1.txt"), b"content1").unwrap();
        std_fs::write(temp_dir.path().join("file2.txt"), b"content2").unwrap();

        let reader = SequentialDirectoryReader::new();
        let count = reader.prefetch_directory(temp_dir.path()).await.unwrap();

        assert_eq!(count, 2);
    }
}
