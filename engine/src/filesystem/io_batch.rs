// I/O Batching for HDD Optimization
// Batches file operations to minimize seeks and maximize sequential I/O
// Sorts operations by physical disk location when possible

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info};

// ============================================================================
// Constants
// ============================================================================

/// Default buffer size for batched I/O operations (1MB)
const DEFAULT_BATCH_BUFFER_SIZE: usize = 1024 * 1024; // 1MB

/// Default maximum batch size (number of operations to batch together)
const DEFAULT_MAX_BATCH_SIZE: usize = 100;

// ============================================================================
// Types
// ============================================================================

/// Type of I/O operation to batch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IOOperationType {
    /// Copy file operation
    Copy,
    /// Move file operation
    Move,
    /// Delete file operation
    Delete,
    /// Read file operation
    Read,
}

/// A single I/O operation to be batched
#[derive(Debug, Clone)]
pub struct IOOperation {
    /// Type of operation
    pub op_type: IOOperationType,
    /// Source path (for copy/move/read)
    pub source: PathBuf,
    /// Destination path (for copy/move)
    pub destination: Option<PathBuf>,
    /// Priority (lower = higher priority, executed first)
    pub priority: u32,
    /// Estimated size in bytes (for sorting by physical location)
    pub estimated_size: u64,
}

impl IOOperation {
    /// Create a new copy operation
    pub fn copy(source: PathBuf, destination: PathBuf, priority: u32) -> Self {
        Self {
            op_type: IOOperationType::Copy,
            source,
            destination: Some(destination),
            priority,
            estimated_size: 0,
        }
    }

    /// Create a new move operation
    pub fn move_op(source: PathBuf, destination: PathBuf, priority: u32) -> Self {
        Self {
            op_type: IOOperationType::Move,
            source,
            destination: Some(destination),
            priority,
            estimated_size: 0,
        }
    }

    /// Create a new delete operation
    pub fn delete(source: PathBuf, priority: u32) -> Self {
        Self {
            op_type: IOOperationType::Delete,
            source,
            destination: None,
            priority,
            estimated_size: 0,
        }
    }

    /// Create a new read operation
    pub fn read(source: PathBuf, priority: u32) -> Self {
        Self {
            op_type: IOOperationType::Read,
            source,
            destination: None,
            priority,
            estimated_size: 0,
        }
    }

    /// Set the estimated size for sorting
    pub fn with_size(mut self, size: u64) -> Self {
        self.estimated_size = size;
        self
    }
}

/// Result of a batched I/O operation
#[derive(Debug, Clone)]
pub struct IOOperationResult {
    /// The operation that was executed
    pub operation: IOOperation,
    /// Whether the operation succeeded
    pub success: bool,
    /// Error message if operation failed
    pub error: Option<String>,
    /// Bytes processed
    pub bytes_processed: u64,
}

/// I/O Batch Executor
///
/// Batches file operations to minimize seeks and maximize sequential I/O.
/// Operations are sorted by priority and estimated physical location.
///
/// # Requirements
/// - Requirements: 27.1, 27.2, 27.4
pub struct IOBatchExecutor {
    /// Buffer size for I/O operations
    buffer_size: usize,
    /// Maximum number of operations to batch together
    max_batch_size: usize,
    /// Pending operations
    pending_operations: Vec<IOOperation>,
}

impl IOBatchExecutor {
    /// Create a new I/O batch executor with default settings
    pub fn new() -> Self {
        Self {
            buffer_size: DEFAULT_BATCH_BUFFER_SIZE,
            max_batch_size: DEFAULT_MAX_BATCH_SIZE,
            pending_operations: Vec::new(),
        }
    }

    /// Create a new I/O batch executor with custom settings
    pub fn with_settings(buffer_size: usize, max_batch_size: usize) -> Self {
        Self {
            buffer_size,
            max_batch_size,
            pending_operations: Vec::new(),
        }
    }

    /// Add an operation to the batch
    pub fn add_operation(&mut self, operation: IOOperation) {
        self.pending_operations.push(operation);
    }

    /// Add multiple operations to the batch
    pub fn add_operations(&mut self, operations: Vec<IOOperation>) {
        self.pending_operations.extend(operations);
    }

    /// Get the number of pending operations
    pub fn pending_count(&self) -> usize {
        self.pending_operations.len()
    }

    /// Sort operations by priority and estimated physical location
    ///
    /// This attempts to minimize seeks by grouping operations on similar
    /// paths together. Operations are sorted by:
    /// 1. Priority (lower = higher priority)
    /// 2. Parent directory (to group operations on same directory)
    /// 3. Source path (for consistent ordering)
    fn sort_operations(&mut self) {
        self.pending_operations.sort_by(|a, b| {
            // First, sort by priority
            match a.priority.cmp(&b.priority) {
                std::cmp::Ordering::Equal => {
                    // Then, sort by parent directory to group operations
                    let a_parent = a.source.parent().unwrap_or_else(|| Path::new(""));
                    let b_parent = b.source.parent().unwrap_or_else(|| Path::new(""));
                    
                    match a_parent.cmp(b_parent) {
                        std::cmp::Ordering::Equal => {
                            // Finally, sort by source path
                            a.source.cmp(&b.source)
                        }
                        other => other,
                    }
                }
                other => other,
            }
        });
    }

    /// Execute all pending operations in batches
    ///
    /// Operations are sorted by priority and physical location before execution.
    /// Returns results for all operations.
    ///
    /// # Returns
    /// A vector of IOOperationResult for each operation
    ///
    /// # Errors
    /// Returns an error if batch execution fails, but continues processing
    /// remaining operations.
    pub async fn execute_batch(&mut self) -> Result<Vec<IOOperationResult>> {
        if self.pending_operations.is_empty() {
            return Ok(Vec::new());
        }

        info!("Executing batch of {} I/O operations", self.pending_operations.len());

        // Sort operations for optimal I/O patterns
        self.sort_operations();

        let mut results = Vec::new();

        // Process operations in batches
        for batch in self.pending_operations.chunks(self.max_batch_size) {
            for operation in batch {
                let result = self.execute_operation(operation).await;
                results.push(result);
            }
        }

        self.pending_operations.clear();

        info!("Batch execution completed: {} operations processed", results.len());
        Ok(results)
    }

    /// Execute a single operation
    async fn execute_operation(&self, operation: &IOOperation) -> IOOperationResult {
        debug!("Executing I/O operation: {:?}", operation.op_type);

        let result = match operation.op_type {
            IOOperationType::Copy => {
                self.execute_copy(operation).await
            }
            IOOperationType::Move => {
                self.execute_move(operation).await
            }
            IOOperationType::Delete => {
                self.execute_delete(operation).await
            }
            IOOperationType::Read => {
                self.execute_read(operation).await
            }
        };

        match result {
            Ok(bytes_processed) => {
                debug!("I/O operation succeeded: {} bytes processed", bytes_processed);
                IOOperationResult {
                    operation: operation.clone(),
                    success: true,
                    error: None,
                    bytes_processed,
                }
            }
            Err(e) => {
                debug!("I/O operation failed: {}", e);
                IOOperationResult {
                    operation: operation.clone(),
                    success: false,
                    error: Some(e.to_string()),
                    bytes_processed: 0,
                }
            }
        }
    }

    /// Execute a copy operation with large buffer
    async fn execute_copy(&self, operation: &IOOperation) -> Result<u64> {
        let dest = operation.destination.as_ref()
            .context("Copy operation missing destination")?;

        // Use large buffer for sequential I/O
        let mut source_file = fs::File::open(&operation.source)
            .await
            .with_context(|| format!("Failed to open source: {}", operation.source.display()))?;

        let mut dest_file = fs::File::create(dest)
            .await
            .with_context(|| format!("Failed to create destination: {}", dest.display()))?;

        let mut buffer = vec![0u8; self.buffer_size];
        let mut total_bytes = 0u64;

        loop {
            let n = tokio::io::AsyncReadExt::read(&mut source_file, &mut buffer)
                .await
                .with_context(|| format!("Failed to read from: {}", operation.source.display()))?;

            if n == 0 {
                break;
            }

            tokio::io::AsyncWriteExt::write_all(&mut dest_file, &buffer[..n])
                .await
                .with_context(|| format!("Failed to write to: {}", dest.display()))?;

            total_bytes += n as u64;
        }

        Ok(total_bytes)
    }

    /// Execute a move operation
    async fn execute_move(&self, operation: &IOOperation) -> Result<u64> {
        let dest = operation.destination.as_ref()
            .context("Move operation missing destination")?;

        // Try atomic rename first (same filesystem)
        match fs::rename(&operation.source, dest).await {
            Ok(_) => {
                // Get file size for reporting
                let metadata = fs::metadata(dest)
                    .await
                    .unwrap_or_else(|_| std::fs::Metadata::from(std::fs::metadata(".").unwrap()));
                Ok(metadata.len())
            }
            Err(_) => {
                // Fall back to copy + delete (cross-filesystem)
                let bytes = self.execute_copy(operation).await?;
                fs::remove_file(&operation.source)
                    .await
                    .with_context(|| format!("Failed to delete source after copy: {}", operation.source.display()))?;
                Ok(bytes)
            }
        }
    }

    /// Execute a delete operation
    async fn execute_delete(&self, operation: &IOOperation) -> Result<u64> {
        // Get file size before deletion for reporting
        let metadata = fs::metadata(&operation.source)
            .await
            .with_context(|| format!("Failed to get metadata: {}", operation.source.display()))?;
        let size = metadata.len();

        fs::remove_file(&operation.source)
            .await
            .with_context(|| format!("Failed to delete: {}", operation.source.display()))?;

        Ok(size)
    }

    /// Execute a read operation
    async fn execute_read(&self, operation: &IOOperation) -> Result<u64> {
        let mut file = fs::File::open(&operation.source)
            .await
            .with_context(|| format!("Failed to open: {}", operation.source.display()))?;

        let mut buffer = vec![0u8; self.buffer_size];
        let mut total_bytes = 0u64;

        loop {
            let n = tokio::io::AsyncReadExt::read(&mut file, &mut buffer)
                .await
                .with_context(|| format!("Failed to read: {}", operation.source.display()))?;

            if n == 0 {
                break;
            }

            total_bytes += n as u64;
        }

        Ok(total_bytes)
    }

    /// Clear all pending operations without executing them
    pub fn clear(&mut self) {
        self.pending_operations.clear();
    }
}

impl Default for IOBatchExecutor {
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
    async fn test_io_operation_creation() {
        let source = PathBuf::from("/tmp/source.txt");
        let dest = PathBuf::from("/tmp/dest.txt");

        let copy_op = IOOperation::copy(source.clone(), dest.clone(), 1);
        assert_eq!(copy_op.op_type, IOOperationType::Copy);
        assert_eq!(copy_op.priority, 1);

        let move_op = IOOperation::move_op(source.clone(), dest.clone(), 2);
        assert_eq!(move_op.op_type, IOOperationType::Move);

        let delete_op = IOOperation::delete(source.clone(), 3);
        assert_eq!(delete_op.op_type, IOOperationType::Delete);

        let read_op = IOOperation::read(source, 4);
        assert_eq!(read_op.op_type, IOOperationType::Read);
    }

    #[tokio::test]
    async fn test_batch_executor_creation() {
        let executor = IOBatchExecutor::new();
        assert_eq!(executor.pending_count(), 0);
        assert_eq!(executor.buffer_size, DEFAULT_BATCH_BUFFER_SIZE);
        assert_eq!(executor.max_batch_size, DEFAULT_MAX_BATCH_SIZE);
    }

    #[tokio::test]
    async fn test_add_operations() {
        let mut executor = IOBatchExecutor::new();
        let source = PathBuf::from("/tmp/source.txt");
        let dest = PathBuf::from("/tmp/dest.txt");

        executor.add_operation(IOOperation::copy(source.clone(), dest.clone(), 1));
        assert_eq!(executor.pending_count(), 1);

        executor.add_operations(vec![
            IOOperation::delete(source.clone(), 2),
            IOOperation::read(source, 3),
        ]);
        assert_eq!(executor.pending_count(), 3);
    }

    #[tokio::test]
    async fn test_copy_operation() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");

        // Create source file
        std_fs::write(&source, b"test content").unwrap();

        let mut executor = IOBatchExecutor::new();
        executor.add_operation(IOOperation::copy(source.clone(), dest.clone(), 1));

        let results = executor.execute_batch().await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert!(dest.exists());
        assert_eq!(std_fs::read(&dest).unwrap(), b"test content");
    }

    #[tokio::test]
    async fn test_move_operation() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");

        // Create source file
        std_fs::write(&source, b"test content").unwrap();

        let mut executor = IOBatchExecutor::new();
        executor.add_operation(IOOperation::move_op(source.clone(), dest.clone(), 1));

        let results = executor.execute_batch().await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert!(!source.exists());
        assert!(dest.exists());
    }

    #[tokio::test]
    async fn test_delete_operation() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");

        // Create source file
        std_fs::write(&source, b"test content").unwrap();

        let mut executor = IOBatchExecutor::new();
        executor.add_operation(IOOperation::delete(source.clone(), 1));

        let results = executor.execute_batch().await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert!(!source.exists());
    }

    #[tokio::test]
    async fn test_read_operation() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");

        // Create source file
        std_fs::write(&source, b"test content").unwrap();

        let mut executor = IOBatchExecutor::new();
        executor.add_operation(IOOperation::read(source, 1));

        let results = executor.execute_batch().await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert_eq!(results[0].bytes_processed, 12); // "test content" is 12 bytes
    }

    #[tokio::test]
    async fn test_operation_sorting() {
        let mut executor = IOBatchExecutor::new();

        // Add operations with different priorities
        executor.add_operation(IOOperation::read(PathBuf::from("/tmp/a.txt"), 3));
        executor.add_operation(IOOperation::read(PathBuf::from("/tmp/b.txt"), 1));
        executor.add_operation(IOOperation::read(PathBuf::from("/tmp/c.txt"), 2));

        executor.sort_operations();

        // Should be sorted by priority
        assert_eq!(executor.pending_operations[0].priority, 1);
        assert_eq!(executor.pending_operations[1].priority, 2);
        assert_eq!(executor.pending_operations[2].priority, 3);
    }

    #[tokio::test]
    async fn test_clear_operations() {
        let mut executor = IOBatchExecutor::new();
        executor.add_operation(IOOperation::read(PathBuf::from("/tmp/a.txt"), 1));
        executor.add_operation(IOOperation::read(PathBuf::from("/tmp/b.txt"), 2));

        assert_eq!(executor.pending_count(), 2);
        executor.clear();
        assert_eq!(executor.pending_count(), 0);
    }

    #[tokio::test]
    async fn test_empty_batch_execution() {
        let mut executor = IOBatchExecutor::new();
        let results = executor.execute_batch().await.unwrap();
        assert_eq!(results.len(), 0);
    }
}
