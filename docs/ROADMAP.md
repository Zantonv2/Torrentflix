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
5. ✅ **Implement YTS indexer** as a template adapter; test end‑to‑end search → normalization.
6. ✅ **Implement MonnaIndexer**: Complete indexer with dynamic torrent extraction, metadata parsing, poster URL extraction, and full Netflix-style integration.
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

## Recent work (Nov 2025) — completed & in progress

### ✅ Completed: Rating Manager System

- **Rating Manager Implementation** (`engine/src/ratings/manager.rs`)
  - ✅ Kinopoisk API client for fetching ratings from `kinopoiskapiunofficial.tech`
  - ✅ TMDB client integration using `tmdb_client` library (v1.8.0)
  - ✅ IMDb scraper that uses `imdb_id` from TMDB to fetch ratings
  - ✅ Title normalization for API requests (`normalize_for_rating()`)
    - Extracts year, season, series status from raw titles
    - Cleans titles for API queries (handles "сериал" prefix, case normalization)
  - ✅ In-memory caching system for ratings and metadata
  - ✅ 3-worker architecture:
    - Kinopoisk Worker (independent)
    - TMDB Worker 1 (metadata: episodes, seasons, external_id)
    - TMDB Worker 2 (IMDb scraper, depends on Worker 1)
  - ✅ Engine integration: automatically fetches missing ratings after metadata enrichment
  - ✅ Cache-aware worker spawning to minimize API calls

- **Job Manager Architecture Documentation**
  - ✅ Complete architecture spec (JOB_MANAGER.md)
  - ✅ Database schema for job queue (DATABASE.md)
  - ✅ Migration path from immediate execution to persistent queue
  - ✅ Worker manager pattern documentation

- **Dependencies & Upgrades**
  - ✅ Upgraded `reqwest` to 0.12
  - ✅ Upgraded `scraper` to 0.24
  - ✅ Upgraded `sqlx` to 0.8
  - ✅ Upgraded `thiserror` to 2.0
  - ✅ Added `tmdb_client` library (git dependency)

### ⏳ In Progress / TODO

- **UI Rating Integration** (High Priority)
  - ⏳ Trigger rating fetch on page load
  - ⏳ Show loading states while fetching ratings
  - ⏳ Update UI with Kinopoisk/IMDb ratings in MovieTile and MovieCard
  - ⏳ Handle missing ratings gracefully (show "N/A" in yellow)

- **Rating Manager Enhancements** (High Priority)
  - ⏳ Retry logic with exponential backoff for failed API calls
  - ⏳ Rate limiting for API calls (especially Kinopoisk and TMDB)
  - ⏳ Detailed error logging and metrics tracking
  - ⏳ Persistent cache (move from in-memory to database-backed)

- **Unified JobManager** (Medium Priority)
  - ⏳ Create `JobManager` structure wrapping `RatingManager`
  - ⏳ Define `WorkerManager` trait for future managers
  - ⏳ Prepare architecture for other worker managers

- **Persistent Job Queue** (Future)
  - ⏳ Implement database-backed job queue
  - ⏳ Job scheduler with dependencies
  - ⏳ Retry mechanism with backoff
  - ⏳ Job history and observability

- **Other Worker Managers** (Future)
  - ⏳ DownloadManager - Torrent download coordination
  - ⏳ MetadataRefreshManager - Periodic metadata updates
  - ⏳ LibrarySyncManager - Library scanning and synchronization
  - ⏳ FilesystemManager - File operations (moves, renames, cleanup)

### Previous Work (Still Valid)

- **Monna indexer**
  - ✅ Implemented DOM‑based genre parsing with regex fallback
  - ✅ Genres propagated through `TorrentResult` and `EnrichedMedia`
  - ✅ Case normalization for genres and cast (title case)

- **UI / Movie cards**
  - ✅ Netflix-style UI components (MovieGrid, MovieTile, DetailPanel)
  - ✅ Genres displayed as colored pill tags
  - ⏳ Wire real ratings from RatingManager (in progress)


---