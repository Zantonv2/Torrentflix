# Implementation Plan: TorrentFlix UI Redesign

## Phase 0: Tech Stack Upgrades

- [ ] 0. Upgrade dependencies
  - [ ] 0.1 Upgrade Vite 4.4 → 7.x - Update package.json, test build
  - [ ] 0.2 Upgrade Svelte 4 → 5 - Update package.json, review runes migration
  - [ ] 0.3 Upgrade Tailwind 3 → 4 - Update package.json, review config changes
  - [ ] 0.4 Upgrade TypeScript 5.0 → 5.5+ - Update package.json, fix type errors
  - [ ] 0.5 Update @tauri-apps/api to latest - Update package.json
  - [ ] 0.6 Run npm audit fix - Resolve security issues
  - [ ] 0.7 Verify app builds and runs - Test basic functionality

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

- [ ] 2. Create Svelte stores
  - [ ] 2.1 searchStore - query, results, loading, error
  - [ ] 2.2 libraryStore - items, filters, selectedItem, collections
  - [ ] 2.3 downloadStore - active downloads, polling (5s interval)
  - [ ] 2.4 settingsStore - data, dirty state, validation errors
  - [ ] 2.5 uiStore - currentPage, modal, toast, sidebarCollapsed

---

## Phase 2: Layout & Navigation

- [ ] 3. Create layout components
  - [ ] 3.1 MainLayout.svelte - Flex layout, Header + Sidebar + Content
  - [ ] 3.2 Header.svelte - Logo, search input (debounce 300ms), search button
  - [ ] 3.3 Sidebar.svelte - 4 nav items (Открыть, Библиотека, Загрузки, Настройки)

- [ ] 4. Implement navigation & routing
  - [ ] 4.1 Update App.svelte - Use MainLayout, route to pages via uiStore
  - [ ] 4.2 Add keyboard navigation - Tab, Arrow keys, Escape
  - [ ] 4.3 Add ARIA labels - All interactive elements
  - [ ] 4.4 Responsive design - Mobile hamburger, tablet 2-3 cols, desktop 4-6 cols

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

---

## Phase 4: Library Page

- [ ] 6. Build Library Browser tab
  - [ ] 6.1 On mount: Call `invoke('query_library', {filters: {}})`
  - [ ] 6.2 Search input - "Поиск в библиотеке...", on input: Call `invoke('search_library', {query, options: {fuzzy: true}})`
  - [ ] 6.3 Filters - Year, Resolution, Rating (slider), Watch Status
  - [ ] 6.4 Grid/List toggle - Switch between grid and list view
  - [ ] 6.5 Click item - Call `invoke('get_media_item', {id})` + `invoke('get_file_versions', {media_id})`
  - [ ] 6.6 Detail modal - Show metadata, ratings, watch status, user rating, tags, notes

- [ ] 7. Build Library Collections tab
  - [ ] 7.1 List collections (left panel, 300px) - Call `invoke('get_collection_items', {collection_id})` on click
  - [ ] 7.2 "+ New" button - Modal to create collection: Call `invoke('create_collection', {name, description})`
  - [ ] 7.3 "🧠 Smart" button - Modal for smart collection: Call `invoke('create_smart_collection', {name, criteria})`
  - [ ] 7.4 Add items to collection - Call `invoke('add_to_collection', {collection_id, media_ids})`

- [ ] 8. Build Library Analytics tab
  - [ ] 8.1 On mount: Call `invoke('get_storage_analytics')`
  - [ ] 8.2 Display stats - Total size, media count, file version count, avg versions
  - [ ] 8.3 Breakdown charts - By resolution, quality, status
  - [ ] 8.4 Largest items - Top 10 list
  - [ ] 8.5 Duplicate space warning - Yellow box if duplicates exist

- [ ] 9. Build Library Cleanup tab
  - [ ] 9.1 On mount: Call `invoke('get_cleanup_candidates')`
  - [ ] 9.2 List candidates - File name, size, reason
  - [ ] 9.3 Checkbox selection - Select candidates to delete
  - [ ] 9.4 Confirmation modal - "Вы уверены? Это удалит X файлов (Y GB)"
  - [ ] 9.5 Execute cleanup - Call `invoke('execute_cleanup', {candidate_ids})`

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

---

## Phase 7: Localization & Polish

- [ ] 12. Add Russian localization
  - [ ] 12.1 Install svelte-i18n
  - [ ] 12.2 Create i18n config - Russian default, English fallback
  - [ ] 12.3 Create translation files - ru.json, en.json
  - [ ] 12.4 Replace all hardcoded strings with $t() calls

- [ ] 13. Error handling & recovery
  - [ ] 13.1 Wrap all invoke() calls in try-catch
  - [ ] 13.2 Preserve previous state on error
  - [ ] 13.3 Display error toasts with Russian messages
  - [ ] 13.4 Show retry buttons on failures

- [ ] 14. Performance optimization
  - [ ] 14.1 Image lazy loading - loading="lazy" on all posters
  - [ ] 14.2 Search debouncing - 300ms delay
  - [ ] 14.3 Download polling - 5 second interval (not 2)

- [ ] 15. Accessibility
  - [ ] 15.1 ARIA labels - All buttons, inputs, modals
  - [ ] 15.2 Focus management - Focus trap in modals, restore on close
  - [ ] 15.3 Keyboard navigation - Tab, Arrow keys, Escape
  - [ ] 15.4 Color contrast - Verify 4.5:1 ratio

- [ ] 16. Cleanup & final testing
  - [ ] 16.1 Delete 23 old components
  - [ ] 16.2 Run full test suite
  - [ ] 16.3 Verify all Tauri commands work
  - [ ] 16.4 Test responsive design at all breakpoints
  - [ ] 16.5 Manual testing - All pages, all actions, all error states

---

## Tauri Commands Reference

**Discover**: `get_feed()`, `search_movies(query, media_type, limit)`, `start_download(magnetLink, title)`

**Library**: `query_library(filters)`, `search_library(query, options)`, `get_media_item(id)`, `get_file_versions(media_id)`, `create_collection(name, description)`, `create_smart_collection(name, criteria)`, `add_to_collection(collection_id, media_ids)`, `get_collection_items(collection_id)`, `get_storage_analytics()`, `get_cleanup_candidates()`, `execute_cleanup(candidate_ids)`

**Downloads**: `get_active_downloads()`, `pause_download(hash)`, `resume_download(hash)`, `delete_download(hash, delete_files)`

**Settings**: `get_settings()`, `save_settings(settings)`, `test_qbittorrent(url, username, password)`, `pick_folder()`

---

## Success Criteria

- [ ] All 15 components built and working
- [ ] All 5 stores managing state correctly
- [ ] All 4 pages fully functional
- [ ] All 26+ Tauri commands wired and tested
- [ ] Full Russian localization
- [ ] Responsive design (1/2-3/4-6 columns)
- [ ] Keyboard navigation working
- [ ] ARIA labels on all interactive elements
- [ ] Error handling with retry
- [ ] 23 old components deleted
- [ ] Zero console errors
