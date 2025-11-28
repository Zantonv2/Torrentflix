// Placeholder atomic mover
// TODO: Implement actual atomic file operations

use anyhow::Result;
use std::path::Path;

pub struct AtomicMover;

impl AtomicMover {
    pub fn new() -> Self {
        Self
    }

    pub async fn atomic_move(&self, from: &Path, to: &Path) -> Result<()> {
        Ok(())
    }

    pub async fn atomic_write(&self, path: &Path, data: &[u8]) -> Result<()> {
        Ok(())
    }
}

impl Default for AtomicMover {
    fn default() -> Self {
        Self::new()
    }
}
