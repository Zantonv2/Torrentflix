use serde::{Deserialize, Serialize};

/// Complete application settings structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    // API & Metadata Services
    pub tmdb_api_key: Option<String>,
    pub tmdb_read_access_token: Option<String>,
    pub kinopoisk_api_token: Option<String>,

    // qBittorrent Configuration
    pub qbittorrent_enabled: bool,
    pub qbittorrent_host: String,
    pub qbittorrent_port: u16,
    pub qbittorrent_username: String,
    pub qbittorrent_password: String,
    pub qbittorrent_use_ssl: bool,
    pub qbittorrent_verify_ssl: bool,
    pub qbittorrent_category: Option<String>,
    pub qbittorrent_tags: Option<String>,
    pub qbittorrent_timeout: u64,

    // Filesystem Paths
    pub library_path: String,
    pub download_path: String,

    // Notifications
    pub telegram_enabled: bool,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
    pub telegram_debounce: u64,
    pub email_enabled: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: Option<String>,
    pub smtp_password: Option<String>,
    pub from_email: Option<String>,
    pub to_email: Option<String>,

    // Application Settings
    pub log_level: String,
    pub log_format: String,
    pub debug: bool,
    pub search_limit: u32,
    pub search_timeout: u64,
    pub max_concurrent_downloads: u32,
    pub enabled_indexers: String,

    // Proxy Configuration
    pub proxy_enabled: bool,
    pub proxy_address: String,
    pub proxy_port: u16,

    // Database Configuration
    pub database_url: String,
    pub db_pool_size: u32,
    pub db_max_overflow: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            // API & Metadata
            tmdb_api_key: None,
            tmdb_read_access_token: None,
            kinopoisk_api_token: None,

            // qBittorrent
            qbittorrent_enabled: true,
            qbittorrent_host: "localhost".to_string(),
            qbittorrent_port: 5555,
            qbittorrent_username: "admin".to_string(),
            qbittorrent_password: String::new(),
            qbittorrent_use_ssl: false,
            qbittorrent_verify_ssl: false,
            qbittorrent_category: None,
            qbittorrent_tags: None,
            qbittorrent_timeout: 10,

            // Filesystem
            library_path: dirs::download_dir()
                .map(|p| p.join("TorrentFlix").to_string_lossy().to_string())
                .unwrap_or_else(|| "~/Downloads/TorrentFlix".to_string()),
            download_path: dirs::download_dir()
                .map(|p| p.join("qBittorrent").to_string_lossy().to_string())
                .unwrap_or_else(|| "~/Downloads/qBittorrent".to_string()),

            // Notifications
            telegram_enabled: false,
            telegram_bot_token: None,
            telegram_chat_id: None,
            telegram_debounce: 300,
            email_enabled: false,
            smtp_host: "smtp.gmail.com".to_string(),
            smtp_port: 587,
            smtp_user: None,
            smtp_password: None,
            from_email: None,
            to_email: None,

            // Application
            log_level: "INFO".to_string(),
            log_format: "json".to_string(),
            debug: false,
            search_limit: 100,
            search_timeout: 30,
            max_concurrent_downloads: 3,
            enabled_indexers: "monna".to_string(),

            // Proxy
            proxy_enabled: false,
            proxy_address: "127.0.0.1".to_string(),
            proxy_port: 1080,

            // Database
            database_url: "sqlite:///torrent_aggregator.db".to_string(),
            db_pool_size: 10,
            db_max_overflow: 20,
        }
    }
}

impl Settings {
    /// Create a new Settings instance with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the qBittorrent URL
    pub fn qbittorrent_url(&self) -> String {
        let protocol = if self.qbittorrent_use_ssl { "https" } else { "http" };
        format!(
            "{}://{}:{}",
            protocol, self.qbittorrent_host, self.qbittorrent_port
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.qbittorrent_enabled, true);
        assert_eq!(settings.qbittorrent_host, "localhost");
        assert_eq!(settings.qbittorrent_port, 5555);
        assert_eq!(settings.log_level, "INFO");
        assert_eq!(settings.max_concurrent_downloads, 3);
    }

    #[test]
    fn test_qbittorrent_url_http() {
        let mut settings = Settings::default();
        settings.qbittorrent_use_ssl = false;
        settings.qbittorrent_host = "192.168.1.100".to_string();
        settings.qbittorrent_port = 8080;
        assert_eq!(
            settings.qbittorrent_url(),
            "http://192.168.1.100:8080"
        );
    }

    #[test]
    fn test_qbittorrent_url_https() {
        let mut settings = Settings::default();
        settings.qbittorrent_use_ssl = true;
        settings.qbittorrent_host = "example.com".to_string();
        settings.qbittorrent_port = 443;
        assert_eq!(
            settings.qbittorrent_url(),
            "https://example.com:443"
        );
    }

    #[test]
    fn test_settings_equality() {
        let settings1 = Settings::default();
        let settings2 = Settings::default();
        assert_eq!(settings1, settings2);
    }
}
