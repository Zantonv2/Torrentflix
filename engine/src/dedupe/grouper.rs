// Placeholder result grouper
// TODO: Implement actual grouping logic

use crate::models::MediaSearchResult;

pub struct ResultGrouper;

impl ResultGrouper {
    pub fn new() -> Self {
        Self
    }

    pub fn group_results(&self, results: Vec<MediaSearchResult>) -> Vec<MediaSearchResult> {
        // Placeholder: return results as-is
        results
    }
}

impl Default for ResultGrouper {
    fn default() -> Self {
        Self::new()
    }
}
