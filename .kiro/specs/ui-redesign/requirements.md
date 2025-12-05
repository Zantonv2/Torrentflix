# Requirements Document

## Introduction

This document specifies the requirements for a comprehensive UI redesign of TorrentFlix, a local desktop application for torrent discovery, download management, and media library organization. The current UI suffers from critical architectural problems: 23 components with unclear responsibilities, broken backend integration, no state management, inconsistent design, and poor maintainability.

**Critical Problems Identified:**

1. **Backend/UI Mismatch**: 16+ Tauri commands exist but are never called from UI (query_library, search_library, get_cleanup_candidates, execute_cleanup, get_storage_analytics, create_collection, etc.)
2. **Broken Components**: LibraryBrowser calls `query_library` but the command isn't registered in main.rs. StorageAnalytics calls `get_storage_analytics` which exists but may not be wired up.
3. **No State Management**: All state lives in App.svelte (300+ lines), props drilling everywhere, rating updates via fragile event system
4. **Massive Components**: SettingsTab.svelte is 773+ lines handling 50+ form fields
5. **Confusing Navigation**: 7 tabs with misleading names (History = StorageAnalytics, Bookmarks = CollectionManager)
6. **No Responsive Design**: MovieGrid hardcoded to 5 columns, no mobile breakpoints
7. **Zero Accessibility**: No ARIA labels, no keyboard navigation, no focus management
8. **Performance Issues**: JobsDashboard polls every 2 seconds, no lazy loading

**Scope**: Complete UI overhaul with lean component architecture, proper state management, and full backend integration.

**Backend Changes**: NONE - all Tauri commands already exist and work.

## Glossary

- **TorrentFlix_UI**: The Svelte/TypeScript frontend application running in Tauri
- **Engine**: The Rust backend providing all business logic via Tauri commands
- **MediaItem**: A movie or series entity in the library database
- **FileVersion**: A specific file/quality variant of a MediaItem
- **Collection**: A user-created grouping of MediaItems (manual or smart/dynamic)
- **Store**: A Svelte writable store managing reactive state
- **Component**: A self-contained Svelte UI element with props and events
- **Page**: A top-level view component representing a navigation destination
- **Core Component**: A reusable primitive component (Button, Input, Modal, etc.)

## Requirements

### Requirement 1: Navigation Architecture

**User Story:** As a user, I want clear, intuitive navigation, so that I can quickly access different features without confusion.

#### Acceptance Criteria

1. WHEN the TorrentFlix_UI loads THEN the system SHALL display exactly 4 navigation destinations: Discover, Library, Downloads, and Settings
2. WHEN a user clicks a navigation item THEN the TorrentFlix_UI SHALL switch to the corresponding page within 100ms without full page reload
3. WHEN a user is on a page THEN the TorrentFlix_UI SHALL visually indicate the active navigation item with distinct styling
4. WHEN the TorrentFlix_UI window is resized below 768px width THEN the navigation SHALL collapse to a compact mobile-friendly format
5. IF a navigation action fails THEN the TorrentFlix_UI SHALL display an error message and remain on the current page

### Requirement 2: State Management Architecture

**User Story:** As a developer, I want centralized state management, so that data flows predictably and components stay synchronized.

#### Acceptance Criteria

1. WHEN the TorrentFlix_UI initializes THEN the system SHALL create Svelte stores for: search, library, downloads, settings, and ui state
2. WHEN any component updates shared state THEN all subscribed components SHALL receive the update within the same render cycle
3. WHEN a Tauri command returns data THEN the corresponding store SHALL update atomically without partial state
4. WHEN a store update fails THEN the system SHALL preserve the previous state and set an error flag
5. WHEN the TorrentFlix_UI unmounts THEN all store subscriptions SHALL be cleaned up to prevent memory leaks

### Requirement 3: Discover Page (Search & Feed)

**User Story:** As a user, I want to search for movies and browse recent releases, so that I can find content to download.

#### Acceptance Criteria

1. WHEN the Discover page loads THEN the TorrentFlix_UI SHALL call `get_feed` and display results in a responsive grid
2. WHEN a user enters a search query and presses Enter THEN the TorrentFlix_UI SHALL call `search_movies` with the query
3. WHEN search results return THEN the TorrentFlix_UI SHALL display movie cards with poster, title, year, rating, and quality badge
4. WHEN a user clicks a movie card THEN the TorrentFlix_UI SHALL open a detail modal with full metadata
5. WHEN a user clicks Download in the detail modal THEN the TorrentFlix_UI SHALL call `start_download` with the magnet link
6. WHILE search is in progress THEN the TorrentFlix_UI SHALL display skeleton loading placeholders
7. IF search returns zero results THEN the TorrentFlix_UI SHALL display a helpful empty state message
8. IF search fails THEN the TorrentFlix_UI SHALL display an error with a retry button
9. WHEN rating updates arrive via `rating-update` event THEN the TorrentFlix_UI SHALL update the corresponding movie card reactively

### Requirement 4: Library Page (Browser & Collections)

**User Story:** As a user, I want to browse my downloaded media library and organize items into collections, so that I can manage my content.

#### Acceptance Criteria

1. WHEN the Library page loads THEN the TorrentFlix_UI SHALL call `query_library` with default filters and display results
2. WHEN a user enters a search query THEN the TorrentFlix_UI SHALL call `search_library` with fuzzy matching enabled
3. WHEN a user applies filters (year, resolution, rating, watch status) THEN the TorrentFlix_UI SHALL call `query_library` with updated filters
4. WHEN a user clicks a library item THEN the TorrentFlix_UI SHALL call `get_media_item` and `get_file_versions` to show details
5. WHEN a user creates a collection THEN the TorrentFlix_UI SHALL call `create_collection` and refresh the collections list
6. WHEN a user creates a smart collection THEN the TorrentFlix_UI SHALL call `create_smart_collection` with filter criteria
7. WHEN a user adds items to a collection THEN the TorrentFlix_UI SHALL call `add_to_collection` with selected media IDs
8. WHEN a user views a collection THEN the TorrentFlix_UI SHALL call `get_collection_items` and display the results
9. WHEN a user sets a rating THEN the TorrentFlix_UI SHALL call `set_user_rating` and update the UI optimistically
10. WHEN a user changes watch status THEN the TorrentFlix_UI SHALL call `set_watch_status` and update the UI
11. WHEN a user adds tags THEN the TorrentFlix_UI SHALL call `add_tags` and update the item display
12. WHEN a user adds notes THEN the TorrentFlix_UI SHALL call `set_custom_notes` and persist immediately

### Requirement 5: Downloads Page

**User Story:** As a user, I want to monitor and control my active downloads, so that I can manage bandwidth and prioritize content.

#### Acceptance Criteria

1. WHEN the Downloads page loads THEN the TorrentFlix_UI SHALL call `get_active_downloads` and display all downloads
2. WHILE the Downloads page is visible THEN the TorrentFlix_UI SHALL poll `get_active_downloads` every 5 seconds (not 2 seconds)
3. WHEN a download status changes THEN the TorrentFlix_UI SHALL update the progress bar and status text reactively
4. WHEN a user clicks Pause THEN the TorrentFlix_UI SHALL call `pause_download` with the torrent hash
5. WHEN a user clicks Resume THEN the TorrentFlix_UI SHALL call `resume_download` with the torrent hash
6. WHEN a user clicks Delete THEN the TorrentFlix_UI SHALL show a confirmation dialog before calling `delete_download`
7. IF no downloads are active THEN the TorrentFlix_UI SHALL display an empty state with guidance
8. WHEN the Downloads page unmounts THEN the TorrentFlix_UI SHALL stop the polling interval

### Requirement 6: Settings Page

**User Story:** As a user, I want to configure API keys, qBittorrent connection, and application preferences, so that the app works with my setup.

#### Acceptance Criteria

1. WHEN the Settings page loads THEN the TorrentFlix_UI SHALL call `get_settings` and populate all form fields
2. WHEN a user modifies any setting THEN the TorrentFlix_UI SHALL track the change without auto-saving
3. WHEN a user clicks Save THEN the TorrentFlix_UI SHALL call `save_settings` with all current values
4. IF settings validation fails THEN the TorrentFlix_UI SHALL display field-level error messages
5. WHEN a user clicks Test Connection for qBittorrent THEN the TorrentFlix_UI SHALL call `test_qbittorrent` and show result
6. WHEN a user clicks Browse for a path THEN the TorrentFlix_UI SHALL call `pick_folder` and update the field
7. WHEN settings save successfully THEN the TorrentFlix_UI SHALL display a success toast notification
8. WHEN sensitive fields (API keys, passwords) are displayed THEN the TorrentFlix_UI SHALL mask the values by default

### Requirement 7: Storage & Maintenance Features

**User Story:** As a user, I want to view storage analytics and clean up duplicate/low-quality files, so that I can manage disk space.

#### Acceptance Criteria

1. WHEN a user accesses storage analytics THEN the TorrentFlix_UI SHALL call `get_storage_analytics` and display breakdown
2. WHEN a user requests cleanup candidates THEN the TorrentFlix_UI SHALL call `get_cleanup_candidates` and list them
3. WHEN a user confirms cleanup THEN the TorrentFlix_UI SHALL call `execute_cleanup` with selected candidate IDs
4. WHEN a user triggers a library rescan THEN the TorrentFlix_UI SHALL call `schedule_rescan` and show job status
5. IF cleanup would delete files THEN the TorrentFlix_UI SHALL require explicit confirmation with file count and size

### Requirement 8: Core Component Library

**User Story:** As a developer, I want reusable primitive components, so that the UI is consistent and maintainable.

#### Acceptance Criteria

1. THE TorrentFlix_UI SHALL provide a Button component supporting variants: primary, secondary, ghost, danger
2. THE TorrentFlix_UI SHALL provide an Input component supporting types: text, password, number, with optional masking
3. THE TorrentFlix_UI SHALL provide a Modal component with backdrop, close button, and keyboard escape handling
4. THE TorrentFlix_UI SHALL provide a Card component with optional hover effects and click handling
5. THE TorrentFlix_UI SHALL provide a Loading component supporting modes: spinner, skeleton, dots
6. THE TorrentFlix_UI SHALL provide a Toast component for success, error, and info notifications
7. THE TorrentFlix_UI SHALL provide a Tabs component for switching between sub-views within a page
8. THE TorrentFlix_UI SHALL provide a Badge component for status indicators and quality labels

### Requirement 9: Responsive Design

**User Story:** As a user, I want the app to work well on different screen sizes, so that I can use it on my laptop or external monitor.

#### Acceptance Criteria

1. WHEN the viewport width is below 640px THEN the TorrentFlix_UI SHALL use single-column layouts
2. WHEN the viewport width is between 640px and 1024px THEN the TorrentFlix_UI SHALL use 2-3 column grids
3. WHEN the viewport width is above 1024px THEN the TorrentFlix_UI SHALL use 4-6 column grids for movie displays
4. WHEN the viewport changes THEN the TorrentFlix_UI SHALL reflow content without horizontal scrolling
5. THE TorrentFlix_UI SHALL use Tailwind responsive prefixes (sm:, md:, lg:, xl:) consistently

### Requirement 10: Accessibility

**User Story:** As a user with accessibility needs, I want the app to be keyboard navigable and screen reader compatible, so that I can use it effectively.

#### Acceptance Criteria

1. THE TorrentFlix_UI SHALL provide ARIA labels for all interactive elements
2. WHEN a user presses Tab THEN focus SHALL move through interactive elements in logical order
3. WHEN a modal opens THEN focus SHALL move to the modal and trap until closed
4. WHEN a modal closes THEN focus SHALL return to the triggering element
5. THE TorrentFlix_UI SHALL maintain a minimum color contrast ratio of 4.5:1 for text
6. WHEN a user presses Escape THEN any open modal or dropdown SHALL close

### Requirement 11: Error Handling & Recovery

**User Story:** As a user, I want clear error messages and recovery options, so that I can resolve issues without frustration.

#### Acceptance Criteria

1. WHEN a Tauri command fails THEN the TorrentFlix_UI SHALL display a user-friendly error message
2. WHEN a network request times out THEN the TorrentFlix_UI SHALL offer a retry option
3. WHEN an error occurs THEN the TorrentFlix_UI SHALL log details to console for debugging
4. IF the backend is unreachable THEN the TorrentFlix_UI SHALL display a connection error state
5. WHEN recovering from an error THEN the TorrentFlix_UI SHALL restore the previous valid state

### Requirement 12: Performance

**User Story:** As a user, I want the app to be fast and responsive, so that I can work efficiently.

#### Acceptance Criteria

1. WHEN the TorrentFlix_UI loads THEN the initial render SHALL complete within 500ms
2. WHEN navigating between pages THEN the transition SHALL complete within 100ms
3. WHEN displaying movie grids THEN the TorrentFlix_UI SHALL lazy-load images outside the viewport
4. WHEN polling for updates THEN the TorrentFlix_UI SHALL use a minimum 5-second interval (not 2 seconds)
5. THE TorrentFlix_UI SHALL debounce search input by 300ms to prevent excessive API calls

### Requirement 13: Internationalization (Russian Localization)

**User Story:** As a Russian user, I want the entire UI to be in Russian, so that I can use the app in my native language.

#### Acceptance Criteria

1. WHEN the TorrentFlix_UI loads THEN all text SHALL be displayed in Russian
2. WHEN a user navigates to any page THEN all labels, buttons, placeholders, and messages SHALL be in Russian
3. WHEN an error occurs THEN the error message SHALL be displayed in Russian
4. WHEN a user hovers over elements THEN tooltips and help text SHALL be in Russian
5. WHEN the app displays notifications THEN success, error, and info messages SHALL be in Russian
6. WHEN the app displays form validation errors THEN error messages SHALL be in Russian
7. WHEN the app displays empty states THEN empty state messages SHALL be in Russian
8. WHEN the app displays loading states THEN loading messages SHALL be in Russian
