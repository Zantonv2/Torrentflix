use anyhow::Result;
use chrono::{DateTime, Utc};

use super::models::{ScoreWeights, ScoreResult, ScoreBreakdown};
use crate::models::{TorrentResult, ParsedMedia, MediaSearchResult};

/// Media scorer for ranking torrent results
pub struct MediaScorer {
    weights: ScoreWeights,
    enabled: bool,
}

impl MediaScorer {
    pub fn new(weights: ScoreWeights) -> Self {
        Self {
            weights,
            enabled: true,
        }
    }

    pub fn with_default_weights() -> Self {
        Self::new(ScoreWeights::default())
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Score a single torrent result
    pub fn score_torrent(&self, torrent: &TorrentResult, parsed_media: &ParsedMedia) -> Result<ScoreResult> {
        if !self.enabled {
            return Ok(ScoreResult::new(0.0, ScoreBreakdown::new()));
        }

        let mut breakdown = ScoreBreakdown::new();

        // Resolution scoring from ParsedMedia (higher is better)
        breakdown.resolution_score = self.score_resolution(&parsed_media.resolution);

        // Codec scoring from ParsedMedia (H.265 > H.264 > others)
        breakdown.codec_score = self.score_codec(&parsed_media.codec);

        // Source scoring from ParsedMedia (BluRay > WEB-DL > HDTV > others)
        breakdown.source_score = self.score_source(&Some(parsed_media.source.clone().unwrap_or_default()));

        // Audio scoring from ParsedMedia (DTS > AC3 > AAC > others)
        breakdown.audio_score = self.score_audio(&parsed_media.audio);

        // Seeders scoring from TorrentResult (logarithmic scale)
        breakdown.seeders_score = self.score_seeders(Some(torrent.seeders));

        // Size scoring from TorrentResult (optimal range based on resolution)
        breakdown.size_score = self.score_size(Some(torrent.size_bytes), &parsed_media.resolution);

        // Recency scoring - not available in TorrentResult, use default
        breakdown.recency_score = 0.5; // Default for unknown recency

        // Calculate weighted total
        let total_score = 
            breakdown.resolution_score * self.weights.resolution_weight +
            breakdown.codec_score * self.weights.codec_weight +
            breakdown.source_score * self.weights.source_weight +
            breakdown.audio_score * self.weights.audio_weight +
            breakdown.seeders_score * self.weights.seeders_weight +
            breakdown.size_score * self.weights.size_weight +
            breakdown.recency_score * self.weights.recency_weight;

        Ok(ScoreResult::new(total_score, breakdown))
    }

    /// Score multiple torrents and return sorted by score
    pub fn score_and_sort<'a>(&self, torrents: &'a [TorrentResult], parsed_media: &ParsedMedia) -> Result<Vec<(f32, &'a TorrentResult)>> {
        let mut scored: Vec<(f32, &TorrentResult)> = Vec::new();

        for torrent in torrents {
            let score_result = self.score_torrent(torrent, parsed_media)?;
            scored.push((score_result.total_score, torrent));
        }

        // Sort by score (descending)
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored)
    }

    /// Score MediaSearchResult (for backward compatibility)
    pub fn score_result(&self, result: &mut MediaSearchResult) {
        if let Some(best_quality) = &result.best_quality {
            if let Ok(score_result) = self.score_torrent(best_quality, &result.enriched.parsed) {
                result.score = score_result.total_score;
            }
        }
    }

    /// Score and sort MediaSearchResults
    pub fn score_results(&self, mut results: Vec<MediaSearchResult>) -> Vec<MediaSearchResult> {
        for result in &mut results {
            self.score_result(result);
        }
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    fn score_resolution(&self, resolution: &Option<String>) -> f32 {
        match resolution.as_deref() {
            Some("4K") | Some("2160p") => 1.0,
            Some("1080p") => 0.8,
            Some("720p") => 0.6,
            Some("480p") => 0.4,
            Some("360p") => 0.2,
            _ => 0.3, // Unknown resolution
        }
    }

    fn score_codec(&self, codec: &Option<String>) -> f32 {
        match codec.as_deref() {
            Some("H.265") | Some("HEVC") | Some("x265") => 1.0,
            Some("H.264") | Some("AVC") | Some("x264") => 0.8,
            Some("XviD") | Some("DivX") => 0.4,
            _ => 0.5, // Unknown codec
        }
    }

    fn score_source(&self, source: &Option<String>) -> f32 {
        match source.as_deref() {
            Some("BluRay") | Some("BDRip") | Some("BRRip") => 1.0,
            Some("WEB-DL") | Some("WEBRip") => 0.9,
            Some("HDTV") => 0.7,
            Some("DVDRip") => 0.6,
            Some("CAM") | Some("TS") | Some("TC") => 0.2,
            _ => 0.5, // Unknown source
        }
    }

    fn score_audio(&self, audio: &Option<String>) -> f32 {
        match audio.as_deref() {
            Some("DTS") | Some("DTS-HD") => 1.0,
            Some("AC3") | Some("Dolby Digital") => 0.8,
            Some("AAC") => 0.7,
            Some("MP3") => 0.5,
            _ => 0.4, // Unknown audio
        }
    }

    fn score_seeders(&self, seeders: Option<u32>) -> f32 {
        match seeders {
            Some(0) => 0.0,
            Some(1..=10) => 0.3,
            Some(11..=50) => 0.5,
            Some(51..=200) => 0.7,
            Some(201..=1000) => 0.9,
            Some(1001..) => 1.0,
            _none => 0.2,
        }
    }

    fn score_size(&self, size_bytes: Option<u64>, resolution: &Option<String>) -> f32 {
        let size = size_bytes.unwrap_or(0);
        if size == 0 {
            return 0.2;
        }

        let size_gb = size as f64 / (1024.0 * 1024.0 * 1024.0);

        // Optimal size ranges based on resolution
        let optimal_range = match resolution.as_deref() {
            Some("4K") | Some("2160p") => (15.0, 50.0), // 15-50 GB
            Some("1080p") => (2.0, 8.0), // 2-8 GB
            Some("720p") => (1.0, 4.0), // 1-4 GB
            Some("480p") => (0.5, 2.0), // 0.5-2 GB
            _ => (1.0, 10.0), // Default range
        };

        if size_gb >= optimal_range.0 && size_gb <= optimal_range.1 {
            1.0 // Perfect size
        } else if size_gb < optimal_range.0 {
            0.5 // Too small
        } else {
            0.3 // Too large
        }
    }

    fn score_recency(&self, created_at: &Option<DateTime<Utc>>) -> f32 {
        match created_at {
            Some(date) => {
                let age = Utc::now() - *date;
                let days_old = age.num_days();
                
                match days_old {
                    0..=7 => 1.0, // Very recent
                    8..=30 => 0.8, // Recent
                    31..=90 => 0.6, // Moderate
                    91..=365 => 0.4, // Old
                    _ => 0.2, // Very old
                }
            }
            _none => 0.5, // Unknown date
        }
    }
}

impl Default for MediaScorer {
    fn default() -> Self {
        Self::with_default_weights()
    }
}
