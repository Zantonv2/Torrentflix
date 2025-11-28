# MovieDownloader — Architecture

*Last updated: Nov 2025*

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
  │   ├── normalize/      # guessit-rs integration
  │   ├── metadata/       # TMDb, Kinopoisk, IMDb clients + cache
  │   ├── dedupe/         # deduplication rules & grouping
  │   ├── scoring/        # scoring model & weights
  │   ├── torrent/        # torrent actions & status polling
  │   ├── filesystem/     # atomic moves, renamer, hashing
  │   ├── library/        # DB access, library model
  │   ├── jobs/           # scheduler & job queue
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

* Using parsed identity, call TMDb (primary), then Kinopoisk / IMDb for ratings where available.
* Cache metadata responses in memory + disk‑backed cache to minimize network calls.
* Merge metadata with parsed results to produce `EnrichedMedia` records that contain posters, overview, runtime, cast, genres, and external IDs.

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
5. ✅ **Implement one indexer** (YTS) as a template adapter; test end‑to‑end search → normalization.
6. 🔄 **Add metadata clients**: TMDb wrapper + caching layer.
7. ⏳ **Implement dedupe + scoring** modules and a basic UI DTO flow.
8. ⏳ **Add qBittorrent client adapter** and test add/pause/resume/status flows.
9. ⏳ **Implement filesystem manager** with staging + atomic move + renamer driven by `ParsedMedia`.
10. ⏳ **Wire library DB** (SQLite) and persist download results; expose library endpoints to UI via commands.
11. ⏳ **Add job scheduler** and background watchers (rescan, watchlist scanning).
12. ⏳ **Add notifications** and polish UI/UX.
13. ⏳ **Harden**: add retries, circuit breakers, rate limits, logging, and tests.

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