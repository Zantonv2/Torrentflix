// Placeholder deduplicator
// TODO: Implement actual deduplication logic

use crate::models::MediaSearchResult;

pub struct Deduplicator {
    grouper: super::grouper::ResultGrouper,
    rules: super::rules::DedupeRules,
}

impl Deduplicator {
    pub fn new() -> Self {
        Self {
            grouper: super::grouper::ResultGrouper::new(),
            rules: super::rules::DedupeRules::new(),
        }
    }

    pub fn deduplicate(&self, results: Vec<MediaSearchResult>) -> Vec<MediaSearchResult> {
        self.grouper.group_results(results)
    }
}

impl Default for Deduplicator {
    fn default() -> Self {
        Self::new()
    }
}
