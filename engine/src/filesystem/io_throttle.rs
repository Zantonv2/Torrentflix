// I/O Throttling for HDD Optimization
// Limits concurrent I/O operations and detects storage type
// Auto-adjusts concurrency based on HDD vs SSD detection

use anyhow::{Result, Context};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, info};

// ============================================================================
// Constants
// ============================================================================

/// Default concurrency limit for HDD storage (1 concurrent operation)
const DEFAULT_HDD_CONCURRENCY: usize = 1;

/// Default concurrency limit for SSD storage (4 concurrent operations)
const DEFAULT_SSD_CONCURRENCY: usize = 4;

/// Default concurrency limit for unknown storage (2 concurrent operations)
const DEFAULT_UNKNOWN_CONCURRENCY: usize = 2;

// ============================================================================
// Types
// ============================================================================

/// Storage type detection result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    /// Hard Disk Drive (mechanical)
    HDD,
    /// Solid State Drive
    SSD,
    /// Unknown storage type
    Unknown,
}

impl StorageType {
    /// Get recommended concurrency limit for this storage type
    pub fn recommended_concurrency(&self) -> usize {
        match self {
            StorageType::HDD => DEFAULT_HDD_CONCURRENCY,
            StorageType::SSD => DEFAULT_SSD_CONCURRENCY,
            StorageType::Unknown => DEFAULT_UNKNOWN_CONCURRENCY,
        }
    }

    /// Get a human-readable name
    pub fn as_str(&self) -> &'static str {
        match self {
            StorageType::HDD => "HDD",
            StorageType::SSD => "SSD",
            StorageType::Unknown => "Unknown",
        }
    }
}

/// I/O Throttler for limiting concurrent operations
///
/// Detects storage type (HDD vs SSD) and automatically adjusts
/// concurrency limits to prevent I/O thrashing.
///
/// # Requirements
/// - Requirements: 27.6
pub struct IOThrottler {
    /// Semaphore for limiting concurrent operations
    semaphore: Arc<Semaphore>,
    /// Current concurrency limit
    concurrency_limit: Arc<AtomicUsize>,
    /// Detected storage type
    storage_type: StorageType,
}

/// A permit for an I/O operation
///
/// Holds a semaphore permit and releases it when dropped.
pub struct IOPermit {
    _semaphore: Arc<Semaphore>,
    _acquired: bool,
}

impl Drop for IOPermit {
    fn drop(&mut self) {
        if self._acquired {
            self._semaphore.add_permits(1);
        }
    }
}

impl IOThrottler {
    /// Create a new I/O throttler with auto-detected storage type
    pub async fn new(path: &Path) -> Result<Self> {
        let storage_type = Self::detect_storage_type(path).await?;
        let concurrency = storage_type.recommended_concurrency();

        info!("I/O Throttler initialized: storage_type={}, concurrency={}", 
              storage_type.as_str(), concurrency);

        Ok(Self {
            semaphore: Arc::new(Semaphore::new(concurrency)),
            concurrency_limit: Arc::new(AtomicUsize::new(concurrency)),
            storage_type,
        })
    }

    /// Create a new I/O throttler with explicit storage type
    pub fn with_storage_type(storage_type: StorageType) -> Self {
        let concurrency = storage_type.recommended_concurrency();

        info!("I/O Throttler initialized: storage_type={}, concurrency={}", 
              storage_type.as_str(), concurrency);

        Self {
            semaphore: Arc::new(Semaphore::new(concurrency)),
            concurrency_limit: Arc::new(AtomicUsize::new(concurrency)),
            storage_type,
        }
    }

    /// Create a new I/O throttler with custom concurrency limit
    pub fn with_concurrency(concurrency: usize) -> Self {
        info!("I/O Throttler initialized: concurrency={}", concurrency);

        Self {
            semaphore: Arc::new(Semaphore::new(concurrency)),
            concurrency_limit: Arc::new(AtomicUsize::new(concurrency)),
            storage_type: StorageType::Unknown,
        }
    }

    /// Get the current storage type
    pub fn storage_type(&self) -> StorageType {
        self.storage_type
    }

    /// Get the current concurrency limit
    pub fn concurrency_limit(&self) -> usize {
        self.concurrency_limit.load(Ordering::Relaxed)
    }

    /// Set a new concurrency limit
    pub fn set_concurrency_limit(&self, limit: usize) {
        debug!("Updating concurrency limit from {} to {}", 
               self.concurrency_limit.load(Ordering::Relaxed), limit);
        self.concurrency_limit.store(limit, Ordering::Relaxed);
    }

    /// Acquire a permit for an I/O operation
    ///
    /// This will block until a permit is available.
    /// The permit is automatically released when dropped.
    pub async fn acquire(&self) -> Result<IOPermit> {
        let _permit = self.semaphore.acquire().await?;
        // Forget the permit so it stays acquired until IOPermit is dropped
        std::mem::forget(_permit);
        Ok(IOPermit { 
            _semaphore: Arc::clone(&self.semaphore),
            _acquired: true,
        })
    }

    /// Try to acquire a permit without blocking
    pub fn try_acquire(&self) -> Option<IOPermit> {
        self.semaphore.try_acquire().ok().map(|_permit| {
            // Forget the permit so it stays acquired until IOPermit is dropped
            std::mem::forget(_permit);
            IOPermit { 
                _semaphore: Arc::clone(&self.semaphore),
                _acquired: true,
            }
        })
    }

    /// Detect storage type for a given path
    ///
    /// Attempts to detect whether the storage is HDD or SSD by checking:
    /// 1. /sys/block/*/queue/rotational (Linux)
    /// 2. Device characteristics
    ///
    /// Returns Unknown if detection fails.
    async fn detect_storage_type(path: &Path) -> Result<StorageType> {
        // Try to get the device for the path
        let device = Self::get_device_for_path(path).await?;

        // Check if device is rotational (HDD) or not (SSD)
        match Self::is_rotational(&device).await {
            Ok(true) => {
                debug!("Detected HDD storage: {}", device);
                Ok(StorageType::HDD)
            }
            Ok(false) => {
                debug!("Detected SSD storage: {}", device);
                Ok(StorageType::SSD)
            }
            Err(_) => {
                debug!("Could not detect storage type for: {}", device);
                Ok(StorageType::Unknown)
            }
        }
    }

    /// Get the device name for a given path
    async fn get_device_for_path(path: &Path) -> Result<String> {
        use std::process::Command;

        let output = Command::new("df")
            .arg(path)
            .output()
            .context("Failed to execute df command")?;

        if !output.status.success() {
            anyhow::bail!("df command failed");
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse df output to get device name
        if let Some(line) = output_str.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                return Ok(parts[0].to_string());
            }
        }

        anyhow::bail!("Could not parse df output")
    }

    /// Check if a device is rotational (HDD)
    async fn is_rotational(device: &str) -> Result<bool> {
        use std::process::Command;

        // Extract device name (e.g., /dev/sda1 -> sda)
        let device_name = device
            .split('/')
            .last()
            .unwrap_or(device)
            .chars()
            .take_while(|c| c.is_alphabetic())
            .collect::<String>();

        if device_name.is_empty() {
            anyhow::bail!("Could not extract device name");
        }

        // Check /sys/block/{device}/queue/rotational
        let sysfs_path = format!("/sys/block/{}/queue/rotational", device_name);
        
        let output = Command::new("cat")
            .arg(&sysfs_path)
            .output()
            .context("Failed to read rotational flag")?;

        if !output.status.success() {
            anyhow::bail!("Could not read rotational flag");
        }

        let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(value == "1")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage_type_concurrency() {
        assert_eq!(StorageType::HDD.recommended_concurrency(), DEFAULT_HDD_CONCURRENCY);
        assert_eq!(StorageType::SSD.recommended_concurrency(), DEFAULT_SSD_CONCURRENCY);
        assert_eq!(StorageType::Unknown.recommended_concurrency(), DEFAULT_UNKNOWN_CONCURRENCY);
    }

    #[test]
    fn test_storage_type_as_str() {
        assert_eq!(StorageType::HDD.as_str(), "HDD");
        assert_eq!(StorageType::SSD.as_str(), "SSD");
        assert_eq!(StorageType::Unknown.as_str(), "Unknown");
    }

    #[tokio::test]
    async fn test_throttler_with_storage_type() {
        let throttler = IOThrottler::with_storage_type(StorageType::HDD);
        assert_eq!(throttler.storage_type(), StorageType::HDD);
        assert_eq!(throttler.concurrency_limit(), DEFAULT_HDD_CONCURRENCY);
    }

    #[tokio::test]
    async fn test_throttler_with_custom_concurrency() {
        let throttler = IOThrottler::with_concurrency(8);
        assert_eq!(throttler.concurrency_limit(), 8);
    }

    #[tokio::test]
    async fn test_throttler_acquire_permit() {
        let throttler = IOThrottler::with_concurrency(2);
        
        // Should be able to acquire 2 permits
        let _permit1 = throttler.acquire().await.unwrap();
        let _permit2 = throttler.acquire().await.unwrap();
        
        // Third permit should not be available immediately
        assert!(throttler.try_acquire().is_none());
        
        // After dropping first permit, should be available again
        drop(_permit1);
        assert!(throttler.try_acquire().is_some());
    }

    #[tokio::test]
    async fn test_throttler_set_concurrency() {
        let throttler = IOThrottler::with_concurrency(2);
        assert_eq!(throttler.concurrency_limit(), 2);
        
        throttler.set_concurrency_limit(4);
        assert_eq!(throttler.concurrency_limit(), 4);
    }

    #[tokio::test]
    async fn test_throttler_auto_detect() {
        let temp_dir = TempDir::new().unwrap();
        let throttler = IOThrottler::new(temp_dir.path()).await.unwrap();
        
        // Should have detected some storage type
        let storage = throttler.storage_type();
        println!("Detected storage type: {:?}", storage);
        
        // Concurrency should be set based on storage type
        assert!(throttler.concurrency_limit() > 0);
    }

    #[tokio::test]
    async fn test_throttler_concurrent_operations() {
        let throttler = Arc::new(IOThrottler::with_concurrency(2));
        let mut handles = vec![];

        // Spawn 4 tasks trying to acquire permits
        for i in 0..4 {
            let throttler_clone = Arc::clone(&throttler);
            let handle = tokio::spawn(async move {
                let _permit = throttler_clone.acquire().await.unwrap();
                // Simulate some work
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                i
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            let _ = handle.await;
        }
    }
}
