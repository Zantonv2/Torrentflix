// Placeholder result ranker
// TODO: Implement actual ranking logic

use crate::models::MediaSearchResult;

pub struct ResultRanker {
    scorer: super::scorer::MediaScorer,
}

impl ResultRanker {
    pub fn new() -> Self {
        Self {
            scorer: super::scorer::MediaScorer::new(),
        }
    }

    pub fn rank_results(&self, mut results: Vec<MediaSearchResult>) -> Vec<MediaSearchResult> {
        results = self.scorer.score_results(results);
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}

impl Default for ResultRanker {
    fn default() -> Self {
        Self::new()
    }
}
