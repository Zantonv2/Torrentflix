# Requirements Document: Settings UI

## Introduction

The Settings UI is a configuration interface that allows users to manage TorrentFlix application settings without editing environment files. It provides centralized management of API keys, qBittorrent connection details, library paths, and application preferences. This feature is critical for user experience and enables the app to function as a standalone desktop application.

## Glossary

- **Settings**: User-configurable application parameters persisted to database
- **API Key**: Authentication token for external services (TMDB, Kinopoisk)
- **qBittorrent**: Torrent client used for downloading; accessed via WebAPI
- **Library Path**: Filesystem directory where downloaded movies are stored
- **Download Path**: Filesystem directory where qBittorrent saves incomplete downloads
- **Settings Tab**: UI component in the main application navigation for accessing settings
- **Settings Database**: SQLite table storing user configuration with key-value pairs
- **Test Connection**: Validation operation that verifies qBittorrent credentials and connectivity

## Requirements

### Requirement 1

**User Story:** As a user, I want to configure API keys for metadata services, so that I can enable rating aggregation and metadata enrichment features.

#### Acceptance Criteria

1. WHEN the user opens the Settings tab THEN the system SHALL display input fields for TMDB API key, TMDB read access token, and Kinopoisk API token
2. WHEN the user enters an API key and clicks Save THEN the system SHALL persist the key to the settings database and display a success confirmation
3. WHEN the user leaves an API key field empty THEN the system SHALL allow saving without that key and the application SHALL gracefully degrade (ratings from that source will not be fetched)
4. WHEN the user views the Settings tab THEN the system SHALL display previously saved API keys (masked for security, showing only last 4 characters)
5. WHEN the user modifies an API key and clicks Save THEN the system SHALL update the key in the database and apply the change immediately without requiring an application restart

### Requirement 2

**User Story:** As a user, I want to configure qBittorrent connection details, so that the application can communicate with my torrent client.

#### Acceptance Criteria

1. WHEN the user opens the Settings tab THEN the system SHALL display input fields for qBittorrent URL, username, and password
2. WHEN the user enters qBittorrent credentials and clicks "Test Connection" THEN the system SHALL attempt to authenticate with the qBittorrent WebAPI and display success or failure message
3. WHEN the test connection succeeds THEN the system SHALL automatically save the credentials to the settings database
4. WHEN the test connection fails THEN the system SHALL display the error reason and NOT save the credentials
5. WHEN the user saves settings with valid qBittorrent credentials THEN the system SHALL persist them to the database and use them for all subsequent download operations

### Requirement 3

**User Story:** As a user, I want to configure filesystem paths for my library and downloads, so that the application knows where to store and find downloaded files.

#### Acceptance Criteria

1. WHEN the user opens the Settings tab THEN the system SHALL display input fields for Library Path and Download Path
2. WHEN the user clicks a folder picker button next to a path field THEN the system SHALL open a native file browser dialog
3. WHEN the user selects a folder in the file browser THEN the system SHALL populate the path field with the selected directory path
4. WHEN the user clicks Save THEN the system SHALL validate that both paths exist and are readable, and persist them to the settings database
5. IF a path does not exist or is not readable THEN the system SHALL display an error message and prevent saving until valid paths are provided

### Requirement 4

**User Story:** As a user, I want my settings to persist across application restarts, so that I don't need to reconfigure the application each time I launch it.

#### Acceptance Criteria

1. WHEN the user saves settings THEN the system SHALL write all settings to the SQLite settings database
2. WHEN the application starts THEN the system SHALL load all settings from the database and apply them
3. WHEN the user closes and reopens the application THEN the system SHALL restore all previously saved settings
4. WHEN a setting is missing from the database THEN the system SHALL use a sensible default value (e.g., localhost:8080 for qBittorrent)
5. WHEN the database is corrupted or settings cannot be loaded THEN the system SHALL display a warning and use default values to allow the application to continue functioning

### Requirement 5

**User Story:** As a user, I want visual feedback when I save settings, so that I know whether the operation succeeded or failed.

#### Acceptance Criteria

1. WHEN the user clicks Save and the operation succeeds THEN the system SHALL display a success notification (toast or modal) for 3 seconds
2. WHEN the user clicks Save and the operation fails THEN the system SHALL display an error notification with the reason for failure
3. WHEN the user clicks "Test Connection" and it succeeds THEN the system SHALL display a success message indicating the connection is valid
4. WHEN the user clicks "Test Connection" and it fails THEN the system SHALL display an error message with the specific failure reason (e.g., "Connection refused", "Invalid credentials")
5. WHEN a notification is displayed THEN the system SHALL automatically dismiss it after 5 seconds or allow the user to manually close it

### Requirement 6

**User Story:** As a developer, I want settings to be validated before saving, so that invalid configurations don't break the application.

#### Acceptance Criteria

1. WHEN the user attempts to save settings THEN the system SHALL validate that qBittorrent URL is a valid HTTP(S) URL
2. WHEN the user attempts to save settings THEN the system SHALL validate that filesystem paths are absolute paths and exist on the system
3. IF validation fails THEN the system SHALL display specific error messages for each invalid field and prevent saving
4. WHEN the user saves valid settings THEN the system SHALL persist them to the database without errors
5. WHEN settings are loaded from the database THEN the system SHALL validate them and use defaults for any corrupted or invalid values

### Requirement 7

**User Story:** As a user, I want to configure optional notification services, so that I can receive alerts about downloads and important events.

#### Acceptance Criteria

1. WHEN the user opens the Settings tab THEN the system SHALL display sections for Telegram and Email notifications with enable/disable toggles
2. WHEN the user enables Telegram notifications and enters bot token and chat ID THEN the system SHALL validate the credentials and allow saving
3. WHEN the user enables Email notifications and enters SMTP details THEN the system SHALL validate the SMTP connection and allow saving
4. WHEN the user leaves notification fields empty THEN the system SHALL allow saving with notifications disabled
5. WHEN the user saves notification settings THEN the system SHALL persist them to the database and use them for future notifications

### Requirement 8

**User Story:** As a user, I want to configure application-level settings, so that I can customize logging, search behavior, and download limits.

#### Acceptance Criteria

1. WHEN the user opens the Settings tab THEN the system SHALL display input fields for log level, search limit, search timeout, and max concurrent downloads
2. WHEN the user modifies application settings and clicks Save THEN the system SHALL persist them to the database
3. WHEN the user changes log level THEN the system SHALL apply the new log level immediately
4. WHEN the user changes max concurrent downloads THEN the system SHALL apply the limit to new downloads
5. WHEN the user saves application settings THEN the system SHALL validate that numeric values are within acceptable ranges

### Requirement 9

**User Story:** As a user, I want to configure proxy settings, so that I can route all application traffic through a SOCKS5 proxy.

#### Acceptance Criteria

1. WHEN the user opens the Settings tab THEN the system SHALL display a Proxy section with enable/disable toggle and proxy address/port fields
2. WHEN the user enables proxy and enters address and port THEN the system SHALL validate the proxy configuration
3. WHEN the user saves proxy settings THEN the system SHALL persist them to the database and apply them to all network connections
4. WHEN the user disables proxy THEN the system SHALL remove proxy configuration and use direct connections
5. WHEN proxy is enabled but unreachable THEN the system SHALL display a warning but allow the application to continue functioning
