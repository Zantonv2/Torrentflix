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

    // Subtask 23.2: Test serialization/deserialization
    #[test]
    fn test_settings_serialization_to_json() {
        let settings = Settings::default();
        let json = serde_json::to_string(&settings);
        assert!(json.is_ok(), "Settings should serialize to JSON");
        
        let json_str = json.unwrap();
        assert!(json_str.contains("qbittorrent_host"));
        assert!(json_str.contains("localhost"));
    }

    #[test]
    fn test_settings_deserialization_from_json() {
        let json = r#"{
            "tmdb_api_key": null,
            "tmdb_read_access_token": null,
            "kinopoisk_api_token": null,
            "qbittorrent_enabled": true,
            "qbittorrent_host": "192.168.1.100",
            "qbittorrent_port": 8080,
            "qbittorrent_username": "testuser",
            "qbittorrent_password": "testpass",
            "qbittorrent_use_ssl": false,
            "qbittorrent_verify_ssl": false,
            "qbittorrent_category": null,
            "qbittorrent_tags": null,
            "qbittorrent_timeout": 15,
            "library_path": "/tmp/library",
            "download_path": "/tmp/downloads",
            "telegram_enabled": false,
            "telegram_bot_token": null,
            "telegram_chat_id": null,
            "telegram_debounce": 300,
            "email_enabled": false,
            "smtp_host": "smtp.gmail.com",
            "smtp_port": 587,
            "smtp_user": null,
            "smtp_password": null,
            "from_email": null,
            "to_email": null,
            "log_level": "INFO",
            "log_format": "json",
            "debug": false,
            "search_limit": 100,
            "search_timeout": 30,
            "max_concurrent_downloads": 3,
            "enabled_indexers": "monna",
            "proxy_enabled": false,
            "proxy_address": "127.0.0.1",
            "proxy_port": 1080,
            "database_url": "sqlite:///torrent_aggregator.db",
            "db_pool_size": 10,
            "db_max_overflow": 20
        }"#;
        
        let settings: Result<Settings, _> = serde_json::from_str(json);
        assert!(settings.is_ok(), "Should deserialize valid JSON");
        
        let settings = settings.unwrap();
        assert_eq!(settings.qbittorrent_host, "192.168.1.100");
        assert_eq!(settings.qbittorrent_port, 8080);
        assert_eq!(settings.qbittorrent_username, "testuser");
        assert_eq!(settings.qbittorrent_timeout, 15);
    }

    #[test]
    fn test_settings_round_trip_serialization() {
        let mut original = Settings::default();
        original.qbittorrent_host = "test.example.com".to_string();
        original.qbittorrent_port = 9999;
        original.tmdb_api_key = Some("test_key_123".to_string());
        original.search_limit = 500;
        
        // Serialize to JSON
        let json = serde_json::to_string(&original).unwrap();
        
        // Deserialize back
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        
        // Should be equal
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_settings_serialization_with_optional_fields() {
        let mut settings = Settings::default();
        settings.tmdb_api_key = Some("key123".to_string());
        settings.kinopoisk_api_token = Some("token456".to_string());
        settings.telegram_bot_token = Some("bot789".to_string());
        
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.tmdb_api_key, Some("key123".to_string()));
        assert_eq!(deserialized.kinopoisk_api_token, Some("token456".to_string()));
        assert_eq!(deserialized.telegram_bot_token, Some("bot789".to_string()));
    }

    #[test]
    fn test_settings_serialization_with_none_optional_fields() {
        let settings = Settings::default();
        
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.tmdb_api_key, None);
        assert_eq!(deserialized.tmdb_read_access_token, None);
        assert_eq!(deserialized.kinopoisk_api_token, None);
    }

    // Subtask 23.3: Test default value generation
    #[test]
    fn test_default_qbittorrent_settings() {
        let settings = Settings::default();
        assert_eq!(settings.qbittorrent_enabled, true);
        assert_eq!(settings.qbittorrent_host, "localhost");
        assert_eq!(settings.qbittorrent_port, 5555);
        assert_eq!(settings.qbittorrent_username, "admin");
        assert_eq!(settings.qbittorrent_password, "");
        assert_eq!(settings.qbittorrent_use_ssl, false);
        assert_eq!(settings.qbittorrent_verify_ssl, false);
        assert_eq!(settings.qbittorrent_timeout, 10);
    }

    #[test]
    fn test_default_notification_settings() {
        let settings = Settings::default();
        assert_eq!(settings.telegram_enabled, false);
        assert_eq!(settings.telegram_bot_token, None);
        assert_eq!(settings.telegram_chat_id, None);
        assert_eq!(settings.telegram_debounce, 300);
        assert_eq!(settings.email_enabled, false);
        assert_eq!(settings.smtp_host, "smtp.gmail.com");
        assert_eq!(settings.smtp_port, 587);
    }

    #[test]
    fn test_default_application_settings() {
        let settings = Settings::default();
        assert_eq!(settings.log_level, "INFO");
        assert_eq!(settings.log_format, "json");
        assert_eq!(settings.debug, false);
        assert_eq!(settings.search_limit, 100);
        assert_eq!(settings.search_timeout, 30);
        assert_eq!(settings.max_concurrent_downloads, 3);
        assert_eq!(settings.enabled_indexers, "monna");
    }

    #[test]
    fn test_default_proxy_settings() {
        let settings = Settings::default();
        assert_eq!(settings.proxy_enabled, false);
        assert_eq!(settings.proxy_address, "127.0.0.1");
        assert_eq!(settings.proxy_port, 1080);
    }

    #[test]
    fn test_default_database_settings() {
        let settings = Settings::default();
        assert_eq!(settings.database_url, "sqlite:///torrent_aggregator.db");
        assert_eq!(settings.db_pool_size, 10);
        assert_eq!(settings.db_max_overflow, 20);
    }

    #[test]
    fn test_new_creates_default() {
        let settings1 = Settings::new();
        let settings2 = Settings::default();
        assert_eq!(settings1, settings2);
    }

    // Subtask 23.4: Test edge cases (empty strings, boundary values)
    #[test]
    fn test_settings_with_empty_strings() {
        let mut settings = Settings::default();
        settings.qbittorrent_host = "".to_string();
        settings.qbittorrent_username = "".to_string();
        settings.qbittorrent_password = "".to_string();
        
        // Should serialize/deserialize without errors
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.qbittorrent_host, "");
        assert_eq!(deserialized.qbittorrent_username, "");
        assert_eq!(deserialized.qbittorrent_password, "");
    }

    #[test]
    fn test_settings_with_boundary_port_values() {
        let mut settings = Settings::default();
        
        // Minimum valid port
        settings.qbittorrent_port = 1;
        assert_eq!(settings.qbittorrent_port, 1);
        
        // Maximum valid port
        settings.qbittorrent_port = 65535;
        assert_eq!(settings.qbittorrent_port, 65535);
        
        // Common ports
        settings.qbittorrent_port = 80;
        assert_eq!(settings.qbittorrent_port, 80);
        
        settings.qbittorrent_port = 443;
        assert_eq!(settings.qbittorrent_port, 443);
        
        settings.qbittorrent_port = 8080;
        assert_eq!(settings.qbittorrent_port, 8080);
    }

    #[test]
    fn test_settings_with_boundary_timeout_values() {
        let mut settings = Settings::default();
        
        // Minimum timeout
        settings.qbittorrent_timeout = 1;
        assert_eq!(settings.qbittorrent_timeout, 1);
        
        // Maximum reasonable timeout
        settings.qbittorrent_timeout = 300;
        assert_eq!(settings.qbittorrent_timeout, 300);
        
        // Very large timeout (edge case)
        settings.qbittorrent_timeout = u64::MAX;
        assert_eq!(settings.qbittorrent_timeout, u64::MAX);
    }

    #[test]
    fn test_settings_with_boundary_search_limit() {
        let mut settings = Settings::default();
        
        // Minimum search limit
        settings.search_limit = 1;
        assert_eq!(settings.search_limit, 1);
        
        // Maximum reasonable search limit
        settings.search_limit = 10000;
        assert_eq!(settings.search_limit, 10000);
        
        // Very large search limit (edge case)
        settings.search_limit = u32::MAX;
        assert_eq!(settings.search_limit, u32::MAX);
    }

    #[test]
    fn test_settings_with_boundary_concurrent_downloads() {
        let mut settings = Settings::default();
        
        // Minimum concurrent downloads
        settings.max_concurrent_downloads = 1;
        assert_eq!(settings.max_concurrent_downloads, 1);
        
        // Maximum reasonable concurrent downloads
        settings.max_concurrent_downloads = 100;
        assert_eq!(settings.max_concurrent_downloads, 100);
        
        // Very large concurrent downloads (edge case)
        settings.max_concurrent_downloads = u32::MAX;
        assert_eq!(settings.max_concurrent_downloads, u32::MAX);
    }

    #[test]
    fn test_settings_with_very_long_strings() {
        let mut settings = Settings::default();
        
        // Very long hostname
        let long_host = "a".repeat(1000);
        settings.qbittorrent_host = long_host.clone();
        assert_eq!(settings.qbittorrent_host, long_host);
        
        // Very long API key
        let long_key = "k".repeat(5000);
        settings.tmdb_api_key = Some(long_key.clone());
        assert_eq!(settings.tmdb_api_key, Some(long_key));
        
        // Should serialize/deserialize
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.qbittorrent_host.len(), 1000);
        assert_eq!(deserialized.tmdb_api_key.as_ref().unwrap().len(), 5000);
    }

    #[test]
    fn test_settings_with_special_characters() {
        let mut settings = Settings::default();
        
        // Special characters in strings
        settings.qbittorrent_host = "host-with_special.chars".to_string();
        settings.qbittorrent_username = "user@domain".to_string();
        settings.qbittorrent_password = "p@$$w0rd!#%".to_string();
        
        // Should serialize/deserialize correctly
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.qbittorrent_host, "host-with_special.chars");
        assert_eq!(deserialized.qbittorrent_username, "user@domain");
        assert_eq!(deserialized.qbittorrent_password, "p@$$w0rd!#%");
    }

    #[test]
    fn test_settings_with_unicode_characters() {
        let mut settings = Settings::default();
        
        // Unicode characters
        settings.qbittorrent_username = "用户名".to_string();
        settings.qbittorrent_password = "密码123".to_string();
        
        // Should serialize/deserialize correctly
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.qbittorrent_username, "用户名");
        assert_eq!(deserialized.qbittorrent_password, "密码123");
    }

    #[test]
    fn test_qbittorrent_url_with_different_ports() {
        let mut settings = Settings::default();
        
        // Test with port 80
        settings.qbittorrent_port = 80;
        settings.qbittorrent_use_ssl = false;
        assert_eq!(settings.qbittorrent_url(), "http://localhost:80");
        
        // Test with port 443
        settings.qbittorrent_port = 443;
        settings.qbittorrent_use_ssl = true;
        assert_eq!(settings.qbittorrent_url(), "https://localhost:443");
        
        // Test with custom port
        settings.qbittorrent_port = 12345;
        settings.qbittorrent_use_ssl = false;
        assert_eq!(settings.qbittorrent_url(), "http://localhost:12345");
    }

    #[test]
    fn test_qbittorrent_url_with_different_hosts() {
        let mut settings = Settings::default();
        settings.qbittorrent_use_ssl = false;
        settings.qbittorrent_port = 8080;
        
        // Test with localhost
        settings.qbittorrent_host = "localhost".to_string();
        assert_eq!(settings.qbittorrent_url(), "http://localhost:8080");
        
        // Test with IP address
        settings.qbittorrent_host = "192.168.1.100".to_string();
        assert_eq!(settings.qbittorrent_url(), "http://192.168.1.100:8080");
        
        // Test with domain name
        settings.qbittorrent_host = "qbittorrent.example.com".to_string();
        assert_eq!(settings.qbittorrent_url(), "http://qbittorrent.example.com:8080");
    }

    #[test]
    fn test_settings_clone() {
        let settings1 = Settings::default();
        let settings2 = settings1.clone();
        
        assert_eq!(settings1, settings2);
        
        // Verify they are independent
        let mut settings3 = settings1.clone();
        settings3.qbittorrent_host = "modified".to_string();
        
        assert_ne!(settings1.qbittorrent_host, settings3.qbittorrent_host);
    }

    #[test]
    fn test_settings_debug_format() {
        let settings = Settings::default();
        let debug_str = format!("{:?}", settings);
        
        // Should contain key fields
        assert!(debug_str.contains("Settings"));
        assert!(debug_str.contains("qbittorrent_host"));
        assert!(debug_str.contains("localhost"));
    }
}
