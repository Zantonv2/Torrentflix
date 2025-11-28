// Placeholder file hasher
// TODO: Implement actual file hashing logic

use anyhow::Result;

pub struct FileHasher {
    algorithm: String,
}

impl FileHasher {
    pub fn new(algorithm: String) -> Self {
        Self { algorithm }
    }

    pub async fn hash_file(&self, path: &str) -> Result<String> {
        Ok("mock_hash".to_string())
    }

    pub async fn hash_bytes(&self, data: &[u8]) -> Result<String> {
        Ok("mock_hash".to_string())
    }
}

impl Default for FileHasher {
    fn default() -> Self {
        Self::new("sha256".to_string())
    }
}
