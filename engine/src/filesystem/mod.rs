// Placeholder module for filesystem management
// TODO: Implement atomic moves, renamer, hashing

pub mod manager;
pub mod atomic;
pub mod renamer;
pub mod hasher;
pub mod links;
pub mod posix;
pub mod optimizations;
pub mod io_batch;
pub mod io_throttle;
pub mod sequential;

pub use manager::FileSystemManager;
pub use atomic::AtomicMover;
pub use renamer::FileRenamer;
pub use hasher::FileHasher;
pub use links::{LinkResolver, LinkInfo};
pub use posix::PosixPathHandler;
pub use optimizations::{FilesystemOptimizer, FilesystemType, FilesystemInfo};
pub use io_batch::{IOBatchExecutor, IOOperation, IOOperationType, IOOperationResult};
pub use io_throttle::{IOThrottler, StorageType, IOPermit};
pub use sequential::{SequentialDirectoryReader, SequentialEntry};
