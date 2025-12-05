# Implementation Plan: TorrentFlix UI Redesign

## Current State Analysis

**Completed:**
- Basic app structure with App.svelte routing
- 23 existing components (MovieCard, MovieGrid, DetailPanel, etc.)
- Tauri commands for search, downloads, library, settings, collections, cleanup
- Rating update event system
- Basic error handling and loading states

**In Progress:**
- Svelte 4 (needs upgrade to 5)
- Vite 4.4 (needs upgrade to 7.x)
- Tailwind 3 (needs upgrade to 4)
- No centralized state management (stores)
- No core component library (Button, Input, Modal, etc.)
- No Russian localization (svelte-i18n)
- Limited accessibility (ARIA labels, keyboard nav)

**Not Started:**
- Responsive design refinement
- Focus trap and modal management
- Search debouncing
- Download polling lifecycle
- Settings dirty state tracking
- Component consolidation

---

## Phase 0: Tech Stack Upgrades

- [ ] 0. Upgrade dependencies
  - [ ] 0.1 Upgrade Vite 4.4 → 7.x - Update package.json, test build
  - [ ] 0.2 Upgrade Svelte 4 → 5 - Update package.json, review runes migration
  - [ ] 0.3 Upgrade Tailwind 3 → 4 - Update package.json, review config changes
  - [ ] 0.4 Upgrade TypeScript 5.0 → 5.5+ - Update package.json, fix type errors
  - [ ] 0.5 Update @tauri-apps/api to latest - Update package.json
  - [ ] 0.6 Run npm audit fix - Resolve security issues
  - [ ] 0.7 Verify app builds and runs - Test basic functionality
  - _Requirements: 12.1_

---

## Phase 1: Core Components & Stores

- [ ] 1. Create core components library
  - [ ] 1.1 Button.svelte - All variants (primary, secondary, ghost, danger)
  - [ ] 1.2 Input.svelte - Text, password, number, email with masking
  - [ ] 1.3 Modal.svelte - Focus trap, Escape key, backdrop
  - [ ] 1.4 Card.svelte - Dark background, hover effects
  - [ ] 1.5 Badge.svelte - Status badges (success, error, warning, info)
  - [ ] 1.6 Loading.svelte - Spinner, skeleton, dots animations
  - [ ] 1.7 Toast.svelte - Auto-dismiss notifications
  - [ ] 1.8 Tabs.svelte - Tab navigation with underline active state
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8_

- [ ] 2. Create Svelte stores
  - [ ] 2.1 searchStore.ts - query, results, loading, error
  - [ ] 2.2 libraryStore.ts - items, filters, selectedItem, collections
  - [ ] 2.3 downloadStore.ts - active downloads, polling (5s interval)
  - [ ] 2.4 settingsStore.ts - data, dirty state, validation errors
  - [ ] 2.5 uiStore.ts - currentPage, modal, toast, sidebarCollapsed
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [ ]* 2.6 Write property tests for store atomicity
  - **Property 2: Store Atomicity**
  - **Validates: Requirements 2.3**

---

## Phase 2: Layout & Navigation

- [ ] 3. Create layout components
  - [ ] 3.1 MainLayout.svelte - Flex layout, Header + Sidebar + Content
  - [ ] 3.2 Header.svelte - Logo, search input (debounce 300ms), search button
  - [ ] 3.3 Sidebar.svelte - 4 nav items (Открыть, Библиотека, Загрузки, Настройки)
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [ ] 4. Implement navigation & routing
  - [ ] 4.1 Update App.svelte - Use MainLayout, route to pages via uiStore
  - [ ] 4.2 Add keyboard navigation - Tab, Arrow keys, Escape
  - [ ] 4.3 Add ARIA labels - All interactive elements
  - [ ] 4.4 Responsive design - Mobile hamburger, tablet 2-3 cols, desktop 4-6 cols
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 9.1, 9.2, 9.3, 9.4, 9.5, 10.1, 10.2, 10.3, 10.4, 10.5, 10.6_

- [ ]* 4.5 Write property tests for navigation consistency
  - **Property 1: Navigation State Consistency**
  - **Validates: Requirements 1.1, 1.3**

- [ ]* 4.6 Write property tests for responsive layout
  - **Property 9: Responsive Layout Reflow**
  - **Validates: Requirements 9.4**

- [ ]* 4.7 Write property tests for focus management
  - **Property 6: Modal Focus Trap**
  - **Validates: Requirements 10.3, 10.4, 10.6**

- [ ]* 4.8 Write property tests for keyboard navigation
  - **Property 10: Accessibility Focus Order**
  - **Validates: Requirements 10.2**

---

## Phase 3: Discover Page

- [ ] 5. Build Discover page
  - [ ] 5.1 On mount: Call `invoke('get_feed')` → display results
  - [ ] 5.2 Movie grid - Responsive columns (1/2-3/4-6 based on viewport)
  - [ ] 5.3 Movie card - Poster (2:3), title, year, rating, quality badge
  - [ ] 5.4 Card hover - Show "View Details" overlay button
  - [ ] 5.5 Click card - Open Modal with full details (title, description, genres, cast, runtime, ratings)
  - [ ] 5.6 Search input - Debounce 300ms, on Enter: Call `invoke('search_movies', {query, media_type: 'movie', limit: 50})`
  - [ ] 5.7 Loading state - Show skeleton placeholders
  - [ ] 5.8 Empty state - "Ничего не найдено" message
  - [ ] 5.9 Error state - Red error box + "Повторить" button
  - [ ] 5.10 Download button in modal - Call `invoke('start_download', {magnetLink, title})`
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9_

- [ ]* 5.11 Write property tests for search results display
  - **Property 3: Search Results Display**
  - **Validates: Requirements 3.1, 9.1, 9.2, 9.3**

- [ ]* 5.12 Write property tests for search debouncing
  - **Property 12: Search Debouncing**
  - **Validates: Requirements 12.5**

- [ ]* 5.13 Write property tests for image lazy loading
  - **Property 11: Image Lazy Loading**
  - **Validates: Requirements 12.3**

---

## Phase 4: Library Page

- [ ] 6. Build Library Browser tab
  - [ ] 6.1 On mount: Call `invoke('query_library', {filters: {}})`
  - [ ] 6.2 Search input - "Поиск в библиотеке...", on input: Call `invoke('search_library', {query, options: {fuzzy: true}})`
  - [ ] 6.3 Filters - Year, Resolution, Rating (slider), Watch Status
  - [ ] 6.4 Grid/List toggle - Switch between grid and list view
  - [ ] 6.5 Click item - Call `invoke('get_media_item', {id})` + `invoke('get_file_versions', {media_id})`
  - [ ] 6.6 Detail modal - Show metadata, ratings, watch status, user rating, tags, notes
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 4.9, 4.10, 4.11, 4.12_

- [ ] 7. Build Library Collections tab
  - [ ] 7.1 List collections (left panel, 300px) - Call `invoke('get_collection_items', {collection_id})` on click
  - [ ] 7.2 "+ New" button - Modal to create collection: Call `invoke('create_collection', {name, description})`
  - [ ] 7.3 "🧠 Smart" button - Modal for smart collection: Call `invoke('create_smart_collection', {name, criteria})`
  - [ ] 7.4 Add items to collection - Call `invoke('add_to_collection', {collection_id, media_ids})`
  - _Requirements: 4.5, 4.6, 4.7, 4.8_

- [ ] 8. Build Library Analytics tab
  - [ ] 8.1 On mount: Call `invoke('get_storage_analytics')`
  - [ ] 8.2 Display stats - Total size, media count, file version count, avg versions
  - [ ] 8.3 Breakdown charts - By resolution, quality, status
  - [ ] 8.4 Largest items - Top 10 list
  - [ ] 8.5 Duplicate space warning - Yellow box if duplicates exist
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [ ] 9. Build Library Cleanup tab
  - [ ] 9.1 On mount: Call `invoke('get_cleanup_candidates')`
  - [ ] 9.2 List candidates - File name, size, reason
  - [ ] 9.3 Checkbox selection - Select candidates to delete
  - [ ] 9.4 Confirmation modal - "Вы уверены? Это удалит X файлов (Y GB)"
  - [ ] 9.5 Execute cleanup - Call `invoke('execute_cleanup', {candidate_ids})`
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

---

## Phase 5: Downloads Page

- [ ] 10. Build Downloads page
  - [ ] 10.1 On mount: Call `invoke('get_active_downloads')` + start polling (5s)
  - [ ] 10.2 Download list - Title, progress bar, stats (%, size, speed)
  - [ ] 10.3 Pause button - Call `invoke('pause_download', {hash})`
  - [ ] 10.4 Resume button - Call `invoke('resume_download', {hash})`
  - [ ] 10.5 Delete button - Confirmation modal → Call `invoke('delete_download', {hash, delete_files: false})`
  - [ ] 10.6 Empty state - "Нет активных загрузок"
  - [ ] 10.7 Stop polling on unmount
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8_

- [ ]* 10.8 Write property tests for download polling lifecycle
  - **Property 5: Download Polling Lifecycle**
  - **Validates: Requirements 5.2, 5.8**

---

## Phase 6: Settings Page

- [ ] 11. Build Settings page
  - [ ] 11.1 On mount: Call `invoke('get_settings')`
  - [ ] 11.2 API & Metadata section - TMDB API Key, Kinopoisk Token (masked inputs)
  - [ ] 11.3 qBittorrent section - Host, Port, Username, Password, SSL options
  - [ ] 11.4 Test Connection button - Call `invoke('test_qbittorrent', {url, username, password})`
  - [ ] 11.5 Filesystem section - Library Path, Download Path (with Browse buttons)
  - [ ] 11.6 Browse button - Call `invoke('pick_folder')` → update field
  - [ ] 11.7 Notifications section - Telegram (token, chat ID, debounce), Email (SMTP settings)
  - [ ] 11.8 Application section - Log level, search limit, timeout, max downloads, indexers, debug mode
  - [ ] 11.9 Proxy section - Enable, address, port
  - [ ] 11.10 Save button - Call `invoke('save_settings', {settings})`, show validation errors inline
  - [ ] 11.11 Dirty state tracking - Warn on navigation if unsaved
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8_

- [ ]* 11.12 Write property tests for settings dirty state
  - **Property 7: Settings Dirty State**
  - **Validates: Requirements 6.2**

---

## Phase 7: Localization & Polish

- [ ] 12. Add Russian localization
  - [ ] 12.1 Install svelte-i18n
  - [ ] 12.2 Create i18n config - Russian default, English fallback
  - [ ] 12.3 Create translation files - ru.json, en.json
  - [ ] 12.4 Replace all hardcoded strings with $t() calls
  - _Requirements: 13.1, 13.2, 13.3, 13.4, 13.5, 13.6, 13.7, 13.8_

- [ ]* 12.5 Write property tests for Russian localization completeness
  - **Property 8: Russian Localization Completeness**
  - **Validates: Requirements 13.1, 13.2, 13.3, 13.4, 13.5, 13.6, 13.7, 13.8**

- [ ] 13. Error handling & recovery
  - [ ] 13.1 Wrap all invoke() calls in try-catch
  - [ ] 13.2 Preserve previous state on error
  - [ ] 13.3 Display error toasts with Russian messages
  - [ ] 13.4 Show retry buttons on failures
  - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

- [ ]* 13.5 Write property tests for error state recovery
  - **Property 4: Error State Recovery**
  - **Validates: Requirements 11.1, 11.5**

- [ ] 14. Performance optimization
  - [ ] 14.1 Image lazy loading - loading="lazy" on all posters
  - [ ] 14.2 Search debouncing - 300ms delay
  - [ ] 14.3 Download polling - 5 second interval (not 2)
  - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5_

- [ ] 15. Accessibility
  - [ ] 15.1 ARIA labels - All buttons, inputs, modals
  - [ ] 15.2 Focus management - Focus trap in modals, restore on close
  - [ ] 15.3 Keyboard navigation - Tab, Arrow keys, Escape
  - [ ] 15.4 Color contrast - Verify 4.5:1 ratio
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5, 10.6_

---

## Phase 8: Cleanup & Final Testing

- [ ] 16. Consolidate and delete old components
  - [ ] 16.1 Identify which of 23 components can be deleted or consolidated
  - [ ] 16.2 Delete unused components (MovieTile, SkeletonTile, LoadingDots, etc.)
  - [ ] 16.3 Consolidate overlapping functionality
  - _Requirements: All_

- [ ] 17. Final integration & testing
  - [ ] 17.1 Run full test suite
  - [ ] 17.2 Verify all Tauri commands work end-to-end
  - [ ] 17.3 Test responsive design at all breakpoints (mobile, tablet, desktop)
  - [ ] 17.4 Manual testing - All pages, all actions, all error states
  - [ ] 17.5 Verify zero console errors
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

- [x] All Tauri commands wired and tested
- [ ] All 8 core components built and working
- [ ] All 5 stores managing state correctly
- [ ] All 4 pages fully functional
- [ ] Full Russian localization
- [ ] Responsive design (1/2-3/4-6 columns)
- [ ] Keyboard navigation working
- [ ] ARIA labels on all interactive elements
- [ ] Error handling with retry
- [ ] Old components consolidated/deleted
- [ ] Zero console errors
- [ ] All property tests passing
