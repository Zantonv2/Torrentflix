// Placeholder library models
// TODO: Implement actual library-specific models

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub id: String,
    pub title: String,
    pub media_type: String,
}

impl LibraryEntry {
    pub fn new(id: String, title: String, media_type: String) -> Self {
        Self {
            id,
            title,
            media_type,
        }
    }
}
