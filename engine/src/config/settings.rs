// Placeholder app settings
// TODO: Implement actual application settings

pub struct AppSettings {
    pub download_path: String,
    pub max_concurrent_downloads: u32,
    pub enabled_indexers: Vec<String>,
}

impl AppSettings {
    pub fn new() -> Self {
        Self {
            download_path: "./downloads".to_string(),
            max_concurrent_downloads: 3,
            enabled_indexers: vec!["monna".to_string()],
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self::new()
    }
}
