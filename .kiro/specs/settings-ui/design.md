# Design Document: Settings UI

## Overview

The Settings UI is a configuration interface that enables users to manage TorrentFlix application settings through a dedicated tab in the main application. The system provides a user-friendly form-based interface for managing API keys, qBittorrent connection details, and filesystem paths. All settings are persisted to SQLite and loaded on application startup, ensuring configuration survives application restarts.

The design follows a layered architecture:
- **Frontend**: Svelte component (`SettingsTab.svelte`) with form validation and user feedback
- **Backend**: Rust engine with settings management, validation, and persistence
- **Database**: SQLite table storing key-value settings with type information
- **Tauri Bridge**: Commands for loading, saving, and validating settings

## Architecture

### High-Level Data Flow

```
User Input (SettingsTab.svelte)
    ↓
Form Validation (client-side)
    ↓
Tauri Command (save_settings / test_connection)
    ↓
Engine Settings Manager (validation, persistence)
    ↓
SQLite Database (settings table)
    ↓
Response back to UI (success/error notification)
```

### Component Interaction

1. **SettingsTab.svelte** - Renders form, handles user input, displays notifications
2. **Engine Settings Manager** - Validates settings, persists to database, loads on startup
3. **qBittorrent Client** - Tests connection with provided credentials
4. **SQLite Database** - Persists settings with defaults for missing values

## Components and Interfaces

### Frontend Component: SettingsTab.svelte

**Responsibilities:**
- Render settings form with input fields for all configuration options
- Display current settings loaded from backend
- Handle form submission and validation feedback
- Show success/error notifications
- Provide file picker for path selection
- Implement "Test Connection" button for qBittorrent validation

**Props:** None (loads settings on mount)

**State:**
- `settings`: Current form values (API keys, paths, qBittorrent config)
- `loading`: Boolean indicating if settings are being loaded/saved
- `notification`: Current notification (message, type, duration)
- `errors`: Validation errors per field

**Events:**
- `save_settings` - Tauri command to persist settings
- `test_qbittorrent` - Tauri command to validate qBittorrent connection
- `get_settings` - Tauri command to load current settings

### Backend: Settings Manager

**Module:** `engine/src/settings/manager.rs`

**Responsibilities:**
- Load settings from database on startup
- Validate settings before persistence
- Persist settings to database
- Provide default values for missing settings
- Handle settings migrations

**Key Functions:**
- `load_settings() -> Result<Settings>` - Load from database with defaults
- `save_settings(settings: Settings) -> Result<()>` - Validate and persist
- `validate_settings(settings: &Settings) -> Result<()>` - Validate all fields
- `test_qbittorrent_connection(url: &str, username: &str, password: &str) -> Result<()>` - Test connection

**Settings Structure:**
```rust
pub struct Settings {
    pub tmdb_api_key: Option<String>,
    pub kinopoisk_api_key: Option<String>,
    pub qbittorrent_url: String,
    pub qbittorrent_username: String,
    pub qbittorrent_password: String,
    pub library_path: String,
    pub download_path: String,
}
```

### Tauri Commands

**Commands to implement:**
- `get_settings() -> Settings` - Load current settings
- `save_settings(settings: Settings) -> Result<()>` - Save and validate
- `test_qbittorrent(url: String, username: String, password: String) -> Result<()>` - Test connection
- `pick_folder() -> Result<String>` - Open native file picker

## Data Models

### Settings Table Schema

```sql
CREATE TABLE IF NOT EXISTS settings (
    id INTEGER PRIMARY KEY,
    key TEXT UNIQUE NOT NULL,
    value TEXT NOT NULL,
    type TEXT NOT NULL,  -- 'string', 'bool', 'int'
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### Complete Settings Keys

#### API & Metadata Services

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `tmdb_api_key` | string | (empty) | TMDB API authentication token |
| `tmdb_read_access_token` | string | (empty) | TMDB read access token for advanced features |
| `kinopoisk_api_token` | string | (empty) | Kinopoisk API authentication token |

#### qBittorrent Configuration

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `qbittorrent_enabled` | bool | `true` | Enable/disable qBittorrent integration |
| `qbittorrent_host` | string | `localhost` | qBittorrent host/IP address |
| `qbittorrent_port` | int | `5555` | qBittorrent WebAPI port |
| `qbittorrent_username` | string | `admin` | qBittorrent username |
| `qbittorrent_password` | string | (empty) | qBittorrent password |
| `qbittorrent_use_ssl` | bool | `false` | Use HTTPS for qBittorrent connection |
| `qbittorrent_verify_ssl` | bool | `false` | Verify SSL certificates |
| `qbittorrent_category` | string | (empty) | Default category for downloads |
| `qbittorrent_tags` | string | (empty) | Default tags for downloads (comma-separated) |
| `qbittorrent_timeout` | int | `10` | Connection timeout in seconds |

#### Filesystem Paths

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `library_path` | string | `~/Downloads/TorrentFlix` | Directory for downloaded files |
| `download_path` | string | `~/Downloads/qBittorrent` | qBittorrent download directory |

#### Notifications

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `telegram_enabled` | bool | `false` | Enable Telegram notifications |
| `telegram_bot_token` | string | (empty) | Telegram bot token |
| `telegram_chat_id` | string | (empty) | Telegram chat ID |
| `telegram_debounce` | int | `300` | Debounce notifications (seconds) |
| `email_enabled` | bool | `false` | Enable email notifications |
| `smtp_host` | string | `smtp.gmail.com` | SMTP server host |
| `smtp_port` | int | `587` | SMTP server port |
| `smtp_user` | string | (empty) | SMTP username |
| `smtp_password` | string | (empty) | SMTP password |
| `from_email` | string | (empty) | Sender email address |
| `to_email` | string | (empty) | Recipient email address |

#### Application Settings

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `log_level` | string | `INFO` | Logging level (DEBUG, INFO, WARN, ERROR) |
| `log_format` | string | `json` | Log format (json, text) |
| `debug` | bool | `false` | Enable debug mode |
| `search_limit` | int | `100` | Maximum search results |
| `search_timeout` | int | `30` | Search timeout in seconds |
| `max_concurrent_downloads` | int | `3` | Maximum concurrent downloads |
| `enabled_indexers` | string | `monna` | Enabled indexers (comma-separated) |

#### Proxy Configuration

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `proxy_enabled` | bool | `false` | Enable SOCKS5 proxy |
| `proxy_address` | string | `127.0.0.1` | Proxy server address |
| `proxy_port` | int | `1080` | Proxy server port |

#### Database Configuration

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `database_url` | string | `sqlite:///torrent_aggregator.db` | Database connection URL |
| `db_pool_size` | int | `10` | Database connection pool size |
| `db_max_overflow` | int | `20` | Database connection pool overflow |

### Settings Struct (Rust)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // API & Metadata
    pub tmdb_api_key: Option<String>,
    pub tmdb_read_access_token: Option<String>,
    pub kinopoisk_api_token: Option<String>,
    
    // qBittorrent
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
    
    // Filesystem
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
    
    // Application
    pub log_level: String,
    pub log_format: String,
    pub debug: bool,
    pub search_limit: u32,
    pub search_timeout: u64,
    pub max_concurrent_downloads: u32,
    pub enabled_indexers: String,
    
    // Proxy
    pub proxy_enabled: bool,
    pub proxy_address: String,
    pub proxy_port: u16,
    
    // Database
    pub database_url: String,
    pub db_pool_size: u32,
    pub db_max_overflow: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            tmdb_api_key: None,
            tmdb_read_access_token: None,
            kinopoisk_api_token: None,
            
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
            
            library_path: dirs::download_dir()
                .map(|p| p.join("TorrentFlix").to_string_lossy().to_string())
                .unwrap_or_else(|| "~/Downloads/TorrentFlix".to_string()),
            download_path: dirs::download_dir()
                .map(|p| p.join("qBittorrent").to_string_lossy().to_string())
                .unwrap_or_else(|| "~/Downloads/qBittorrent".to_string()),
            
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
            
            log_level: "INFO".to_string(),
            log_format: "json".to_string(),
            debug: false,
            search_limit: 100,
            search_timeout: 30,
            max_concurrent_downloads: 3,
            enabled_indexers: "monna".to_string(),
            
            proxy_enabled: false,
            proxy_address: "127.0.0.1".to_string(),
            proxy_port: 1080,
            
            database_url: "sqlite:///torrent_aggregator.db".to_string(),
            db_pool_size: 10,
            db_max_overflow: 20,
        }
    }
}
```

## Correctness Properties

A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.

### Property 1: Settings Persistence Round Trip

*For any* valid settings object, saving it to the database and then loading it should produce an equivalent settings object.

**Validates: Requirements 4.1, 4.2, 4.3**

**Rationale:** This ensures that settings survive the save-load cycle without data loss or corruption. It's a fundamental round-trip property for any persistence layer.

### Property 2: Invalid Settings Rejection

*For any* settings object with invalid values (malformed URL, non-existent path, empty required field), the system SHALL reject the save operation and return a specific error for each invalid field.

**Validates: Requirements 6.1, 6.2, 6.3**

**Rationale:** Validation must be consistent and comprehensive. Invalid settings should never be persisted, preventing runtime errors later.

### Property 3: Default Values on Missing Settings

*For any* missing or corrupted setting in the database, the system SHALL load a sensible default value that allows the application to function.

**Validates: Requirements 4.4, 4.5**

**Rationale:** Graceful degradation is critical. Missing settings should not crash the app; defaults should be used instead.

### Property 4: qBittorrent Connection Validation

*For any* qBittorrent URL, username, and password combination, the test connection operation SHALL either succeed (returning success) or fail with a specific error message (connection refused, invalid credentials, timeout, etc.).

**Validates: Requirements 2.2, 2.3, 2.4**

**Rationale:** Connection testing must be reliable and provide actionable feedback. The operation should never hang or return ambiguous results.

### Property 5: API Key Masking

*For any* API key stored in settings, when displayed in the UI, the system SHALL show only the last 4 characters and mask the rest with asterisks.

**Validates: Requirements 1.4**

**Rationale:** Security property ensuring sensitive credentials are not exposed in the UI while still allowing users to verify they have the right key.

### Property 6: Settings Isolation

*For any* two independent settings save operations with different values, each operation SHALL persist only its own values without affecting other settings.

**Validates: Requirements 1.5, 2.5**

**Rationale:** Settings updates must be atomic and isolated. Changing one setting should not affect others.

## Error Handling

### Validation Errors

- **Invalid URL**: "qBittorrent URL must be a valid HTTP(S) URL (e.g., http://localhost:8080)"
- **Invalid Path**: "Path must be an absolute path that exists on the system"
- **Connection Failed**: "Failed to connect to qBittorrent: {reason}" (connection refused, timeout, invalid credentials)
- **Database Error**: "Failed to save settings: {reason}"

### User-Facing Notifications

- **Success**: "Settings saved successfully" (green, 3 seconds)
- **Error**: "{error message}" (red, 5 seconds or manual close)
- **Connection Test Success**: "Connection to qBittorrent successful" (green, 3 seconds)
- **Connection Test Failed**: "Connection failed: {reason}" (red, 5 seconds)

### Graceful Degradation

- Missing API keys: Application continues with reduced functionality (no ratings from that source)
- Invalid paths: Use default paths and log warning
- qBittorrent unreachable: Show warning but allow app to continue (downloads will fail when attempted)

## Testing Strategy

### Unit Tests

- Settings validation (valid/invalid URLs, paths, API keys)
- Settings serialization/deserialization
- Default values generation
- Settings struct equality

### Property-Based Tests

- **Property 1**: Settings round trip (save → load → compare)
- **Property 2**: Invalid settings rejection (generate invalid settings, verify rejection)
- **Property 3**: Default values on missing settings (corrupt database, verify defaults)
- **Property 4**: qBittorrent connection validation (mock qBittorrent responses)
- **Property 5**: API key masking (verify masking logic)
- **Property 6**: Settings isolation (concurrent saves, verify no interference)

### Integration Tests

- End-to-end settings save/load flow
- qBittorrent connection test with real client
- File picker integration
- Notification display

### Testing Framework

- **Rust**: `proptest` for property-based testing (100+ iterations per property)
- **Svelte**: Component testing with `vitest` and `@testing-library/svelte`

## Implementation Notes

### Database Initialization

Settings table should be created on first application startup. If table exists, skip creation.

### Settings Loading

On application startup, the Engine should:
1. Load settings from database
2. Validate loaded settings
3. Use defaults for any missing/invalid values
4. Make settings available to all components

### Settings Updates

When user saves settings:
1. Validate all fields
2. If validation fails, return errors to UI
3. If validation succeeds, persist to database
4. Update in-memory settings cache
5. Notify all components of changes (if needed)

### qBittorrent Connection Testing

The test connection operation should:
1. Create a temporary qBittorrent client with provided credentials
2. Attempt to authenticate
3. Return success or specific error message
4. NOT persist credentials if test fails
5. Automatically persist if test succeeds

### File Picker Integration

Use Tauri's native file picker dialog:
- `tauri::api::dialog::FileDialogBuilder`
- Allow directory selection only
- Return absolute path to selected directory
