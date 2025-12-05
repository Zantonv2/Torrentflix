# Implementation Plan: TorrentFlix UI Redesign (Complete Rewrite)

## Overview

Complete UI redesign from scratch with upgraded tech stack (Svelte 5, Vite 7, Tailwind 4), refined architecture, and comprehensive testing. All old components will be replaced with a lean, focused component library. All tasks are required for a production-ready implementation.

---

## Phase 0: Tech Stack Upgrades & Project Setup

- [ ] 0. Upgrade dependencies and initialize new project structure
  - [ ] 0.1 Upgrade Vite 4.4 → 7.x - Update package.json, verify build
  - [ ] 0.2 Upgrade Svelte 4 → 5 - Update package.json, migrate to runes
  - [ ] 0.3 Upgrade Tailwind 3 → 4 - Update package.json, update config
  - [ ] 0.4 Upgrade TypeScript 5.0 → 5.5+ - Update package.json, fix type errors
  - [ ] 0.5 Update @tauri-apps/api to latest - Update package.json
  - [ ] 0.6 Install svelte-i18n for localization - Add to package.json
  - [ ] 0.7 Run npm audit fix - Resolve security issues
  - [ ] 0.8 Verify app builds and runs - Test basic functionality
  - _Requirements: 12.1_

- [ ] 0.9 Create new component directory structure
  - [ ] 0.9.1 Create ui/src/components/core/ for primitive components
  - [ ] 0.9.2 Create ui/src/components/layout/ for layout components
  - [ ] 0.9.3 Create ui/src/components/pages/ for page components
  - [ ] 0.9.4 Create ui/src/stores/ for Svelte stores
  - [ ] 0.9.5 Create ui/src/i18n/ for localization files

- [ ] 0.10 Unit tests for setup
  - [ ] 0.10.1 Write unit tests for build process
  - [ ] 0.10.2 Verify all dependencies resolve correctly

---

## Phase 1: Core Components Library

- [ ] 1. Create primitive components (ui/src/components/core/)
  - [ ] 1.1 Button.svelte - Variants: primary, secondary, ghost, danger; sizes: sm, md, lg
  - [ ] 1.2 Input.svelte - Types: text, password, number, email; with masking support
  - [ ] 1.3 Modal.svelte - Focus trap, Escape key handling, backdrop
  - [ ] 1.4 Card.svelte - Dark background, hover effects, padding variants
  - [ ] 1.5 Badge.svelte - Variants: default, success, warning, error, info
  - [ ] 1.6 Loading.svelte - Modes: spinner, skeleton, dots animations
  - [ ] 1.7 Toast.svelte - Auto-dismiss, types: success, error, info, warning
  - [ ] 1.8 Tabs.svelte - Underline active state, keyboard navigation
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8_

- [ ] 1.9 Unit tests for core components
  - [ ] 1.9.1 Write unit tests for Button component (all variants)
  - [ ] 1.9.2 Write unit tests for Input component (all types, masking)
  - [ ] 1.9.3 Write unit tests for Modal component (focus trap, escape)
  - [ ] 1.9.4 Write unit tests for Card component (hover, padding)
  - [ ] 1.9.5 Write unit tests for Badge component (all variants)
  - [ ] 1.9.6 Write unit tests for Loading component (all modes)
  - [ ] 1.9.7 Write unit tests for Toast component (auto-dismiss, types)
  - [ ] 1.9.8 Write unit tests for Tabs component (keyboard nav)

---

## Phase 2: State Management (Svelte Stores)

- [ ] 2. Create centralized stores (ui/src/stores/)
  - [ ] 2.1 searchStore.ts - query, results, loading, error state
  - [ ] 2.2 libraryStore.ts - items, filters, selectedItem, collections
  - [ ] 2.3 downloadStore.ts - active downloads, polling interval (5s)
  - [ ] 2.4 settingsStore.ts - data, dirty state, validation errors
  - [ ] 2.5 uiStore.ts - currentPage, modal state, toast queue, sidebarCollapsed
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [ ] 2.6 Unit tests for stores
  - [ ] 2.6.1 Write unit tests for searchStore (state updates, subscriptions)
  - [ ] 2.6.2 Write unit tests for libraryStore (filters, selections)
  - [ ] 2.6.3 Write unit tests for downloadStore (polling lifecycle)
  - [ ] 2.6.4 Write unit tests for settingsStore (dirty state tracking)
  - [ ] 2.6.5 Write unit tests for uiStore (page routing, modals)

- [ ] 2.7 Property-based tests for stores
  - [ ] 2.7.1 Write property test for store atomicity
  - **Property 2: Store Atomicity**
  - **Validates: Requirements 2.3**

---

## Phase 3: Layout & Navigation Components

- [ ] 3. Create layout components (ui/src/components/layout/)
  - [ ] 3.1 MainLayout.svelte - Flex layout: Header + Sidebar + Content
  - [ ] 3.2 Header.svelte - Logo, search input (debounce 300ms), search button
  - [ ] 3.3 Sidebar.svelte - 4 nav items (Открыть, Библиотека, Загрузки, Настройки)
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [ ] 3.4 Unit tests for layout components
  - [ ] 3.4.1 Write unit tests for MainLayout (flex structure, responsive)
  - [ ] 3.4.2 Write unit tests for Header (search debounce, button click)
  - [ ] 3.4.3 Write unit tests for Sidebar (nav items, active state)

- [ ] 4. Implement navigation & routing
  - [ ] 4.1 Update App.svelte - Use MainLayout, route to pages via uiStore
  - [ ] 4.2 Add keyboard navigation - Tab, Arrow keys, Escape
  - [ ] 4.3 Add ARIA labels - All interactive elements
  - [ ] 4.4 Responsive design - Mobile hamburger, tablet 2-3 cols, desktop 4-6 cols
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 9.1, 9.2, 9.3, 9.4, 9.5, 10.1, 10.2, 10.3, 10.4, 10.5, 10.6_

- [ ] 4.5 Unit tests for navigation
  - [ ] 4.5.1 Write unit tests for App.svelte routing
  - [ ] 4.5.2 Write unit tests for keyboard navigation
  - [ ] 4.5.3 Write unit tests for ARIA labels

- [ ] 4.6 Property-based tests for navigation
  - [ ] 4.6.1 Write property test for navigation state consistency
  - **Property 1: Navigation State Consistency**
  - **Validates: Requirements 1.1, 1.3**

- [ ] 4.7 Property-based tests for responsive layout
  - [ ] 4.7.1 Write property test for responsive layout reflow
  - **Property 9: Responsive Layout Reflow**
  - **Validates: Requirements 9.4**

- [ ] 4.8 Property-based tests for focus management
  - [ ] 4.8.1 Write property test for modal focus trap
  - **Property 6: Modal Focus Trap**
  - **Validates: Requirements 10.3, 10.4, 10.6**

- [ ] 4.9 Property-based tests for keyboard navigation
  - [ ] 4.9.1 Write property test for accessibility focus order
  - **Property 10: Accessibility Focus Order**
  - **Validates: Requirements 10.2**

---

## Phase 4: Discover Page

- [ ] 5. Build Discover page (ui/src/components/pages/Discover.svelte)
  - [ ] 5.1 On mount: Call `invoke('get_feed')` → display results
  - [ ] 5.2 Movie grid - Responsive columns (1/2-3/4-6 based on viewport)
  - [ ] 5.3 Movie card - Poster (2:3), title, year, rating, quality badge
  - [ ] 5.4 Card hover - Show "View Details" overlay button
  - [ ] 5.5 Click card - Open Modal with full details
  - [ ] 5.6 Search input - Debounce 300ms, on Enter: Call `invoke('search_movies')`
  - [ ] 5.7 Loading state - Show skeleton placeholders
  - [ ] 5.8 Empty state - "Ничего не найдено" message
  - [ ] 5.9 Error state - Red error box + "Повторить" button
  - [ ] 5.10 Download button in modal - Call `invoke('start_download')`
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9_

- [ ] 5.11 Unit tests for Discover page
  - [ ] 5.11.1 Write unit tests for feed loading
  - [ ] 5.11.2 Write unit tests for search functionality
  - [ ] 5.11.3 Write unit tests for movie card interactions
  - [ ] 5.11.4 Write unit tests for error handling

- [ ] 5.12 Property-based tests for Discover page
  - [ ] 5.12.1 Write property test for search results display
  - **Property 3: Search Results Display**
  - **Validates: Requirements 3.1, 9.1, 9.2, 9.3**

- [ ] 5.13 Property-based tests for search debouncing
  - [ ] 5.13.1 Write property test for search debouncing
  - **Property 12: Search Debouncing**
  - **Validates: Requirements 12.5**

- [ ] 5.14 Property-based tests for image lazy loading
  - [ ] 5.14.1 Write property test for image lazy loading
  - **Property 11: Image Lazy Loading**
  - **Validates: Requirements 12.3**

---

## Phase 5: Library Page

- [ ] 6. Build Library Browser tab (ui/src/components/pages/LibraryBrowser.svelte)
  - [ ] 6.1 On mount: Call `invoke('query_library', {filters: {}})`
  - [ ] 6.2 Search input - "Поиск в библиотеке...", fuzzy matching
  - [ ] 6.3 Filters - Year, Resolution, Rating (slider), Watch Status
  - [ ] 6.4 Grid/List toggle - Switch between grid and list view
  - [ ] 6.5 Click item - Call `invoke('get_media_item')` + `invoke('get_file_versions')`
  - [ ] 6.6 Detail modal - Show metadata, ratings, watch status, user rating, tags, notes
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 4.9, 4.10, 4.11, 4.12_

- [ ] 6.7 Unit tests for Library Browser
  - [ ] 6.7.1 Write unit tests for library loading
  - [ ] 6.7.2 Write unit tests for search and filters
  - [ ] 6.7.3 Write unit tests for grid/list toggle
  - [ ] 6.7.4 Write unit tests for detail modal

- [ ] 7. Build Library Collections tab (ui/src/components/pages/LibraryCollections.svelte)
  - [ ] 7.1 List collections (left panel, 300px)
  - [ ] 7.2 "+ New" button - Modal to create collection
  - [ ] 7.3 "🧠 Smart" button - Modal for smart collection
  - [ ] 7.4 Add items to collection - Call `invoke('add_to_collection')`
  - _Requirements: 4.5, 4.6, 4.7, 4.8_

- [ ] 7.5 Unit tests for Library Collections
  - [ ] 7.5.1 Write unit tests for collection creation
  - [ ] 7.5.2 Write unit tests for smart collection creation
  - [ ] 7.5.3 Write unit tests for adding items to collection

- [ ] 8. Build Library Analytics tab (ui/src/components/pages/LibraryAnalytics.svelte)
  - [ ] 8.1 On mount: Call `invoke('get_storage_analytics')`
  - [ ] 8.2 Display stats - Total size, media count, file version count, avg versions
  - [ ] 8.3 Breakdown charts - By resolution, quality, status
  - [ ] 8.4 Largest items - Top 10 list
  - [ ] 8.5 Duplicate space warning - Yellow box if duplicates exist
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [ ] 8.6 Unit tests for Library Analytics
  - [ ] 8.6.1 Write unit tests for analytics loading
  - [ ] 8.6.2 Write unit tests for chart rendering
  - [ ] 8.6.3 Write unit tests for duplicate warning

- [ ] 9. Build Library Cleanup tab (ui/src/components/pages/LibraryCleanup.svelte)
  - [ ] 9.1 On mount: Call `invoke('get_cleanup_candidates')`
  - [ ] 9.2 List candidates - File name, size, reason
  - [ ] 9.3 Checkbox selection - Select candidates to delete
  - [ ] 9.4 Confirmation modal - "Вы уверены? Это удалит X файлов (Y GB)"
  - [ ] 9.5 Execute cleanup - Call `invoke('execute_cleanup')`
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [ ] 9.6 Unit tests for Library Cleanup
  - [ ] 9.6.1 Write unit tests for cleanup candidates loading
  - [ ] 9.6.2 Write unit tests for candidate selection
  - [ ] 9.6.3 Write unit tests for cleanup execution

---

## Phase 6: Downloads Page

- [ ] 10. Build Downloads page (ui/src/components/pages/Downloads.svelte)
  - [ ] 10.1 On mount: Call `invoke('get_active_downloads')` + start polling (5s)
  - [ ] 10.2 Download list - Title, progress bar, stats (%, size, speed)
  - [ ] 10.3 Pause button - Call `invoke('pause_download')`
  - [ ] 10.4 Resume button - Call `invoke('resume_download')`
  - [ ] 10.5 Delete button - Confirmation modal → Call `invoke('delete_download')`
  - [ ] 10.6 Empty state - "Нет активных загрузок"
  - [ ] 10.7 Stop polling on unmount
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8_

- [ ] 10.8 Unit tests for Downloads page
  - [ ] 10.8.1 Write unit tests for downloads loading
  - [ ] 10.8.2 Write unit tests for pause/resume functionality
  - [ ] 10.8.3 Write unit tests for delete functionality
  - [ ] 10.8.4 Write unit tests for polling lifecycle

- [ ] 10.9 Property-based tests for download polling
  - [ ] 10.9.1 Write property test for download polling lifecycle
  - **Property 5: Download Polling Lifecycle**
  - **Validates: Requirements 5.2, 5.8**

---

## Phase 7: Settings Page

- [ ] 11. Build Settings page (ui/src/components/pages/Settings.svelte)
  - [ ] 11.1 On mount: Call `invoke('get_settings')`
  - [ ] 11.2 API & Metadata section - TMDB API Key, Kinopoisk Token (masked)
  - [ ] 11.3 qBittorrent section - Host, Port, Username, Password, SSL options
  - [ ] 11.4 Test Connection button - Call `invoke('test_qbittorrent')`
  - [ ] 11.5 Filesystem section - Library Path, Download Path (with Browse buttons)
  - [ ] 11.6 Browse button - Call `invoke('pick_folder')` → update field
  - [ ] 11.7 Notifications section - Telegram, Email settings
  - [ ] 11.8 Application section - Log level, search limit, timeout, max downloads
  - [ ] 11.9 Proxy section - Enable, address, port
  - [ ] 11.10 Save button - Call `invoke('save_settings')`, show validation errors
  - [ ] 11.11 Dirty state tracking - Warn on navigation if unsaved
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8_

- [ ] 11.12 Unit tests for Settings page
  - [ ] 11.12.1 Write unit tests for settings loading
  - [ ] 11.12.2 Write unit tests for form validation
  - [ ] 11.12.3 Write unit tests for settings saving
  - [ ] 11.12.4 Write unit tests for dirty state tracking

- [ ] 11.13 Property-based tests for settings dirty state
  - [ ] 11.13.1 Write property test for settings dirty state
  - **Property 7: Settings Dirty State**
  - **Validates: Requirements 6.2**

---

## Phase 8: Localization & Internationalization

- [ ] 12. Add Russian localization (ui/src/i18n/)
  - [ ] 12.1 Install svelte-i18n
  - [ ] 12.2 Create i18n config - Russian default, English fallback
  - [ ] 12.3 Create translation files - ru.json, en.json
  - [ ] 12.4 Replace all hardcoded strings with $t() calls
  - _Requirements: 13.1, 13.2, 13.3, 13.4, 13.5, 13.6, 13.7, 13.8_

- [ ] 12.5 Unit tests for localization
  - [ ] 12.5.1 Write unit tests for i18n initialization
  - [ ] 12.5.2 Write unit tests for translation loading
  - [ ] 12.5.3 Write unit tests for language switching

- [ ] 12.6 Property-based tests for localization
  - [ ] 12.6.1 Write property test for Russian localization completeness
  - **Property 8: Russian Localization Completeness**
  - **Validates: Requirements 13.1, 13.2, 13.3, 13.4, 13.5, 13.6, 13.7, 13.8**

---

## Phase 9: Error Handling & Recovery

- [ ] 13. Error handling & recovery
  - [ ] 13.1 Wrap all invoke() calls in try-catch
  - [ ] 13.2 Preserve previous state on error
  - [ ] 13.3 Display error toasts with Russian messages
  - [ ] 13.4 Show retry buttons on failures
  - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

- [ ] 13.5 Unit tests for error handling
  - [ ] 13.5.1 Write unit tests for error recovery
  - [ ] 13.5.2 Write unit tests for retry functionality
  - [ ] 13.5.3 Write unit tests for error toast display

- [ ] 13.6 Property-based tests for error recovery
  - [ ] 13.6.1 Write property test for error state recovery
  - **Property 4: Error State Recovery**
  - **Validates: Requirements 11.1, 11.5**

---

## Phase 10: Performance Optimization

- [ ] 14. Performance optimization
  - [ ] 14.1 Image lazy loading - loading="lazy" on all posters
  - [ ] 14.2 Search debouncing - 300ms delay
  - [ ] 14.3 Download polling - 5 second interval (not 2)
  - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5_

- [ ] 14.4 Unit tests for performance
  - [ ] 14.4.1 Write unit tests for image lazy loading
  - [ ] 14.4.2 Write unit tests for search debouncing
  - [ ] 14.4.3 Write unit tests for polling intervals

---

## Phase 11: Accessibility

- [ ] 15. Accessibility implementation
  - [ ] 15.1 ARIA labels - All buttons, inputs, modals
  - [ ] 15.2 Focus management - Focus trap in modals, restore on close
  - [ ] 15.3 Keyboard navigation - Tab, Arrow keys, Escape
  - [ ] 15.4 Color contrast - Verify 4.5:1 ratio
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5, 10.6_

- [ ] 15.5 Unit tests for accessibility
  - [ ] 15.5.1 Write unit tests for ARIA labels
  - [ ] 15.5.2 Write unit tests for focus management
  - [ ] 15.5.3 Write unit tests for keyboard navigation
  - [ ] 15.5.4 Write unit tests for color contrast

---

## Phase 12: Cleanup & Final Testing

- [ ] 16. Delete old components and consolidate
  - [ ] 16.1 Identify which of 23 old components can be deleted
  - [ ] 16.2 Delete unused components (MovieTile, SkeletonTile, LoadingDots, etc.)
  - [ ] 16.3 Consolidate overlapping functionality
  - _Requirements: All_

- [ ] 16.4 Integration tests
  - [ ] 16.4.1 Write integration tests for Discover page
  - [ ] 16.4.2 Write integration tests for Library page
  - [ ] 16.4.3 Write integration tests for Downloads page
  - [ ] 16.4.4 Write integration tests for Settings page

- [ ] 17. Final integration & testing
  - [ ] 17.1 Run full test suite (unit + property + integration)
  - [ ] 17.2 Verify all Tauri commands work end-to-end
  - [ ] 17.3 Test responsive design at all breakpoints
  - [ ] 17.4 Manual testing - All pages, all actions, all error states
  - [ ] 17.5 Verify zero console errors
  - [ ] 17.6 Performance profiling - Check load times, memory usage
  - _Requirements: All_

---

## Tauri Commands Reference (Already Implemented)

**Discover**: `get_feed()`, `search_movies(query, media_type, limit)`, `start_download(magnetLink, title)`

**Library**: `query_library(filters)`, `search_library(query, options)`, `get_media_item(id)`, `get_file_versions(media_id)`, `create_collection(name, description)`, `create_smart_collection(name, criteria)`, `add_to_collection(collection_id, media_ids)`, `get_collection_items(collection_id)`, `get_storage_analytics()`, `get_cleanup_candidates()`, `execute_cleanup(candidate_ids)`

**Downloads**: `get_active_downloads()`, `pause_download(hash)`, `resume_download(hash)`, `delete_download(hash, delete_files)`

**Settings**: `get_settings()`, `save_settings(settings)`, `test_qbittorrent(url, username, password)`, `pick_folder()`

**User Metadata**: `set_user_rating(media_id, rating)`, `add_tags(media_id, tags)`, `set_watch_status(media_id, status)`, `set_custom_notes(media_id, notes)`

---

## Success Criteria

- [ ] All 8 core components built and working
- [ ] All 5 stores managing state correctly
- [ ] All 4 pages fully functional
- [ ] All 26+ Tauri commands wired and tested
- [ ] Full Russian localization
- [ ] Responsive design (1/2-3/4-6 columns)
- [ ] Keyboard navigation working
- [ ] ARIA labels on all interactive elements
- [ ] Error handling with retry
- [ ] Old components deleted
- [ ] Zero console errors
- [ ] All unit tests passing
- [ ] All property tests passing
- [ ] All integration tests passing
