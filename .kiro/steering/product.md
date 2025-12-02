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
