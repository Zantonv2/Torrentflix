// Configurable weights for media quality scoring

pub struct ScoreWeights {
    pub resolution_weight: f32,
    pub source_weight: f32,
    pub codec_weight: f32,
    pub seeders_weight: f32,
    pub size_weight: f32,
}

impl ScoreWeights {
    pub fn new() -> Self {
        Self {
            resolution_weight: 0.2,
            source_weight: 0.2,
            codec_weight: 0.1,
            seeders_weight: 0.3,
            size_weight: 0.2,
        }
    }
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self::new()
    }
}
