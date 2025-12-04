// File Hasher Component
// Computes file fingerprints for duplicate detection and integrity verification
// Optimized for HDD-based storage with large buffer sizes

use anyhow::{Result, Context};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use sha2::{Sha256, Digest as Sha2Digest};
use blake3::Hasher as Blake3Hasher;
use tracing::{debug, info};

// ============================================================================
// Constants
// ============================================================================

/// Default buffer size for HDD-optimized I/O (1MB)
const DEFAULT_BUFFER_SIZE: usize = 1024 * 1024; // 1MB

/// Size to read for fast hash (first and last 1MB)
const FAST_HASH_CHUNK_SIZE: usize = 1024 * 1024; // 1MB

// ============================================================================
// Types
// ============================================================================

/// Hash algorithm selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// SHA-256 (widely supported, good security)
    Sha256,
    /// BLAKE3 (faster, modern)
    Blake3,
}

impl HashAlgorithm {
    /// Get the algorithm name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            HashAlgorithm::Sha256 => "sha256",
            HashAlgorithm::Blake3 => "blake3",
        }
    }
    
    /// Parse algorithm from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sha256" | "sha-256" => Some(HashAlgorithm::Sha256),
            "blake3" => Some(HashAlgorithm::Blake3),
            _ => None,
        }
    }
}

/// Fast hash result (first 1MB + last 1MB + file size)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FastHash {
    /// Hash bytes (32 bytes for both SHA-256 and BLAKE3)
    pub value: Vec<u8>,
}

impl FastHash {
    /// Create from byte vector
    pub fn new(value: Vec<u8>) -> Self {
        Self { value }
    }
    
    /// Get as hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.value)
    }
    
    /// Create from hex string
    pub fn from_hex(hex: &str) -> Result<Self> {
        let value = hex::decode(hex)
            .context("Failed to decode hex string")?;
        Ok(Self { value })
    }
}

/// Full file hash result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullHash {
    /// Hash algorithm used
    pub algorithm: HashAlgorithm,
    /// Hash bytes
    pub value: Vec<u8>,
}

impl FullHash {
    /// Create new full hash
    pub fn new(algorithm: HashAlgorithm, value: Vec<u8>) -> Self {
        Self { algorithm, value }
    }
    
    /// Get as hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.value)
    }
    
    /// Create from hex string and algorithm
    pub fn from_hex(algorithm: HashAlgorithm, hex: &str) -> Result<Self> {
        let value = hex::decode(hex)
            .context("Failed to decode hex string")?;
        Ok(Self { algorithm, value })
    }
}

// ============================================================================
// FileHasher
// ============================================================================

/// File hasher for computing fingerprints
///
/// Provides two types of hashing:
/// - Fast hash: First 1MB + last 1MB + file size (for quick duplicate detection)
/// - Full hash: Complete file hash (for exact duplicate verification)
///
/// Optimized for HDD storage with large buffer sizes to maximize sequential I/O.
///
/// # Requirements
/// - Requirements: 2.3, 3.2, 3.4
pub struct FileHasher {
    /// Buffer size for reading files (HDD-optimized)
    buffer_size: usize,
    
    /// Size of chunks to read for fast hash
    fast_hash_chunk_size: usize,
}

impl FileHasher {
    /// Create a new FileHasher with default settings
    ///
    /// Uses 1MB buffer size for HDD optimization.
    pub fn new() -> Self {
        Self {
            buffer_size: DEFAULT_BUFFER_SIZE,
            fast_hash_chunk_size: FAST_HASH_CHUNK_SIZE,
        }
    }
    
    /// Create a FileHasher with custom buffer size
    ///
    /// # Arguments
    /// * `buffer_size` - Buffer size in bytes (should be >= 1MB for HDD optimization)
    pub fn with_buffer_size(buffer_size: usize) -> Self {
        Self {
            buffer_size,
            fast_hash_chunk_size: FAST_HASH_CHUNK_SIZE,
        }
    }
    
    /// Compute fast hash (first 1MB + last 1MB + file size)
    ///
    /// This provides a quick fingerprint for duplicate detection without
    /// reading the entire file. It's not cryptographically secure but is
    /// sufficient for identifying likely duplicates.
    ///
    /// The hash includes:
    /// - First 1MB of the file (or entire file if smaller)
    /// - Last 1MB of the file (if file is large enough)
    /// - File size (encoded as 8 bytes)
    ///
    /// # Arguments
    /// * `path` - Path to the file to hash
    ///
    /// # Returns
    /// A FastHash containing the computed hash
    ///
    /// # Requirements
    /// - Requirements: 2.3
    ///
    /// # Errors
    /// Returns an error if:
    /// - File cannot be opened
    /// - File cannot be read
    /// - I/O error occurs
    pub async fn compute_fast_hash(&self, path: &Path) -> Result<FastHash> {
        debug!("Computing fast hash for: {:?}", path);
        
        let mut file = File::open(path)
            .await
            .with_context(|| format!("Failed to open file for fast hashing: {}", path.display()))?;
        
        // Get file size
        let metadata = file.metadata()
            .await
            .with_context(|| format!("Failed to read file metadata: {}", path.display()))?;
        let file_size = metadata.len();
        
        // Use BLAKE3 for fast hash (it's faster than SHA-256)
        let mut hasher = Blake3Hasher::new();
        
        // Hash first chunk
        let first_chunk_size = std::cmp::min(self.fast_hash_chunk_size as u64, file_size) as usize;
        let mut buffer = vec![0u8; first_chunk_size];
        
        file.read_exact(&mut buffer[..first_chunk_size])
            .await
            .with_context(|| format!("Failed to read first chunk: {}", path.display()))?;
        hasher.update(&buffer[..first_chunk_size]);
        
        // Hash last chunk if file is large enough
        if file_size > self.fast_hash_chunk_size as u64 {
            let last_chunk_size = std::cmp::min(self.fast_hash_chunk_size as u64, file_size) as usize;
            let seek_pos = file_size - last_chunk_size as u64;
            
            file.seek(std::io::SeekFrom::Start(seek_pos))
                .await
                .with_context(|| format!("Failed to seek to last chunk: {}", path.display()))?;
            
            let mut last_buffer = vec![0u8; last_chunk_size];
            file.read_exact(&mut last_buffer)
                .await
                .with_context(|| format!("Failed to read last chunk: {}", path.display()))?;
            hasher.update(&last_buffer);
        }
        
        // Include file size in hash
        hasher.update(&file_size.to_le_bytes());
        
        let hash = hasher.finalize();
        let hash_bytes = hash.as_bytes().to_vec();
        
        debug!("Fast hash computed: {} bytes, hash: {}", file_size, hex::encode(&hash_bytes));
        Ok(FastHash::new(hash_bytes))
    }
    
    /// Compute full file hash
    ///
    /// Reads the entire file and computes a cryptographic hash.
    /// Uses large buffer sizes optimized for sequential HDD reads.
    ///
    /// # Arguments
    /// * `path` - Path to the file to hash
    /// * `algorithm` - Hash algorithm to use (SHA-256 or BLAKE3)
    ///
    /// # Returns
    /// A FullHash containing the algorithm and computed hash
    ///
    /// # Requirements
    /// - Requirements: 3.2, 3.4
    ///
    /// # Errors
    /// Returns an error if:
    /// - File cannot be opened
    /// - File cannot be read
    /// - I/O error occurs
    pub async fn compute_full_hash(&self, path: &Path, algorithm: HashAlgorithm) -> Result<FullHash> {
        info!("Computing full hash ({:?}) for: {:?}", algorithm, path);
        
        let mut file = File::open(path)
            .await
            .with_context(|| format!("Failed to open file for full hashing: {}", path.display()))?;
        
        let metadata = file.metadata()
            .await
            .with_context(|| format!("Failed to read file metadata: {}", path.display()))?;
        let file_size = metadata.len();
        
        let hash_bytes = match algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                let mut buffer = vec![0u8; self.buffer_size];
                let mut total_read = 0u64;
                
                loop {
                    let n = file.read(&mut buffer)
                        .await
                        .with_context(|| format!("Failed to read file: {}", path.display()))?;
                    
                    if n == 0 {
                        break;
                    }
                    
                    hasher.update(&buffer[..n]);
                    total_read += n as u64;
                }
                
                debug!("SHA-256 hash computed: {} bytes read", total_read);
                hasher.finalize().to_vec()
            }
            HashAlgorithm::Blake3 => {
                let mut hasher = Blake3Hasher::new();
                let mut buffer = vec![0u8; self.buffer_size];
                let mut total_read = 0u64;
                
                loop {
                    let n = file.read(&mut buffer)
                        .await
                        .with_context(|| format!("Failed to read file: {}", path.display()))?;
                    
                    if n == 0 {
                        break;
                    }
                    
                    hasher.update(&buffer[..n]);
                    total_read += n as u64;
                }
                
                debug!("BLAKE3 hash computed: {} bytes read", total_read);
                hasher.finalize().as_bytes().to_vec()
            }
        };
        
        info!("Full hash computed: {} bytes, hash: {}", file_size, hex::encode(&hash_bytes));
        Ok(FullHash::new(algorithm, hash_bytes))
    }
    
    /// Verify a file's checksum
    ///
    /// Recomputes the hash and compares it with the expected value.
    ///
    /// # Arguments
    /// * `path` - Path to the file to verify
    /// * `expected` - Expected hash value
    ///
    /// # Returns
    /// `true` if the hash matches, `false` otherwise
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or hashed
    pub async fn verify_checksum(&self, path: &Path, expected: &FullHash) -> Result<bool> {
        debug!("Verifying checksum for: {:?}", path);
        
        let computed = self.compute_full_hash(path, expected.algorithm).await?;
        let matches = computed.value == expected.value;
        
        if matches {
            debug!("Checksum verification passed");
        } else {
            debug!("Checksum verification failed: expected {}, got {}", 
                   hex::encode(&expected.value), 
                   hex::encode(&computed.value));
        }
        
        Ok(matches)
    }
}

impl Default for FileHasher {
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
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_fast_hash_small_file() {
        // Create a small test file (< 1MB)
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = b"Hello, world! This is a test file.";
        temp_file.write_all(content).unwrap();
        temp_file.flush().unwrap();
        
        let hasher = FileHasher::new();
        let hash = hasher.compute_fast_hash(temp_file.path()).await.unwrap();
        
        // Hash should be 32 bytes (BLAKE3)
        assert_eq!(hash.value.len(), 32);
        
        // Same file should produce same hash
        let hash2 = hasher.compute_fast_hash(temp_file.path()).await.unwrap();
        assert_eq!(hash.value, hash2.value);
    }
    
    #[tokio::test]
    async fn test_fast_hash_large_file() {
        // Create a large test file (> 2MB)
        let mut temp_file = NamedTempFile::new().unwrap();
        let chunk = vec![0u8; 1024 * 1024]; // 1MB of zeros
        temp_file.write_all(&chunk).unwrap();
        temp_file.write_all(&chunk).unwrap();
        temp_file.write_all(&chunk).unwrap(); // 3MB total
        temp_file.flush().unwrap();
        
        let hasher = FileHasher::new();
        let hash = hasher.compute_fast_hash(temp_file.path()).await.unwrap();
        
        // Hash should be 32 bytes
        assert_eq!(hash.value.len(), 32);
    }
    
    #[tokio::test]
    async fn test_full_hash_sha256() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = b"Test content for SHA-256 hashing";
        temp_file.write_all(content).unwrap();
        temp_file.flush().unwrap();
        
        let hasher = FileHasher::new();
        let hash = hasher.compute_full_hash(temp_file.path(), HashAlgorithm::Sha256).await.unwrap();
        
        // SHA-256 produces 32 bytes
        assert_eq!(hash.value.len(), 32);
        assert_eq!(hash.algorithm, HashAlgorithm::Sha256);
        
        // Same file should produce same hash
        let hash2 = hasher.compute_full_hash(temp_file.path(), HashAlgorithm::Sha256).await.unwrap();
        assert_eq!(hash.value, hash2.value);
    }
    
    #[tokio::test]
    async fn test_full_hash_blake3() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = b"Test content for BLAKE3 hashing";
        temp_file.write_all(content).unwrap();
        temp_file.flush().unwrap();
        
        let hasher = FileHasher::new();
        let hash = hasher.compute_full_hash(temp_file.path(), HashAlgorithm::Blake3).await.unwrap();
        
        // BLAKE3 produces 32 bytes
        assert_eq!(hash.value.len(), 32);
        assert_eq!(hash.algorithm, HashAlgorithm::Blake3);
    }
    
    #[tokio::test]
    async fn test_verify_checksum() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = b"Test content for checksum verification";
        temp_file.write_all(content).unwrap();
        temp_file.flush().unwrap();
        
        let hasher = FileHasher::new();
        let hash = hasher.compute_full_hash(temp_file.path(), HashAlgorithm::Sha256).await.unwrap();
        
        // Verification should pass
        let valid = hasher.verify_checksum(temp_file.path(), &hash).await.unwrap();
        assert!(valid);
        
        // Create a different hash - verification should fail
        let wrong_hash = FullHash::new(HashAlgorithm::Sha256, vec![0u8; 32]);
        let valid = hasher.verify_checksum(temp_file.path(), &wrong_hash).await.unwrap();
        assert!(!valid);
    }
    
    #[tokio::test]
    async fn test_hash_hex_conversion() {
        let hash_bytes = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
        let fast_hash = FastHash::new(hash_bytes.clone());
        
        let hex = fast_hash.to_hex();
        assert_eq!(hex, "0123456789abcdef");
        
        let decoded = FastHash::from_hex(&hex).unwrap();
        assert_eq!(decoded.value, hash_bytes);
    }
}
