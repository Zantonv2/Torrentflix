use serde::{Deserialize, Serialize};

/// Scoring weights for different torrent attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreWeights {
    pub resolution_weight: f32,
    pub codec_weight: f32,
    pub source_weight: f32,
    pub audio_weight: f32,
    pub seeders_weight: f32,
    pub size_weight: f32,
    pub recency_weight: f32,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            resolution_weight: 0.3,
            codec_weight: 0.2,
            source_weight: 0.15,
            audio_weight: 0.1,
            seeders_weight: 0.15,
            size_weight: 0.05,
            recency_weight: 0.05,
        }
    }
}

/// Score calculation result
#[derive(Debug, Clone)]
pub struct ScoreResult {
    pub total_score: f32,
    pub breakdown: ScoreBreakdown,
}

/// Detailed score breakdown for debugging
#[derive(Debug, Clone)]
pub struct ScoreBreakdown {
    pub resolution_score: f32,
    pub codec_score: f32,
    pub source_score: f32,
    pub audio_score: f32,
    pub seeders_score: f32,
    pub size_score: f32,
    pub recency_score: f32,
}

impl ScoreResult {
    pub fn new(total_score: f32, breakdown: ScoreBreakdown) -> Self {
        Self {
            total_score,
            breakdown,
        }
    }
}

impl ScoreBreakdown {
    pub fn new() -> Self {
        Self {
            resolution_score: 0.0,
            codec_score: 0.0,
            source_score: 0.0,
            audio_score: 0.0,
            seeders_score: 0.0,
            size_score: 0.0,
            recency_score: 0.0,
        }
    }
}

impl Default for ScoreBreakdown {
    fn default() -> Self {
        Self::new()
    }
}
