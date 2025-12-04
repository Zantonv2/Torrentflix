// Filesystem link handling: symbolic links and hard links
// Provides detection, resolution, and management of filesystem links

use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Information about a filesystem link
#[derive(Debug, Clone)]
pub struct LinkInfo {
    /// The path that was checked
    pub path: PathBuf,
    /// Whether this is a symbolic link
    pub is_symlink: bool,
    /// The target of the symlink (if it's a symlink)
    pub symlink_target: Option<PathBuf>,
    /// Whether this is a hard link (multiple paths to same inode)
    pub is_hardlink: bool,
    /// The inode number
    pub inode: Option<u64>,
    /// Whether the link is broken (target doesn't exist)
    pub is_broken: bool,
}

/// Detects and resolves symbolic links
pub struct LinkResolver;

impl LinkResolver {
    /// Analyze a path to determine if it's a symlink, hardlink, or regular file
    pub fn analyze_path(path: &Path) -> Result<LinkInfo> {
        debug!("Analyzing path for link information: {:?}", path);

        // Get metadata without following symlinks
        let metadata = fs::symlink_metadata(path)
            .context(format!("Failed to get metadata for {:?}", path))?;

        let is_symlink = metadata.is_symlink();
        let inode = Some(metadata.ino());
        
        let mut symlink_target = None;
        let mut is_broken = false;

        // If it's a symlink, resolve the target
        if is_symlink {
            match fs::read_link(path) {
                Ok(target) => {
                    debug!("Symlink {:?} points to {:?}", path, target);
                    
                    // Check if the target exists
                    let resolved_target = if target.is_absolute() {
                        target.clone()
                    } else {
                        // Resolve relative symlinks relative to the symlink's parent directory
                        path.parent()
                            .unwrap_or_else(|| Path::new("/"))
                            .join(&target)
                    };
                    
                    is_broken = !resolved_target.exists();
                    
                    if is_broken {
                        warn!("Broken symlink detected: {:?} -> {:?}", path, resolved_target);
                    }
                    
                    symlink_target = Some(resolved_target);
                }
                Err(e) => {
                    warn!("Failed to read symlink target for {:?}: {}", path, e);
                    is_broken = true;
                }
            }
        }

        // Determine if this is a hard link
        // A file is considered a hard link if it has more than one link count
        let is_hardlink = metadata.nlink() > 1;

        Ok(LinkInfo {
            path: path.to_path_buf(),
            is_symlink,
            symlink_target,
            is_hardlink,
            inode,
            is_broken,
        })
    }

    /// Resolve a symlink to its final target (following chains of symlinks)
    pub fn resolve_symlink_chain(path: &Path) -> Result<PathBuf> {
        debug!("Resolving symlink chain for {:?}", path);

        // Use canonicalize to follow all symlinks to the final target
        match fs::canonicalize(path) {
            Ok(resolved) => {
                debug!("Resolved {:?} to {:?}", path, resolved);
                Ok(resolved)
            }
            Err(e) => {
                // If canonicalize fails, the symlink might be broken
                warn!("Failed to resolve symlink chain for {:?}: {}", path, e);
                Err(anyhow::anyhow!("Failed to resolve symlink: {}", e))
            }
        }
    }

    /// Get the target path for a symlink, or return the original path if not a symlink
    pub fn get_target_path(path: &Path) -> Result<PathBuf> {
        let link_info = Self::analyze_path(path)?;

        if link_info.is_symlink {
            if let Some(target) = link_info.symlink_target {
                Ok(target)
            } else {
                // Broken symlink
                Err(anyhow::anyhow!("Broken symlink: {:?}", path))
            }
        } else {
            // Not a symlink, return the original path
            Ok(path.to_path_buf())
        }
    }

    /// Check if a path is a broken symlink
    pub fn is_broken_symlink(path: &Path) -> bool {
        match Self::analyze_path(path) {
            Ok(info) => info.is_broken,
            Err(_) => false,
        }
    }

    /// Find all paths that share the same inode (hard links)
    pub fn find_hardlinks_in_directory(
        directory: &Path,
        target_inode: u64,
    ) -> Result<Vec<PathBuf>> {
        debug!(
            "Searching for hard links with inode {} in {:?}",
            target_inode, directory
        );

        let mut hardlinks = Vec::new();

        // Recursively walk the directory
        for entry in walkdir::WalkDir::new(directory)
            .follow_links(false) // Don't follow symlinks
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.ino() == target_inode {
                        hardlinks.push(entry.path().to_path_buf());
                    }
                }
            }
        }

        debug!("Found {} hard links for inode {}", hardlinks.len(), target_inode);
        Ok(hardlinks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::os::unix::fs as unix_fs;
    use tempfile::TempDir;

    #[test]
    fn test_analyze_regular_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("regular.txt");
        File::create(&file_path).unwrap();

        let info = LinkResolver::analyze_path(&file_path).unwrap();

        assert!(!info.is_symlink);
        assert!(info.symlink_target.is_none());
        assert!(!info.is_hardlink); // Single link count
        assert!(info.inode.is_some());
        assert!(!info.is_broken);
    }

    #[test]
    fn test_analyze_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let target_path = temp_dir.path().join("target.txt");
        let link_path = temp_dir.path().join("link.txt");

        File::create(&target_path).unwrap();
        unix_fs::symlink(&target_path, &link_path).unwrap();

        let info = LinkResolver::analyze_path(&link_path).unwrap();

        assert!(info.is_symlink);
        assert!(info.symlink_target.is_some());
        assert!(!info.is_broken);
        assert!(info.inode.is_some());
    }

    #[test]
    fn test_analyze_broken_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let target_path = temp_dir.path().join("nonexistent.txt");
        let link_path = temp_dir.path().join("broken_link.txt");

        unix_fs::symlink(&target_path, &link_path).unwrap();

        let info = LinkResolver::analyze_path(&link_path).unwrap();

        assert!(info.is_symlink);
        assert!(info.symlink_target.is_some());
        assert!(info.is_broken);
    }

    #[test]
    fn test_analyze_hardlink() {
        let temp_dir = TempDir::new().unwrap();
        let original_path = temp_dir.path().join("original.txt");
        let hardlink_path = temp_dir.path().join("hardlink.txt");

        File::create(&original_path).unwrap();
        fs::hard_link(&original_path, &hardlink_path).unwrap();

        let info1 = LinkResolver::analyze_path(&original_path).unwrap();
        let info2 = LinkResolver::analyze_path(&hardlink_path).unwrap();

        assert!(info1.is_hardlink);
        assert!(info2.is_hardlink);
        assert_eq!(info1.inode, info2.inode);
        assert!(!info1.is_symlink);
        assert!(!info2.is_symlink);
    }

    #[test]
    fn test_resolve_symlink_chain() {
        let temp_dir = TempDir::new().unwrap();
        let target_path = temp_dir.path().join("target.txt");
        let link1_path = temp_dir.path().join("link1.txt");
        let link2_path = temp_dir.path().join("link2.txt");

        File::create(&target_path).unwrap();
        unix_fs::symlink(&target_path, &link1_path).unwrap();
        unix_fs::symlink(&link1_path, &link2_path).unwrap();

        let resolved = LinkResolver::resolve_symlink_chain(&link2_path).unwrap();
        
        // The resolved path should be the canonical path to the target
        assert!(resolved.ends_with("target.txt"));
    }

    #[test]
    fn test_get_target_path_for_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let target_path = temp_dir.path().join("target.txt");
        let link_path = temp_dir.path().join("link.txt");

        File::create(&target_path).unwrap();
        unix_fs::symlink(&target_path, &link_path).unwrap();

        let target = LinkResolver::get_target_path(&link_path).unwrap();
        assert!(target.ends_with("target.txt"));
    }

    #[test]
    fn test_get_target_path_for_regular_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("regular.txt");
        File::create(&file_path).unwrap();

        let target = LinkResolver::get_target_path(&file_path).unwrap();
        assert_eq!(target, file_path);
    }

    #[test]
    fn test_is_broken_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let target_path = temp_dir.path().join("nonexistent.txt");
        let link_path = temp_dir.path().join("broken_link.txt");

        unix_fs::symlink(&target_path, &link_path).unwrap();

        assert!(LinkResolver::is_broken_symlink(&link_path));
    }

    #[test]
    fn test_find_hardlinks_in_directory() {
        let temp_dir = TempDir::new().unwrap();
        let original_path = temp_dir.path().join("original.txt");
        let hardlink1_path = temp_dir.path().join("hardlink1.txt");
        let hardlink2_path = temp_dir.path().join("hardlink2.txt");

        File::create(&original_path).unwrap();
        fs::hard_link(&original_path, &hardlink1_path).unwrap();
        fs::hard_link(&original_path, &hardlink2_path).unwrap();

        let metadata = fs::metadata(&original_path).unwrap();
        let inode = metadata.ino();

        let hardlinks = LinkResolver::find_hardlinks_in_directory(temp_dir.path(), inode).unwrap();

        assert_eq!(hardlinks.len(), 3); // original + 2 hardlinks
    }
}
