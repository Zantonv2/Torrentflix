# Implementation Plan: Settings UI

## Overview

This implementation plan converts the Settings UI design into a series of actionable coding tasks. Each task builds incrementally on previous tasks, with no orphaned code. The plan focuses on implementing the complete settings system with database persistence, validation, and a user-friendly Svelte interface.

---

## Task List

- [x] 1. Set up settings database schema and migrations
  - [x] 1.1 Create SQLite settings table with key-value schema
  - [x] 1.2 Implement database initialization on first startup
  - [x] 1.3 Add migration system for schema updates
  - _Requirements: 4.1, 4.2_

- [x] 2. Implement Settings struct and validation logic
  - [x] 2.1 Create complete Settings struct with all configuration fields
  - [x] 2.2 Implement URL validation function
  - [x] 2.3 Implement path validation function
  - [x] 2.4 Implement numeric range validation function
  - [x] 2.5 Add default values for all settings
  - _Requirements: 6.1, 6.2, 6.3_

- [x] 3. Implement Settings Manager (backend)
  - [x] 3.1 Create `engine/src/settings/manager.rs` module
  - [x] 3.2 Implement `load_settings()` to read from database with defaults
  - [x] 3.3 Implement `save_settings()` to validate and persist
  - [x] 3.4 Implement `validate_settings()` for comprehensive validation
  - [x] 3.5 Add error handling for database operations
  - _Requirements: 4.1, 4.2, 4.3, 6.4_

- [x] 4. Implement qBittorrent connection testing
  - [x] 4.1 Create `test_qbittorrent_connection()` function
  - [x] 4.2 Implement authentication with provided credentials
  - [x] 4.3 Return specific error messages for different failure types
  - [x] 4.4 Handle connection timeouts gracefully
  - _Requirements: 2.2, 2.3, 2.4_

- [x] 5. Implement Tauri commands for settings
  - [x] 5.1 Create `get_settings()` command to load current settings
  - [x] 5.2 Create `save_settings()` command to validate and persist
  - [x] 5.3 Create `test_qbittorrent()` command for connection testing
  - [x] 5.4 Create `pick_folder()` command for native file picker
  - [x] 5.5 Add error handling and response serialization
  - _Requirements: 1.2, 2.5, 3.4_

- [x] 6. Create SettingsTab.svelte component structure
  - [x] 6.1 Build form layout with sections for each setting category
  - [x] 6.2 Implement form state management and input binding
  - [x] 6.3 Add input validation feedback display
  - [x] 6.4 Create section components (API, qBittorrent, Paths, etc.)
  - _Requirements: 1.1, 2.1, 3.1_

- [x] 7. Implement API key masking in SettingsTab
  - [x] 7.1 Create masking utility function
  - [x] 7.2 Display masked API keys (show only last 4 characters)
  - [x] 7.3 Implement secure input handling for sensitive fields
  - [x] 7.4 Add copy-to-clipboard functionality for verification
  - _Requirements: 1.4_

- [x] 8. Implement file picker integration in SettingsTab
  - [x] 8.1 Add folder picker buttons for library and download paths
  - [x] 8.2 Integrate with Tauri's native file dialog
  - [x] 8.3 Update path fields with selected directories
  - [x] 8.4 Add path validation feedback
  - _Requirements: 3.2, 3.3_

- [x] 9. Implement notification system in SettingsTab
  - [x] 9.1 Create notification component for success/error messages
  - [x] 9.2 Implement auto-dismiss after 5 seconds
  - [x] 9.3 Add manual close button
  - [x] 9.4 Style notifications with appropriate colors
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 10. Implement qBittorrent test connection UI
  - [x] 10.1 Add "Test Connection" button in qBittorrent section
  - [x] 10.2 Display connection status (success/failure)
  - [x] 10.3 Show specific error messages
  - [x] 10.4 Auto-save on successful connection test
  - _Requirements: 2.2, 2.3, 2.4_

- [x] 11. Implement notification settings UI
  - [x] 11.1 Add Telegram configuration section (token, chat ID, debounce)
  - [x] 11.2 Add Email configuration section (SMTP details)
  - [x] 11.3 Add enable/disable toggles for each service
  - [x] 11.4 Add validation for notification settings
  - _Requirements: 7.1, 7.2, 7.3_

- [x] 12. Implement application settings UI
  - [x] 12.1 Add log level dropdown (DEBUG, INFO, WARN, ERROR)
  - [x] 12.2 Add numeric inputs for search limit, timeout, max downloads
  - [x] 12.3 Add enabled indexers configuration
  - [x] 12.4 Add range validation for numeric fields
  - _Requirements: 8.1, 8.2, 8.3, 8.4_

- [x] 13. Implement proxy settings UI
  - [ ] 13.1 Add proxy enable/disable toggle
  - [ ] 13.2 Add proxy address and port inputs
  - [ ] 13.3 Add validation for proxy configuration
  - [ ] 13.4 Show proxy status indicator
  - _Requirements: 9.1, 9.2, 9.3_

- [x] 14. Implement settings loading on app startup
  - [x] 14.1 Load settings from database when Engine initializes
  - [x] 14.2 Apply defaults for missing settings
  - [x] 14.3 Make settings available to all components
  - [x] 14.4 Add logging for settings initialization
  - _Requirements: 4.2, 4.3, 4.4_

- [x] 15. Implement settings persistence across restarts
  - [x] 15.1 Verify settings survive application close/reopen
  - [x] 15.2 Test with various setting combinations
  - [x] 15.3 Test with corrupted database entries
  - _Requirements: 4.3, 4.4_

- [x] 16. Implement graceful error handling
  - [x] 16.1 Handle database errors gracefully
  - [x] 16.2 Use defaults when settings cannot be loaded
  - [x] 16.3 Display user-friendly error messages
  - [x] 16.4 Log errors for debugging
  - _Requirements: 4.5, 6.5_

- [x] 17. Write property-based tests for settings persistence
  - [x] 17.1 Set up proptest framework for settings tests
  - [x] 17.2 Create generators for valid Settings objects
  - [x] 17.3 Implement round-trip test (save → load → compare)
  - [x] 17.4 Run 100+ iterations to verify consistency
  - **Property 1: Settings Persistence Round Trip**
  - **Validates: Requirements 4.1, 4.2, 4.3**

- [x] 18. Write property-based tests for settings validation
  - [x] 18.1 Create generators for invalid Settings objects
  - [x] 18.2 Implement validation rejection test
  - [x] 18.3 Verify specific error messages per field
  - [x] 18.4 Run 100+ iterations with various invalid inputs
  - **Property 2: Invalid Settings Rejection**
  - **Validates: Requirements 6.1, 6.2, 6.3**

- [x] 19. Write property-based tests for default values
  - [x] 19.1 Create test database with missing entries
  - [x] 19.2 Implement default value verification test
  - [x] 19.3 Test with corrupted database entries
  - [x] 19.4 Run 100+ iterations with various missing settings
  - **Property 3: Default Values on Missing Settings**
  - **Validates: Requirements 4.4, 4.5**

- [x] 20. Write property-based tests for qBittorrent connection
  - [x] 20.1 Set up mock qBittorrent server
  - [x] 20.2 Create generators for connection scenarios
  - [x] 20.3 Implement connection test with mocked responses
  - [x] 20.4 Verify correct error handling for failures
  - **Property 4: qBittorrent Connection Validation**
  - **Validates: Requirements 2.2, 2.3, 2.4**

- [x] 21. Write property-based tests for API key masking
  - [x] 21.1 Create generators for random API keys
  - [x] 21.2 Implement masking verification test
  - [x] 21.3 Verify only last 4 characters are visible
  - [x] 21.4 Run 100+ iterations with various key lengths
  - **Property 5: API Key Masking**
  - **Validates: Requirements 1.4**

- [x] 22. Write property-based tests for settings isolation
  - [x] 22.1 Create concurrent save scenarios
  - [x] 22.2 Implement isolation verification test
  - [x] 22.3 Verify no interference between concurrent saves
  - [x] 22.4 Run 100+ iterations with various concurrent patterns
  - **Property 6: Settings Isolation**
  - **Validates: Requirements 1.5, 2.5**

- [ ] 23. Write unit tests for Settings struct
  - [ ] 23.1 Test validation functions for each field type
  - [ ] 23.2 Test serialization/deserialization
  - [ ] 23.3 Test default value generation
  - [ ] 23.4 Test edge cases (empty strings, boundary values)

- [ ] 24. Write integration tests for settings workflow
  - [ ] 24.1 Test end-to-end save/load flow
  - [ ] 24.2 Test qBittorrent connection with real client
  - [ ] 24.3 Test file picker integration
  - [ ] 24.4 Test notification display

- [ ] 25. Checkpoint - Ensure all tests pass
  - [ ] 25.1 Run all unit tests
  - [ ] 25.2 Run all property-based tests
  - [ ] 25.3 Run all integration tests
  - [ ] 25.4 Fix any failing tests

- [ ] 26. Integrate settings into Engine initialization
  - [ ] 26.1 Load settings on Engine startup
  - [ ] 26.2 Use settings for qBittorrent client initialization
  - [ ] 26.3 Use settings for API client initialization
  - [ ] 26.4 Add logging for settings application
  - _Requirements: 4.2, 4.3_

- [ ] 27. Update qBittorrent client to use settings
  - [ ] 27.1 Replace hardcoded values in `engine/src/engine.rs` with settings
  - [ ] 27.2 Update `add_download()` to use settings
  - [ ] 27.3 Update `pause_download()` to use settings
  - [ ] 27.4 Update `resume_download()` to use settings
  - [ ] 27.5 Update `delete_download()` to use settings
  - _Requirements: 2.5_

- [ ] 28. Update API clients to use settings
  - [ ] 28.1 Update TMDB client to use API key from settings
  - [ ] 28.2 Update Kinopoisk client to use token from settings
  - [ ] 28.3 Remove hardcoded API keys from code
  - [ ] 28.4 Add logging for API client initialization
  - _Requirements: 1.2, 1.5_

- [ ] 29. Implement settings UI tab integration
  - [ ] 29.1 Add SettingsTab to main TabBar navigation
  - [ ] 29.2 Ensure tab is accessible from main application
  - [ ] 29.3 Add icon and label for Settings tab
  - [ ] 29.4 Test tab switching and state persistence
  - _Requirements: 1.1, 2.1, 3.1_

- [ ] 30. Final Checkpoint - Ensure all tests pass
  - [ ] 30.1 Run all tests one final time
  - [ ] 30.2 Verify no hardcoded values remain
  - [ ] 30.3 Test complete workflow end-to-end
  - [ ] 30.4 Document any known issues

---

## Implementation Notes

### Database Schema

The settings table uses a key-value pattern for flexibility:

```sql
CREATE TABLE IF NOT EXISTS settings (
    id INTEGER PRIMARY KEY,
    key TEXT UNIQUE NOT NULL,
    value TEXT NOT NULL,
    type TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### Settings Loading Strategy

1. On Engine startup, load all settings from database
2. For each missing setting, use default value
3. Validate all loaded settings
4. Make settings available via Engine API

### Validation Strategy

- URL validation: Must be valid HTTP(S) URL
- Path validation: Must be absolute path and exist on filesystem
- Numeric validation: Must be within acceptable ranges
- Optional fields: Can be empty/None

### Error Handling

- Database errors: Log and use defaults
- Validation errors: Return specific error per field
- Connection errors: Return specific error message
- File picker errors: Handle gracefully, allow retry

### Testing Approach

- Unit tests: Validation logic, serialization, defaults
- Property tests: Round trips, isolation, masking, validation
- Integration tests: End-to-end workflows, real connections
- All tests use 100+ iterations for property-based testing

---

## Success Criteria

- [ ] All settings can be configured via UI
- [ ] Settings persist across application restarts
- [ ] Invalid settings are rejected with specific error messages
- [ ] qBittorrent connection can be tested from UI
- [ ] API keys are masked in UI
- [ ] File picker works for path selection
- [ ] Notifications display for save success/failure
- [ ] All tests pass (unit, property, integration)
- [ ] No hardcoded values remain in engine code
- [ ] Settings are loaded on app startup
