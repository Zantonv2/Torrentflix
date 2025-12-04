// Quality scoring component for FileVersion ranking
// Scores file versions based on resolution, codec, source, and size

use super::models::{FileVersion, QualityScore, SourceType};

/// Configurable weights for quality scoring
#[derive(Debug, Clone)]
pub struct QualityWeights {
    pub resolution: f32,
    pub codec: f32,
    pub source: f32,
    pub size: f32,
}

impl Default for QualityWeights {
    fn default() -> Self {
        Self {
            resolution: 0.4,  // Resolution is most important
            codec: 0.25,      // Codec efficiency matters
            source: 0.25,     // Source quality matters
            size: 0.1,        // Size is least important (prefer reasonable sizes)
        }
    }
}

/// Quality scorer for ranking FileVersions
pub struct QualityScorer {
    weights: QualityWeights,
}

impl QualityScorer {
    /// Create a new QualityScorer with custom weights
    pub fn new(weights: QualityWeights) -> Self {
        Self { weights }
    }

    /// Create a QualityScorer with default weights
    pub fn with_default_weights() -> Self {
        Self::new(QualityWeights::default())
    }

    /// Score a single FileVersion
    /// Returns a QualityScore with total and component scores
    pub fn score_version(&self, version: &FileVersion) -> QualityScore {
        let resolution_score = self.score_resolution(version);
        let codec_score = self.score_codec(version);
        let source_score = self.score_source(version);
        let size_score = self.score_size(version);

        let total = resolution_score * self.weights.resolution
            + codec_score * self.weights.codec
            + source_score * self.weights.source
            + size_score * self.weights.size;

        QualityScore {
            total,
            resolution_score,
            codec_score,
            source_score,
            size_score,
        }
    }

    /// Score resolution (4K > 1080p > 720p > SD)
    fn score_resolution(&self, version: &FileVersion) -> f32 {
        if let Some(ref metadata) = version.technical_metadata {
            if let Some(ref video) = metadata.video {
                let resolution = &video.resolution;
                
                // 4K (2160p)
                if resolution.height >= 2160 {
                    return 1.0;
                }
                // 1080p
                if resolution.height >= 1080 {
                    return 0.8;
                }
                // 720p
                if resolution.height >= 720 {
                    return 0.6;
                }
                // 480p (SD)
                if resolution.height >= 480 {
                    return 0.4;
                }
                // Lower than SD
                return 0.2;
            }
        }
        
        // No metadata available
        0.3
    }

    /// Score codec (H.265 > H.264 > others)
    fn score_codec(&self, version: &FileVersion) -> f32 {
        if let Some(ref metadata) = version.technical_metadata {
            if let Some(ref video) = metadata.video {
                let codec = video.codec.to_lowercase();
                
                // H.265/HEVC (most efficient)
                if codec.contains("h.265") || codec.contains("hevc") || codec.contains("x265") {
                    return 1.0;
                }
                // H.264/AVC (standard)
                if codec.contains("h.264") || codec.contains("avc") || codec.contains("x264") {
                    return 0.8;
                }
                // VP9 (good alternative)
                if codec.contains("vp9") {
                    return 0.75;
                }
                // Older codecs
                if codec.contains("xvid") || codec.contains("divx") || codec.contains("mpeg") {
                    return 0.4;
                }
                // Unknown codec
                return 0.5;
            }
        }
        
        // No metadata available
        0.5
    }

    /// Score source (BluRay > WEB-DL > HDTV > DVD)
    fn score_source(&self, version: &FileVersion) -> f32 {
        match version.source_type {
            Some(SourceType::BluRay) => 1.0,
            Some(SourceType::WebDl) => 0.9,
            Some(SourceType::Hdtv) => 0.7,
            Some(SourceType::Dvd) => 0.6,
            Some(SourceType::Unknown) | None => 0.5,
        }
    }

    /// Score size (prefer reasonable sizes, penalize extremes)
    /// Optimal size depends on resolution
    fn score_size(&self, version: &FileVersion) -> f32 {
        let size_gb = version.file_size as f64 / (1024.0 * 1024.0 * 1024.0);
        
        // Determine optimal size range based on resolution
        let (min_optimal, max_optimal) = if let Some(ref metadata) = version.technical_metadata {
            if let Some(ref video) = metadata.video {
                let height = video.resolution.height;
                
                if height >= 2160 {
                    // 4K: 15-50 GB is optimal
                    (15.0, 50.0)
                } else if height >= 1080 {
                    // 1080p: 2-8 GB is optimal
                    (2.0, 8.0)
                } else if height >= 720 {
                    // 720p: 1-4 GB is optimal
                    (1.0, 4.0)
                } else {
                    // SD: 0.5-2 GB is optimal
                    (0.5, 2.0)
                }
            } else {
                // No video info, use default range
                (1.0, 10.0)
            }
        } else {
            // No metadata, use default range
            (1.0, 10.0)
        };

        // Score based on how close to optimal range
        if size_gb >= min_optimal && size_gb <= max_optimal {
            // Perfect size
            1.0
        } else if size_gb < min_optimal {
            // Too small - might be low quality
            let ratio = (size_gb / min_optimal) as f32;
            0.3 + (ratio * 0.7) // Scale from 0.3 to 1.0
        } else {
            // Too large - might be bloated
            let ratio = (max_optimal / size_gb) as f32;
            0.3 + (ratio * 0.7) // Scale from 0.3 down as size increases
        }
    }
}

impl Default for QualityScorer {
    fn default() -> Self {
        Self::with_default_weights()
    }
}

/// Ranked version with its quality score
#[derive(Debug, Clone)]
pub struct RankedVersion {
    pub version_id: super::models::FileVersionId,
    pub score: QualityScore,
    pub is_preferred: bool,
}

impl QualityScorer {
    /// Rank multiple versions by quality score
    /// Respects user-set preferred flag - preferred versions always rank first
    /// Returns versions sorted by quality (highest first)
    pub fn rank_versions(&self, versions: &[FileVersion]) -> Vec<RankedVersion> {
        let mut ranked: Vec<RankedVersion> = versions
            .iter()
            .map(|v| RankedVersion {
                version_id: v.id,
                score: self.score_version(v),
                is_preferred: v.is_preferred,
            })
            .collect();

        // Sort by preferred flag first, then by quality score
        ranked.sort_by(|a, b| {
            // Preferred versions always come first
            match (a.is_preferred, b.is_preferred) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                // If both preferred or both not preferred, sort by score
                _ => b.score.total.partial_cmp(&a.score.total)
                    .unwrap_or(std::cmp::Ordering::Equal),
            }
        });

        ranked
    }

    /// Compare two versions and return the better one
    /// Respects preferred flag
    pub fn compare_versions(&self, a: &FileVersion, b: &FileVersion) -> std::cmp::Ordering {
        // Preferred always wins
        match (a.is_preferred, b.is_preferred) {
            (true, false) => return std::cmp::Ordering::Greater,
            (false, true) => return std::cmp::Ordering::Less,
            _ => {}
        }

        // Compare by quality score
        let score_a = self.score_version(a);
        let score_b = self.score_version(b);
        
        score_a.total.partial_cmp(&score_b.total)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::models::{
        FileVersionId, MediaItemId, LibraryRootId, FileStatus, HashingStatus,
        FilesystemInfo, TechnicalMetadata, VideoInfo, Resolution,
    };
    use std::path::PathBuf;
    use chrono::Utc;

    fn create_test_version(
        resolution: Resolution,
        codec: &str,
        source: SourceType,
        size_gb: f64,
    ) -> FileVersion {
        let size_bytes = (size_gb * 1024.0 * 1024.0 * 1024.0) as u64;
        
        FileVersion {
            id: FileVersionId(1),
            media_item_id: MediaItemId(1),
            library_root_id: LibraryRootId(1),
            relative_path: PathBuf::from("test.mkv"),
            absolute_path: PathBuf::from("/library/test.mkv"),
            file_size: size_bytes,
            status: FileStatus::Present,
            is_preferred: false,
            technical_metadata: Some(TechnicalMetadata {
                container: "mkv".to_string(),
                video: Some(VideoInfo {
                    codec: codec.to_string(),
                    resolution,
                    framerate: 24.0,
                    bitrate: 5000000,
                    color_space: Some("bt709".to_string()),
                }),
                audio_tracks: vec![],
                subtitle_tracks: vec![],
                duration: 7200,
                bitrate: 5000000,
                file_size: size_bytes,
            }),
            quality_label: None,
            release_group: None,
            source_type: Some(source),
            fast_hash: None,
            full_hash: None,
            hashing_status: HashingStatus::Pending,
            checksum: None,
            last_verified_at: None,
            corruption_detected_at: None,
            filesystem_info: FilesystemInfo {
                is_symlink: false,
                symlink_target: None,
                is_hardlink: false,
                inode: None,
            },
            added_at: Utc::now(),
            last_seen_at: Utc::now(),
            trashed_at: None,
            original_path: None,
            import_source: None,
        }
    }

    #[test]
    fn test_resolution_scoring() {
        let scorer = QualityScorer::with_default_weights();
        
        // 4K should score highest
        let v4k = create_test_version(
            Resolution::new(3840, 2160),
            "H.265",
            SourceType::BluRay,
            20.0,
        );
        let score_4k = scorer.score_resolution(&v4k);
        assert_eq!(score_4k, 1.0);
        
        // 1080p should score 0.8
        let v1080 = create_test_version(
            Resolution::new(1920, 1080),
            "H.264",
            SourceType::BluRay,
            5.0,
        );
        let score_1080 = scorer.score_resolution(&v1080);
        assert_eq!(score_1080, 0.8);
        
        // 720p should score 0.6
        let v720 = create_test_version(
            Resolution::new(1280, 720),
            "H.264",
            SourceType::WebDl,
            2.0,
        );
        let score_720 = scorer.score_resolution(&v720);
        assert_eq!(score_720, 0.6);
    }

    #[test]
    fn test_codec_scoring() {
        let scorer = QualityScorer::with_default_weights();
        
        // H.265 should score highest
        let v265 = create_test_version(
            Resolution::new(1920, 1080),
            "H.265",
            SourceType::BluRay,
            5.0,
        );
        let score_265 = scorer.score_codec(&v265);
        assert_eq!(score_265, 1.0);
        
        // H.264 should score 0.8
        let v264 = create_test_version(
            Resolution::new(1920, 1080),
            "H.264",
            SourceType::BluRay,
            5.0,
        );
        let score_264 = scorer.score_codec(&v264);
        assert_eq!(score_264, 0.8);
    }

    #[test]
    fn test_source_scoring() {
        let scorer = QualityScorer::with_default_weights();
        
        let v_bluray = create_test_version(
            Resolution::new(1920, 1080),
            "H.264",
            SourceType::BluRay,
            5.0,
        );
        assert_eq!(scorer.score_source(&v_bluray), 1.0);
        
        let v_webdl = create_test_version(
            Resolution::new(1920, 1080),
            "H.264",
            SourceType::WebDl,
            5.0,
        );
        assert_eq!(scorer.score_source(&v_webdl), 0.9);
    }

    #[test]
    fn test_size_scoring() {
        let scorer = QualityScorer::with_default_weights();
        
        // 1080p with optimal size (5GB) should score 1.0
        let v_optimal = create_test_version(
            Resolution::new(1920, 1080),
            "H.264",
            SourceType::BluRay,
            5.0,
        );
        let score_optimal = scorer.score_size(&v_optimal);
        assert_eq!(score_optimal, 1.0);
        
        // 1080p with too small size should score lower
        let v_small = create_test_version(
            Resolution::new(1920, 1080),
            "H.264",
            SourceType::BluRay,
            0.5,
        );
        let score_small = scorer.score_size(&v_small);
        assert!(score_small < 1.0);
        assert!(score_small > 0.3);
    }

    #[test]
    fn test_total_score_calculation() {
        let scorer = QualityScorer::with_default_weights();
        
        // Perfect version: 4K, H.265, BluRay, optimal size
        let perfect = create_test_version(
            Resolution::new(3840, 2160),
            "H.265",
            SourceType::BluRay,
            30.0,
        );
        let score = scorer.score_version(&perfect);
        
        // Should have high total score
        assert!(score.total > 0.9);
        assert_eq!(score.resolution_score, 1.0);
        assert_eq!(score.codec_score, 1.0);
        assert_eq!(score.source_score, 1.0);
    }

    #[test]
    fn test_rank_versions_by_quality() {
        let scorer = QualityScorer::with_default_weights();
        
        // Create versions with different qualities
        let v4k = create_test_version(
            Resolution::new(3840, 2160),
            "H.265",
            SourceType::BluRay,
            30.0,
        );
        let v1080 = create_test_version(
            Resolution::new(1920, 1080),
            "H.264",
            SourceType::WebDl,
            5.0,
        );
        let v720 = create_test_version(
            Resolution::new(1280, 720),
            "H.264",
            SourceType::Hdtv,
            2.0,
        );
        
        let versions = vec![v720.clone(), v1080.clone(), v4k.clone()];
        let ranked = scorer.rank_versions(&versions);
        
        // Should be sorted by quality: 4K > 1080p > 720p
        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].version_id, v4k.id);
        assert_eq!(ranked[1].version_id, v1080.id);
        assert_eq!(ranked[2].version_id, v720.id);
    }

    #[test]
    fn test_preferred_version_override() {
        let scorer = QualityScorer::with_default_weights();
        
        // Create a high-quality version
        let mut v4k = create_test_version(
            Resolution::new(3840, 2160),
            "H.265",
            SourceType::BluRay,
            30.0,
        );
        
        // Create a lower-quality version but mark it as preferred
        let mut v720 = create_test_version(
            Resolution::new(1280, 720),
            "H.264",
            SourceType::Hdtv,
            2.0,
        );
        v720.is_preferred = true;
        
        let versions = vec![v4k.clone(), v720.clone()];
        let ranked = scorer.rank_versions(&versions);
        
        // Preferred version should rank first despite lower quality
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].version_id, v720.id);
        assert!(ranked[0].is_preferred);
        assert_eq!(ranked[1].version_id, v4k.id);
        assert!(!ranked[1].is_preferred);
    }

    #[test]
    fn test_compare_versions() {
        let scorer = QualityScorer::with_default_weights();
        
        let v4k = create_test_version(
            Resolution::new(3840, 2160),
            "H.265",
            SourceType::BluRay,
            30.0,
        );
        let v1080 = create_test_version(
            Resolution::new(1920, 1080),
            "H.264",
            SourceType::WebDl,
            5.0,
        );
        
        // 4K should be greater than 1080p
        let ordering = scorer.compare_versions(&v4k, &v1080);
        assert_eq!(ordering, std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_compare_versions_with_preferred() {
        let scorer = QualityScorer::with_default_weights();
        
        let v4k = create_test_version(
            Resolution::new(3840, 2160),
            "H.265",
            SourceType::BluRay,
            30.0,
        );
        let mut v720 = create_test_version(
            Resolution::new(1280, 720),
            "H.264",
            SourceType::Hdtv,
            2.0,
        );
        v720.is_preferred = true;
        
        // Preferred 720p should be greater than non-preferred 4K
        let ordering = scorer.compare_versions(&v720, &v4k);
        assert_eq!(ordering, std::cmp::Ordering::Greater);
    }
}
