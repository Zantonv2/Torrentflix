// Placeholder config migration
// TODO: Implement actual configuration migration

use anyhow::Result;

pub struct ConfigMigration {
    current_version: u32,
}

impl ConfigMigration {
    pub fn new(current_version: u32) -> Self {
        Self { current_version }
    }

    pub async fn migrate(&self) -> Result<()> {
        Ok(())
    }

    pub async fn check_version(&self) -> Result<u32> {
        Ok(self.current_version)
    }
}

impl Default for ConfigMigration {
    fn default() -> Self {
        Self::new(1)
    }
}
