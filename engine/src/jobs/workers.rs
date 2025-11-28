// Placeholder background worker
// TODO: Implement actual background worker logic

use anyhow::Result;

pub struct BackgroundWorker {
    enabled: bool,
}

impl BackgroundWorker {
    pub fn new() -> Self {
        Self {
            enabled: true,
        }
    }

    pub async fn start(&self) -> Result<()> {
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        Ok(())
    }

    pub async fn process_job(&self, job_id: String) -> Result<()> {
        Ok(())
    }
}

impl Default for BackgroundWorker {
    fn default() -> Self {
        Self::new()
    }
}
