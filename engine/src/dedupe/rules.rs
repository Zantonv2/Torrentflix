// Placeholder deduplication rules
// TODO: Implement actual deduplication logic

pub struct DedupeRules {
    enabled: bool,
}

impl DedupeRules {
    pub fn new() -> Self {
        Self {
            enabled: true,
        }
    }

    pub fn is_duplicate(&self, _title1: &str, _title2: &str) -> bool {
        false
    }
}

impl Default for DedupeRules {
    fn default() -> Self {
        Self::new()
    }
}
