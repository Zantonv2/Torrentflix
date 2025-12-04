# 🎬 TORRENTFLIX UI - REALITY CHECK ANALYSIS

**Status**: ACTUAL ANALYSIS (No Bullshit)  
**App Type**: LOCAL DESKTOP APP (Tauri + Rust)  
**Date**: December 4, 2025

---

## WHAT THIS APP ACTUALLY DOES

This is a **LOCAL TORRENT DISCOVERY & LIBRARY MANAGEMENT DESKTOP APP**. Not a web service. Not a streaming platform. Not a social app.

### Core Functions (What Actually Works):
1. **Search torrents** - Query Monna2 indexer for movies/series
2. **Fetch ratings** - Get ratings from Kinopoisk, IMDb, TMDB (async, background)
3. **Download management** - Start/pause/resume/delete downloads via qBittorrent
4. **Library management** - Track downloaded files, organize collections
5. **Settings** - Configure API keys, qBittorrent connection, paths
6. **Storage analytics** - Show disk usage, cleanup candidates

### What the UI ACTUALLY Calls:
```
✅ get_feed() - Get recent movies from Monna2
✅ search_movies() - Search for movies
✅ start_download() - Send magnet to qBittorrent
✅ get_active_downloads() - Poll download status
✅ pause_download() - Pause a download
✅ resume_download() - Resume a download
✅ delete_download() - Delete a download
✅ get_settings() - Load settings
✅ save_settings() - Save settings
✅ test_qbittorrent() - Test qBittorrent connection
✅ pick_folder() - Open folder picker
✅ set_user_rating() - Rate a movie
✅ set_watch_status() - Mark watched/unwatched
✅ set_custom_notes() - Add notes to movie
✅ set_preferred_version() - Mark preferred file version
```

### What the UI DOESN'T Call (But Exists in Backend):
```
❌ query_library() - Query library with filters
❌ search_library() - Search library by text
❌ get_media_item() - Get specific media item
❌ get_file_versions() - Get file versions
❌ stage_file() - Stage file for import
❌ commit_staged_file() - Commit staged file
❌ get_staged_files() - Get staged files
❌ schedule_rescan() - Schedule rescan
❌ get_cleanup_candidates() - Get cleanup candidates
❌ execute_cleanup() - Execute cleanup
❌ get_storage_analytics() - Get storage analytics
❌ create_collection() - Create collection
❌ add_to_collection() - Add to collection
❌ get_collection_items() - Get collection items
❌ create_smart_collection() - Create smart collection
❌ add_tags() - Add tags
```

**Translation**: The backend has TONS of functionality that the UI doesn't use yet.

---

## ACTUAL UI PROBLEMS (Not Assumptions)

### 1. NAVIGATION IS BROKEN
**Current**: 7 tabs in a horizontal bar
- Feed, Library, History, Bookmarks, Management, Downloads, Settings

**Reality**: 
- Most tabs show placeholder components or incomplete features
- "Management" tab has Cleanup + Jobs (should be nested)
- "History" is StorageAnalytics (confusing name)
- "Bookmarks" is CollectionManager (not implemented in backend)
- No way to navigate between related items

**Impact**: Users get lost, unclear what each tab does

### 2. COMPONENTS ARE MASSIVE & INCOMPLETE
- **SettingsTab.svelte**: 700+ lines, handles 50+ form fields
- **App.svelte**: 300+ lines, manages all state
- **ManagementPanel.svelte**: Wrapper for Cleanup + Jobs (why?)
- **LibraryBrowser.svelte**: Grid/list view but doesn't call backend
- **CollectionManager.svelte**: UI exists but backend commands not called
- **StorageAnalytics.svelte**: Calls `get_storage_analytics()` which doesn't exist in backend

**Impact**: Hard to maintain, hard to test, hard to add features

### 3. STATE MANAGEMENT IS NONEXISTENT
- All state in App.svelte
- Props drilling everywhere
- Rating updates via events (fragile)
- No persistence between sessions
- No error recovery

**Impact**: Data inconsistency, hard to debug

### 4. BACKEND/UI MISMATCH
**Backend has these commands but UI doesn't use them**:
- Library querying (query_library, search_library)
- File management (stage_file, commit_staged_file)
- Cleanup operations (get_cleanup_candidates, execute_cleanup)
- Collections (create_collection, add_to_collection)
- Storage analytics (get_storage_analytics)

**UI calls these but backend might not fully support**:
- set_preferred_version() - exists but unclear if working
- get_storage_analytics() - doesn't exist in commands.rs

**Impact**: Wasted backend work, incomplete UI features

### 5. RESPONSIVE DESIGN IS MISSING
- MovieGrid hardcoded to 5 columns
- No mobile breakpoints
- Sidebar takes up space on mobile
- Modals not optimized for small screens

**Impact**: Unusable on laptop/tablet

### 6. ACCESSIBILITY IS ZERO
- No ARIA labels
- No keyboard navigation
- No focus management
- No screen reader support

**Impact**: Unusable for people with disabilities

### 7. PERFORMANCE ISSUES
- JobsDashboard polls every 2 seconds (wasteful)
- No lazy loading for images
- No code splitting
- Rating updates block UI

**Impact**: High CPU usage, battery drain, slow UI

### 8. ERROR HANDLING IS WEAK
- Generic error messages
- No retry logic
- No error recovery
- No user feedback on failures

**Impact**: Users don't know what went wrong

### 9. DESIGN INCONSISTENCY
- Netflix theme mixed with generic gray boxes
- No design system
- Colors hardcoded everywhere
- Inconsistent button styles
- Inconsistent spacing

**Impact**: Looks unprofessional, hard to maintain

### 10. MISSING FEATURES
- No way to browse library (LibraryBrowser exists but doesn't work)
- No way to manage collections (UI exists but backend not called)
- No way to cleanup files (UI exists but backend not called)
- No way to view storage analytics (UI exists but backend not called)
- No way to import files
- No way to rescan library

**Impact**: Backend features are useless

---

## WHAT ACTUALLY WORKS WELL

1. **Search & Feed** - Works great, fast, good results
2. **Download Management** - Works well, good UI
3. **Settings** - Works, comprehensive
4. **Rating System** - Works, async background fetching
5. **Tauri Integration** - Solid, good command structure

---

## REAL PRIORITIES (Not Bullshit)

### Phase 1: Fix What's Broken (1-2 weeks)
1. **Fix backend/UI mismatch**
   - Remove unused UI components
   - Implement missing backend calls
   - Fix get_storage_analytics() command

2. **Fix navigation**
   - Reorganize tabs (Discover, Library, Downloads, Settings)
   - Remove placeholder tabs
   - Add breadcrumbs

3. **Fix state management**
   - Move state to Svelte stores
   - Implement proper error handling
   - Add retry logic

4. **Fix responsive design**
   - Make grid responsive
   - Add mobile navigation
   - Optimize modals

### Phase 2: Implement Missing Features (2-3 weeks)
1. **Library Browser** - Actually call backend
2. **Collections** - Actually call backend
3. **Cleanup** - Actually call backend
4. **Storage Analytics** - Actually call backend
5. **File Import** - Implement staging/commit

### Phase 3: Polish (1 week)
1. **Design System** - Consistent colors, spacing, typography
2. **Accessibility** - ARIA labels, keyboard nav, focus management
3. **Performance** - Optimize polling, lazy loading, code splitting
4. **Error Handling** - Better messages, retry logic, recovery

---

## TECH STACK REALITY

**Current**:
- Vite 4.4 (2023)
- Svelte 4 (2023)
- Tailwind 3 (2023)
- TypeScript 5.0

**Should Be**:
- Vite 7+ (current)
- Svelte 5 (current, has runes)
- Tailwind 4 (current, has container queries)
- TypeScript 5.5+ (current)

**Why Update**:
- Better performance
- Better DX
- Security fixes
- Modern features

**Effort**: 2-3 hours

---

## ACTUAL COMPONENT BREAKDOWN

### What Exists:
```
App.svelte (300+ lines) - TOO BIG
├── TopBar - Search bar only
├── TabBar - 7 tabs
├── MovieGrid - Hardcoded 5 columns
├── DetailPanel - Right sidebar
├── DownloadsTab - Download management
├── SettingsTab - 700+ lines, TOO BIG
├── LibraryBrowser - Grid/list view (doesn't call backend)
├── MediaItemDetail - Modal (doesn't call backend)
├── CollectionManager - UI only (doesn't call backend)
├── StorageAnalytics - Calls non-existent command
├── ManagementPanel - Wrapper for Cleanup + Jobs
├── JobsDashboard - Polls every 2 seconds
├── CleanupManager - UI only (doesn't call backend)
├── VersionManager - Quality scoring UI
├── MovieCard - Card component
├── MovieTile - Tile component
├── SkeletonTile - Loading placeholder
├── LoadingSpinner - Spinner
├── LoadingDots - Dots animation
├── SearchBar - Search input
├── SecureInput - Password input
├── SettingsSection - Settings group
└── SettingsNotification - Toast
```

### What Should Exist:
```
App.svelte (100 lines) - SMALL
├── MainLayout
│   ├── Header (Logo + Search)
│   ├── Sidebar (Navigation)
│   └── Content
│       ├── Discover (Search + Grid)
│       ├── Library (Browser + Collections)
│       ├── Downloads (Active + History)
│       └── Settings (Modular sections)
├── Modals
│   ├── MovieDetail
│   ├── SettingsModal
│   └── ConfirmDialog
└── Stores
    ├── searchStore
    ├── libraryStore
    ├── downloadStore
    ├── settingsStore
    └── uiStore
```

---

## REAL EFFORT ESTIMATE

| Task | Time | Impact |
|------|------|--------|
| Update dependencies | 2-3h | Medium |
| Fix backend/UI mismatch | 4-6h | High |
| Fix navigation | 4-6h | High |
| Fix state management | 6-8h | High |
| Implement missing features | 16-20h | High |
| Design system | 4-6h | Medium |
| Accessibility | 4-6h | Medium |
| Performance | 4-6h | Medium |
| **TOTAL** | **44-61h** | **Very High** |

**Timeline**: 2-3 weeks with 1 developer

---

## WHAT NOT TO DO

❌ Don't rewrite everything at once  
❌ Don't add new features before fixing existing ones  
❌ Don't assume what users want  
❌ Don't ignore the backend  
❌ Don't make it a web app  
❌ Don't add sharing/social features  
❌ Don't add cloud sync  
❌ Don't make it cross-platform (it's desktop only)  

---

## WHAT TO DO

✅ Fix the backend/UI mismatch first  
✅ Implement missing features that backend supports  
✅ Make it work well locally  
✅ Make it fast and responsive  
✅ Make it accessible  
✅ Make it maintainable  
✅ Document everything  
✅ Test thoroughly  

---

## NEXT STEPS

1. **This Week**: Audit all UI components, identify what's broken
2. **Next Week**: Fix backend/UI mismatch, implement missing calls
3. **Week 3**: Reorganize navigation, fix state management
4. **Week 4**: Implement missing features
5. **Week 5**: Polish, accessibility, performance

---

**This is a REAL app with REAL problems. Fix them systematically.**

