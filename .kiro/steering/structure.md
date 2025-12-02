# Project Structure

## Root Layout

```
Cargo.toml              # Workspace root with shared dependencies
package.json            # Root npm scripts for dev/build
.data/                  # SQLite database files (gitignored)
docs/                   # Architecture and implementation docs
```

## Engine (`engine/`)

Core Rust library with all business logic:

```
engine/src/
├── config/             # Settings loader, migrations
├── database/           # SQLite connection, queries (library, jobs, ratings)
├── dedupe/             # Deduplication rules and grouping logic
├── filesystem/         # Atomic file operations, hashing, renaming
├── indexers/           # Torrent indexer adapters (YTS, Monna)
│   ├── manager.rs      # Parallel indexer orchestration
│   ├── registry.rs     # Auto-registration system
│   └── traits.rs       # Indexer trait definition
├── jobs/               # Background job system (scheduler, workers, queue)
├── library/            # Library models and database queries
├── metadata/           # TMDB client, metadata cache
├── models/             # Domain models (TorrentResult, ParsedMedia, EnrichedMedia)
├── normalize/          # Release name parsing (guessit-rs port)
│   └── title_normalizer/ # Title normalization for API queries
├── ratings/            # Rating aggregation system
│   ├── manager.rs      # RatingManager with 3-worker architecture
│   ├── kinopoisk.rs    # Kinopoisk API client
│   ├── imdb.rs         # IMDb scraper
│   └── cache.rs        # In-memory rating cache
├── scoring/            # Quality scoring and ranking
├── torrent/            # qBittorrent client (planned)
├── ui/                 # UI DTOs and command helpers
└── engine.rs           # Main Engine orchestrator
```

## Tauri App (`src-tauri/`)

Thin wrapper exposing engine to frontend:

```
src-tauri/src/
├── commands.rs         # Tauri command handlers (search, feed, status)
└── main.rs             # App initialization, event system
```

## UI (`ui/`)

Svelte frontend with Netflix-style components:

```
ui/src/
├── components/
│   ├── MovieGrid.svelte      # Grid layout with infinite scroll
│   ├── MovieCard.svelte      # Card with hover overlay + detail modal
│   ├── MovieTile.svelte      # Tile variant
│   ├── DetailPanel.svelte    # Full movie details modal
│   ├── SearchBar.svelte      # Search input
│   ├── TopBar.svelte         # App header
│   ├── TabBar.svelte         # Navigation tabs
│   ├── SkeletonTile.svelte   # Loading placeholder
│   └── LoadingDots.svelte    # Loading indicator
├── components_inspiration/   # Reference implementations
├── types.ts            # TypeScript interfaces
├── App.svelte          # Root component
└── main.ts             # App entry point
```

## Documentation (`docs/`)

- `AGENT_INSTRUCTIONS.md`: Current work context and next steps
- `ROADMAP.md`: Complete architecture and implementation status
- `DATABASE.md`: Schema, maintenance, job queue design
- `JOB_MANAGER.md`: Background job system architecture
- `RATING_MANAGER.md`: Rating aggregation system details
- `UI.md`: UI component specifications

## Conventions

- **Rust modules**: One feature per module, public API via `mod.rs`
- **Error handling**: Use `anyhow::Result` for application errors, `thiserror` for library errors
- **Async**: All I/O is async with Tokio, use `Arc<Mutex<T>>` for shared state
- **Database**: SQLite with WAL mode, foreign keys enabled, use transactions for multi-step operations
- **UI components**: Self-contained Svelte components with TypeScript, TailwindCSS for styling
- **Naming**: Snake_case for Rust, camelCase for TypeScript/Svelte
