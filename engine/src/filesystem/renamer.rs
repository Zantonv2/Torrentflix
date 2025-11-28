// Placeholder file renamer
// TODO: Implement actual file renaming logic

use anyhow::Result;
use crate::models::ParsedMedia;

pub struct FileRenamer;

impl FileRenamer {
    pub fn new() -> Self {
        Self
    }

    pub async fn rename_file(&self, current_name: &str, parsed_media: &ParsedMedia) -> Result<String> {
        // Placeholder: return cleaned title
        Ok(parsed_media.title.clone())
    }

    pub async fn generate_filename(&self, parsed_media: &ParsedMedia) -> Result<String> {
        let mut filename = parsed_media.title.clone();
        
        if let Some(year) = parsed_media.year {
            filename.push_str(&format!(" ({})", year));
        }
        
        Ok(filename)
    }
}

impl Default for FileRenamer {
    fn default() -> Self {
        Self::new()
    }
}
