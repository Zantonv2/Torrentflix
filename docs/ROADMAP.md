# MovieDownloader — Architecture

*Last updated: Dec 2025*

This document describes the **complete, replicable architecture** of MovieDownloader: a Tauri + Svelte UI with a pure‑Rust backend. It explains responsibilities, data flows, and step‑by‑step behavior for every component so a developer can reproduce or extend the system.

---

## Goals

* Single, consistent domain model for torrent results and media metadata.
* Pure Rust backend: parsing, IO, indexers, deduplication, metadata enrichment, downloads, and file management.
* Tiny, responsive Tauri UI (Svelte) that only presents data and issues commands.
* Easy extensibility: add indexers by implementing a trait/adapter; add post‑processing hooks.
* Safe file operations (atomic moves / crash‑resistance) and robust background scheduling.

---

## High-level pipeline

User → UI → Tauri Command → Engine → Indexers (parallel) → Normalization (guessit‑rs) → Metadata enrichment (TMDb/KP/IMDb) → Deduplication → Scoring → Sorting → UI presentation

Download: UI → Engine → Torrent Client → Download Tracking → Filesystem Manager → Library DB → Postprocessing (rename, tags) → Notifications

Background jobs: watchers and schedulers manage periodic rescans, metadata refreshes, and watchlist checking.

---

## Stack & runtime

* **Frontend:** Svelte (within Tauri) — purely presentation and user interactions.
* **Backend / Engine:** Rust (Tokio async runtime for concurrency). All heavy lifting lives here.
* **Storage:** SQLite (via `sqlx` or `rusqlite`) for library, downloads, watchlist, history. Cache images/metadata to disk.
* **Network / HTTP:** `reqwest` / `httpx` equivalent in Rust; JSON deserialization via `serde`.
* **Parsing:** `guessit‑rs` (your port) for filename/release parsing and normalization.
* **Indexer scraping:** `scraper` or equivalent HTML parser in Rust for HTML indexers; JSON parsing for API-based indexers.
* **Torrent control:** qBittorrent WebAPI client implemented in Rust.
* **Concurrency primitives:** channels (tokio::sync), Arc + Mutex/RwLock as needed, worker pools for CPU‑bound tasks.

---

## Module structure (conceptual)

```
src-tauri/
  ├── commands/           # Tauri command handlers (thin)
  ├── engine/             # Core engine crate
  │   ├── indexers/       # adapter modules (auto-registered)
  │   ├── normalize/      # guessit-rs integration + rating title normalization
  │   ├── metadata/       # TMDb client + cache (Kinopoisk/IMDb moved to ratings/)
  │   ├── ratings/        # ✅ Rating Manager + Kinopoisk/IMDb workers + cache
  │   │   ├── manager.rs  # RatingManager (3 workers: Kinopoisk, TMDB metadata, IMDb scraper)
  │   │   ├── kinopoisk.rs # Kinopoisk API client
  │   │   ├── imdb.rs     # IMDb scraper
  │   │   ├── models.rs   # Rating data structures
  │   │   └── cache.rs    # In-memory rating cache
  │   ├── dedupe/         # deduplication rules & grouping
  │   ├── scoring/        # scoring model & weights
  │   ├── torrent/        # torrent actions & status polling
  │   ├── filesystem/     # atomic moves, renamer, hashing
  │   ├── library/        # DB access, library model
  │   ├── jobs/           # ⏳ scheduler & job queue (future: persistent queue)
  │   └── config/         # config loader
  └── main.rs

ui/
  ├── src/                # svelte components
  └── static/

docs/

```

---

## Component responsibilities (step‑by‑step)

### 1) UI (Svelte/Tauri)

* Presents search box, results grid, detail view, downloads, library.
* Calls Tauri commands for actions (`search`, `add_to_download`, `pause`, `rename`, `rescan`).
* Receives streaming updates (progress, notifications) via Tauri event system.
* Does not perform parsing, scraping or filesystem operations.

### 2) Tauri Command Layer

* Very thin bridge that validates inputs and calls into the Engine API.
* Translates Engine domain models into UI DTOs (smaller, UI friendly).
* Runs commands asynchronously and returns errors in stable shape.

### 3) Engine (Core)

* Central orchestrator. Receives high‑level commands and sequences work across modules.
* Exposes an idiomatic API (search, download, get_library, rescan, watchlist_add).
* Manages lifecycle of background tasks, worker pools and shared state.

### 4) Indexer Manager & Indexers

* `Indexer` trait defines a single contract: `search(query) -> Vec<TorrentResult>` (async).
* Each indexer module is a small adapter that knows how to call the site (headers, cookies, pagination) and parse HTML/JSON.
* Auto‑registration: indexers register themselves in a global registry at initialization (compile‑time macro or module init) so Engine can discover them without manual wiring.
* Manager runs indexers in parallel, collects results, and forwards raw `TorrentResult`s to the Normalization stage.

### 5) Normalization (guessit‑rs)

* Runs the guessit parsing on every raw title/filename to produce `ParsedMedia` (title, year, season, episode, resolution, source, codec, release group, audio, etc.).
* Normalizer produces canonical fields used in dedupe, metadata lookup, and renaming.
* This is deterministic and the single source of truth for identity matching.

### 6) Metadata Enrichment

* ✅ **TMDb Client**: Integrated `tmdb_client` library for movie/TV show search, details, and external IDs.
* ✅ **Rating Manager**: Implemented `RatingManager` with 3 workers:
  - Kinopoisk Worker: Fetches ratings from Kinopoisk API
  - TMDB Worker 1: Fetches metadata (episodes, seasons, external_id including imdb_id)
  - TMDB Worker 2: IMDb scraper (depends on Worker 1, uses imdb_id)
* ✅ **In-Memory Caching**: Cache-aware worker spawning to minimize API calls.
* ✅ **Title Normalization**: Specialized `normalize_for_rating()` function for API requests (extracts year, season, series status).
* ✅ **Engine Integration**: RatingManager automatically fetches missing ratings after metadata enrichment in all search methods.
* ⏳ **UI Integration**: Trigger rating fetch on page load, show loading states, update UI with ratings.
* ⏳ **Error Handling**: Retry logic with exponential backoff, rate limiting, detailed logging.
* Cache metadata responses in memory + disk‑backed cache to minimize network calls.
* Merge metadata with parsed results to produce `EnrichedMedia` records that contain posters, overview, runtime, cast, genres, external IDs, and ratings.

### 7) Deduplication

* Group results by identity: `key = (canonical_title, year, season?, episode?)`.
* Within group, group releases by release signature (resolution, codec, source, audio, size bucket). That yields distinct releases per group.
* For items that share infohash (magnet → infohash) treat as exact duplicate and collapse immediately.
* After grouping, emit the set of distinct candidate releases for scoring.

### 8) Scoring & Ranking

* Compute a weighted score per candidate using configurable `ScoreWeights` (resolution, codec, source, audio, seeders, file size proximity, recency).
* Sort releases by score; optionally keep top N alternatives to present as “other qualities”.

### 9) Presentation to UI

* Convert top results to UI DTOs (title, poster, overview, top qualities, best magnet link, seeds, size, score).
* Return to UI; UI displays the grid with actions per tile.

### 10) Download & Torrent Client

* On user action, engine instructs the qBittorrent client (or alternative adapter) to add a magnet or torrent file. Use a pool for concurrent control and a status poller.
* Download state is persisted (library DB) and emitted to UI for live progress.

### 11) Filesystem Manager

* When download completes, move files from client download dir to temporary staging area, run verification (size/hash), then perform an atomic move to final path using a transaction pattern: write‑to‑temp + rename.
* Apply renaming rules based on `ParsedMedia`.
* In case of failures, roll back or keep an orphaned queue and notify user.

### 12) Library DB

* Persist library items with keys: local path, tmdb/imdb/kp ids, parsed metadata, current quality, dates, and history.
* Support queries for UI filtering and for background jobs.

### 13) Job Scheduler & Background Watchers

* ✅ **RatingManager Architecture**: First worker manager implementation with immediate execution pattern.
* ✅ **Job Manager Documentation**: Complete architecture spec (JOB_MANAGER.md) with migration path from immediate execution to persistent queue.
* ✅ **Database Schema**: Job queue schema defined in DATABASE.md (jobs table with retries, dependencies, scheduling).
* ⏳ **Unified JobManager**: Create `JobManager` structure to wrap `RatingManager` and future managers.
* ⏳ **Persistent Job Queue**: Database-backed job persistence with retries, dependencies, scheduling.
* ⏳ **Other Worker Managers**: DownloadManager, MetadataRefreshManager, LibrarySyncManager, FilesystemManager.
* Runs periodic tasks (rescan library, re‑fetch metadata, check watchlist), and handles retries/backoffs for failed external calls.
* Uses a persistent job queue for long‑running/important jobs so they survive restarts.

### 14) Notifications

* Translate engine events to user notifications (desktop + optional Telegram/Webhooks).
* Allow user preference for notification channels.

### 15) Config & Secrets

* Centralized config (toml/json) with user settings: indexers enabled, qBittorrent credentials, download paths, quality preferences, API keys.
* Provide a migration mechanism for config schema changes.

### 16) Logging & Diagnostics

* Use structured logging (e.g., `tracing` + `tracing_subscriber`), and keep an app log viewer in UI.
* Optionally support an opt‑in telemetry / error reporting channel.

---

## How to replicate (step-by-step)

1. ✅ **Bootstrap workspace**: create a Rust workspace with `engine` crate and `tauri` crate. Add `ui` Svelte project.
2. ✅ **Define domain models**: `TorrentResult`, `ParsedMedia`, `EnrichedMedia`, `DownloadStatus`, `LibraryItem`.
3. ✅ **Implement Indexer trait** and a simple static registry + manager that runs indexers concurrently.
4. ✅ **Port or include guessit-rs** and integrate normalizer pipeline (enhanced pattern-based normalizer with advanced regex patterns).
5. ✅ **Implement MonnaIndexer**: Complete indexer with dynamic torrent extraction, metadata parsing, poster URL extraction, and full Netflix-style integration.
7. ✅ **Add metadata clients**: Basic metadata structure with poster_url field support for UI integration.
8. ✅ **Implement Rating Manager**: 
   - Kinopoisk API client for rating fetching
   - TMDB client integration (using `tmdb_client` library)
   - IMDb scraper for ratings
   - Title normalization for API requests
   - In-memory caching system
   - Engine integration (automatic rating fetch after metadata enrichment)
9. ✅ **Job Manager Architecture**: Complete documentation (JOB_MANAGER.md) and database schema (DATABASE.md) for future persistent job queue.
10. 🔄 **Implement Netflix-style UI components**: Complete component library including MovieGrid, MovieTile, SkeletonTile, LoadingDots, TopBar, TabBar, DetailPanel with full accessibility compliance.
11. ⏳ **UI Rating Integration**: 
    - Trigger rating fetch on page load
    - Show loading states while fetching ratings
    - Update UI with Kinopoisk/IMDb ratings
    - Handle missing ratings gracefully
12. ⏳ **Rating Manager Enhancements**:
    - Retry logic with exponential backoff
    - Rate limiting for API calls
    - Detailed error logging and metrics
13. ⏳ **Unified JobManager**: Create `JobManager` structure wrapping `RatingManager` and preparing for other worker managers.
14. ⏳ **Add qBittorrent client adapter** and test add/pause/resume/status flows.
15. ⏳ **Implement filesystem manager** with staging + atomic move + renamer driven by `ParsedMedia`.
16. ⏳ **Wire library DB** (SQLite) and persist download results; expose library endpoints to UI via commands.
17. ⏳ **Persistent Job Queue**: Implement database-backed job queue with retries, dependencies, scheduling.
18. ⏳ **Other Worker Managers**: DownloadManager, MetadataRefreshManager, LibrarySyncManager, FilesystemManager.
19. ⏳ **Add notifications** and polish UI/UX.
20. ⏳ **Harden**: add retries, circuit breakers, rate limits, logging, and tests.

---

## Operational concerns & best practices

* Keep all network I/O async and non‑blocking using Tokio.
* Treat the UI as ephemeral; persist all important state server‑side (library DB) so restarts are safe.
* Cache aggressively but invalidate properly when metadata changes.
* Use transaction patterns for file moves and database writes.
* Respect legal and ethical boundaries for scraping third‑party sites; implement politeness (User‑Agent, rate limits).
* Write comprehensive unit tests for normalization and dedupe logic; use integration tests for indexers (mocked HTTP).

---

## Extensibility notes

* **Adding an indexer:** create new module, parse site, map to `TorrentResult`, register. No engine changes.
* **Plugins/hook points:** postdownload, prerenaming, prescan hooks allow custom behavior.
* **CLI / headless:** expose engine API via a CLI wrapper for headless operation and automation.

---

## 📊 Project Status (December 2, 2025)

**ACTUAL Completion**: ~55% | **Production Ready**: ❌ Not Yet (But Very Close!)

### ✅ What's Working (Completed Features)

#### Core Search & Discovery
- ✅ Multi-indexer search (Monna2) with parallel execution
- ✅ Advanced title normalization (pattern-based guessit-rs port)
- ✅ TMDB metadata enrichment (posters, descriptions, cast, genres, seasons/episodes)
- ✅ Deduplication and quality scoring
- ✅ Netflix-style UI with 5x2 grid layout
- ✅ Skeleton loading states and WCAG AA accessibility
- ✅ Feed loading on initial mount (Svelte 5 fix)

#### Rating System (Best-in-Class) ✅ **FULLY WORKING**
- ✅ Multi-source rating aggregation (Kinopoisk + IMDb + TMDB)
- ✅ 3-worker architecture with dependency management
- ✅ Dual-layer caching (in-memory + SQLite)
- ✅ Real-time UI updates via Tauri events (FIXED - channel now set before initialize)
- ✅ Graceful degradation (works without optional API keys)
- ✅ Cache-aware worker spawning
- ✅ All 3 ratings display correctly in UI (KP | IMDB | TMDB)
- ✅ TV show metadata (seasons/episodes) extracted from TMDB

#### Download Management ✅ **FULLY WORKING**
- ✅ qBittorrent WebAPI client (login, add, pause, resume, delete)
- ✅ Get download status (progress, speed, ETA, seeders, leechers)
- ✅ Get all active downloads
- ✅ Tauri commands registered
- ✅ Engine integration with environment config
- ✅ Downloads UI tab (DownloadsTab.svelte) - real-time progress updates
- ✅ Tested and working (see DOWNLOAD_IMPLEMENTATION_PROOF.md)

#### Infrastructure
- ✅ Tokio async runtime with workspace dependencies
- ✅ SQLite database with WAL mode
- ✅ Tauri 2.0 desktop wrapper
- ✅ Comprehensive documentation (5 spec documents)
- ✅ Job Manager architecture (immediate execution pattern)
- ✅ Database schema for job queue (defined, not yet used)

### 🚨 What's Actually Missing (Honest Assessment)

#### 1. Library Management ❌ **0% COMPLETE**
**Reality**: Database schema exists. That's it. No scanning, no tracking, no UI.

**What exists**: SQL schema in docs
**What's needed**: Everything else
- File scanner (doesn't exist)
- Database operations (doesn't exist)
- Watch history (doesn't exist)
- Library UI (doesn't exist)

**Effort**: 3-4 days of actual work

#### 2. Bookmarks (Wishlist) ❌ **0% COMPLETE**
**Purpose**: Bookmark movies that DON'T EXIST YET on indexers. Wait for them to appear with specific quality preferences.

**Features**:
- Bookmark movie with quality filters:
  - **Quality**: 480p, 720p, 1080p, 2160p (4K), etc.
  - **Source**: WEBRip, WEB-DL, BluRay, HDTV, etc.
  - **Audio**: DTS, AC3, Dolby Atmos, AAC, etc.
- Auto-notify when movie appears matching filters
- Parse quality/source/audio from torrent name (using normalizer)
- Show bookmark count badge on tab

**What's needed**:
- Bookmarks database table
- Bookmark CRUD operations
- Quality filter UI
- Indexer monitoring (check bookmarks periodically)
- Notifications when match found
- BookmarksTab.svelte component

**Effort**: 2-3 days

#### 3. History (Analytics) ❌ **0% COMPLETE**
**Purpose**: Download statistics and analytics. Show user preferences and download history.

**Features**:
- **Statistics Dashboard**:
  - Preferred quality (most downloaded: 1080p, 720p, etc.)
  - Preferred genres (Action, Drama, etc.)
  - Preferred years (2020s, 2010s, etc.)
  - Total GB downloaded
  - Download count by month/year
  - Charts and graphs
- **Download History**:
  - List of all downloaded movies
  - Show parsed metadata (quality, source, audio)
  - Show download date, size
  - Filter by quality/genre/year
  - Search history

**What's needed**:
- Download history database table
- Track downloads with parsed metadata
- Statistics aggregation queries
- Charts library (Chart.js or similar)
- HistoryTab.svelte component

**Effort**: 2-3 days

#### 4. Settings UI ❌ **0% COMPLETE**
**Reality**: No settings UI. No settings persistence. Users edit .env files like cavemen.

**What exists**: Nothing
**What's needed**: Everything
- Settings database table
- Settings CRUD operations
- SettingsTab.svelte component
- API key management (TMDB, Kinopoisk)
- qBittorrent connection config
- Library/download paths
- Indexer enable/disable

**Effort**: 1-2 days

#### 5. Job Queue Persistence ⏳ **10% COMPLETE**
**Reality**: JobManager exists but only wraps RatingManager. No persistent queue, no retries, no scheduling.

**What exists**: 
- Architecture docs (complete)
- Database schema (defined)
- RatingManager (immediate execution)

**What's needed**: 
- Persistent job queue implementation
- Job scheduler
- Retry logic with backoff
- Job dependencies
- Other worker managers (DownloadManager, MetadataRefreshManager, etc.)

**Effort**: 3-4 days of actual work

#### 6. Monna Indexer Metadata Parsing ⚠️ **NEEDS IMPROVEMENT**
**Reality**: Current regex patterns are too rigid. Fails on field label variations.

**Issues**:
- Missing "ГОД:" pattern (uppercase)
- Missing "СТРАНА:" pattern
- Missing "Студия:" extraction
- Missing "Перевод:" extraction
- Description extraction fails without explicit label

**Effort**: 2-3 hours to fix

**See**: NEXT_SESSION_CONTEXT.md for detailed requirements

### ⏳ Planned Features (Roadmap)

See **[GRAND_PLAN.md](./GRAND_PLAN.md)** for complete 4-month development plan.

**Phase 1** (Month 1): Critical features - Download, Library, Settings
**Phase 2** (Month 2): UX enhancements - Search, Shortcuts, Notifications
**Phase 3** (Month 3): Reliability - Job queue, Testing, Performance
**Phase 4** (Month 4): Advanced - Watchlist, Auto-download, Smart cleanup
**Phase 5** (Month 4): Polish - Statistics, Themes, Backup

### 📈 Recent Achievements (November 2025)

- ✅ Implemented complete rating aggregation system
- ✅ Added real-time rating updates to UI
- ✅ Created comprehensive documentation suite
- ✅ Upgraded all dependencies to latest versions
- ✅ Established job manager architecture

### 🎯 Next Immediate Steps

1. **This Week**: Downloads UI tab (2-3 hours) - **HIGHEST PRIORITY**
2. **Next Week**: Library management (file scanning, database, UI)
3. **Week 3**: Bookmarks system (wishlist with quality filters)
4. **Week 4**: History/Analytics tab + Settings UI

**Goal**: Ship functional v1.0 with all 6 tabs by end of Month 1

---

## 📑 Complete Tab Structure

### **1. 📚 Библиотека (Library)** ❌ **NOT STARTED**
**Purpose**: Browse and manage downloaded movies/shows

**Features**:
- Grid view of downloaded files (like Feed)
- Metadata, posters, ratings
- Filters: genre, year, quality, watched/unwatched
- Search within library
- Mark as watched, add to favorites
- Delete files
- Open in player

**Backend**: File scanner, library database, CRUD operations
**Frontend**: LibraryTab.svelte (similar to Feed)

---

### **2. 📋 История (History)** ❌ **NOT STARTED**
**Purpose**: Download analytics and statistics

**Features**:
- **Statistics Dashboard**:
  - Preferred quality chart (1080p: 45%, 720p: 30%, 4K: 25%)
  - Preferred genres pie chart
  - Downloads by year/month line chart
  - Total GB downloaded counter
  - Average file size
- **Download History List**:
  - All downloaded movies with metadata
  - Show parsed quality/source/audio from torrent name
  - Download date, size, seeders at time of download
  - Filter by quality/genre/year
  - Search history

**Backend**: 
- Download history table (movie_id, torrent_name, parsed_quality, parsed_source, parsed_audio, download_date, size_gb)
- Statistics aggregation queries
- Parse quality/source/audio using normalizer patterns

**Frontend**: 
- HistoryTab.svelte with charts (Chart.js)
- Statistics cards
- History list with filters

**Data Source**: Parse from torrent names using `engine/src/normalize/patterns.rs` and `title_normalizer/`

---

### **3. 🎬 Лента (Feed)** ✅ **WORKING**
**Purpose**: Browse recent movies from indexers

**Status**: Fully functional, shows movies from Monna2

---

### **4. ⭐ Закладки (Bookmarks)** ❌ **NOT STARTED**
**Purpose**: Wishlist for movies that don't exist yet on indexers

**Features**:
- Bookmark movie with quality preferences:
  - **Quality**: 480p, 720p, 1080p, 2160p (4K), etc.
  - **Source**: WEBRip, WEB-DL, BluRay, HDTV, BRRip, etc.
  - **Audio**: DTS, AC3, Dolby Atmos, AAC, MP3, etc.
- Auto-check indexers periodically (every 6-24 hours)
- Notify when movie appears matching filters
- Show match quality score (how well it matches preferences)
- One-click download when match found
- Badge on tab showing bookmark count

**Backend**:
- Bookmarks table (movie_id, title, year, quality_filter, source_filter, audio_filter, created_at, last_checked)
- Periodic job to check indexers for bookmarked movies
- Parse quality/source/audio from torrent names using normalizer
- Match against user preferences
- Notification system

**Frontend**:
- BookmarksTab.svelte
- Add bookmark button in movie detail panel
- Quality filter dropdowns (multi-select)
- Bookmark list with match status
- Notification when match found

**Data Source**: Parse from torrent names using `engine/src/normalize/patterns.rs` and `title_normalizer/`

**Example**:
```
User bookmarks "Dune: Part Three (2026)"
Preferences: Quality >= 1080p, Source = BluRay OR WEB-DL, Audio = DTS OR Dolby
System checks indexers every 12 hours
When "Dune.Part.Three.2026.1080p.BluRay.DTS.x264-GROUP" appears:
  → Notify user: "Dune: Part Three is now available! (1080p BluRay DTS)"
  → Show in bookmarks with "✅ Match Found" badge
  → One-click download
```

---

### **5. 📥 Загрузки (Downloads)** ⏳ **BACKEND DONE, UI PENDING**
**Purpose**: Monitor active downloads from qBittorrent

**Features**:
- List of active torrents
- Real-time progress bars
- Speed, ETA, seeders/leechers
- Pause/resume/cancel buttons
- Open in qBittorrent button
- Auto-refresh every 2 seconds

**Backend**: ✅ Complete (qBittorrent client working)
**Frontend**: ❌ DownloadsTab.svelte (not built yet)

---

### **6. ⚙️ Настройки (Settings)** ❌ **NOT STARTED**
**Purpose**: Configure app settings

**Features**:
- **API Keys**: TMDB, Kinopoisk (masked input, test button)
- **qBittorrent**: URL, username, password (test connection)
- **Paths**: Library folder, download folder
- **Indexers**: Enable/disable, test connectivity
- **Quality Preferences**: Default quality for auto-download
- **Notifications**: Desktop notifications on/off
- **Advanced**: Cache size, log level, proxy settings

**Backend**: Settings table, CRUD operations
**Frontend**: SettingsTab.svelte with forms

---

**Goal**: Ship functional v1.0 with all 6 tabs by end of Month 1

---

## 📚 Documentation

**Essential Docs**:
- **[ACTION_PLAN.md](./ACTION_PLAN.md)** - What to build RIGHT NOW (3-week plan)
- **[RATING_MANAGER.md](./RATING_MANAGER.md)** - Rating system (actually works)
- **[DATABASE.md](./DATABASE.md)** - Database schema
- **[UI.md](./UI.md)** - UI components

**Architecture Docs** (for reference):
- **[JOB_MANAGER.md](./JOB_MANAGER.md)** - Job queue design (not implemented yet)

---