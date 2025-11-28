// Placeholder module for filesystem management
// TODO: Implement atomic moves, renamer, hashing

pub mod manager;
pub mod atomic;
pub mod renamer;
pub mod hasher;

pub use manager::FileSystemManager;
pub use atomic::AtomicMover;
pub use renamer::FileRenamer;
pub use hasher::FileHasher;
