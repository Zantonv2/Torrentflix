---
inclusion: always
---

# Product Overview

TorrentFlix (MovieDownloader) is a desktop application for discovering, downloading, and managing movie torrents with a Netflix-style interface.

## Core Features

- **Multi-indexer search**: Aggregates results from multiple torrent indexers (YTS, Monna) with parallel fetching
- **Smart deduplication**: Groups releases by identity (title, year, season/episode) and quality signature
- **Metadata enrichment**: Fetches posters, descriptions, cast, runtime, and genres from TMDB
- **Rating aggregation**: Displays ratings from Kinopoisk, IMDb, and TMDB with background fetching
- **Quality scoring**: Ranks torrents by resolution, codec, source, seeders, and file size
- **Netflix-style UI**: Modern Svelte interface with movie grids, detail panels, and skeleton loading states

## Architecture

Pure Rust backend (Tokio async) with embedded SQLite database, wrapped in a Tauri desktop app with Svelte/TypeScript frontend. All heavy lifting (parsing, scraping, metadata, deduplication, scoring) happens in the Rust engine.

## Current State

Active development focused on rating system integration and UI polish. Core search, metadata, and indexer functionality is implemented. Download management and library features are planned.

## Design Principles

- **Separation of concerns**: Business logic in Rust engine, UI concerns in Svelte, thin command bridge in Tauri
- **Async-first**: All I/O operations are non-blocking using Tokio
- **Data-driven UI**: UI components receive data via props, no global state except stores
- **Fail gracefully**: Missing metadata or ratings don't block search results; they load asynchronously
- **User-centric**: Prioritize responsiveness and visual feedback (loading states, skeleton screens)

## Key Workflows

### Search & Discovery
1. User enters search query in UI
2. Tauri command triggers Engine search
3. Engine queries multiple indexers in parallel
4. Results are deduplicated and scored
5. Metadata is fetched asynchronously from TMDB
6. Ratings are fetched in background (don't block results)
7. UI displays results with progressive enhancement

### Download Management
1. User selects torrent from results
2. Tauri command sends to qBittorrent client
3. Job system tracks download progress
4. UI updates via polling or event system
5. Completed downloads are organized in library

### Rating System
1. Ratings are fetched asynchronously in background
2. Three-worker architecture (Kinopoisk, IMDb, TMDB)
3. Results are cached in-memory and persisted to database
4. UI displays available ratings, gracefully handles missing data

## Code Organization Principles

- **Engine is the source of truth**: All business logic, validation, and state management
- **Tauri is a bridge**: Minimal logic, just command routing and event handling
- **UI is presentation**: Components are stateless where possible, reactive declarations for computed values
- **Database is persistent**: SQLite with WAL mode for concurrent access
- **External APIs are abstracted**: Traits for indexers, metadata providers, rating sources

## Development Priorities

1. **Stability**: Core search and metadata must be reliable
2. **Performance**: Parallel indexing and async operations prevent UI blocking
3. **User experience**: Loading states, error handling, and graceful degradation
4. **Extensibility**: New indexers and rating sources should be easy to add via trait implementations
