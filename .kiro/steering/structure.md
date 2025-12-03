---
inclusion: always
---

# Project Structure

## Root Layout

```text
Cargo.toml              # Workspace root with shared dependencies
package.json            # Root npm scripts for dev/build
.data/                  # SQLite database files (gitignored)
docs/                   # Architecture and implementation docs
```

## Engine (`engine/`)

Core Rust library with all business logic. When working on backend features, start here.

```text
engine/src/
├── config/             # Settings loader, migrations
├── database/           # SQLite connection, queries (library, jobs, ratings)
├── dedupe/             # Deduplication rules and grouping logic
├── filesystem/         # Atomic file operations, hashing, renaming
├── indexers/           # Torrent indexer adapters (Monna)
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

Thin wrapper exposing engine to frontend. Minimal logic here—keep it as a command bridge.

```text
src-tauri/src/
├── commands.rs         # Tauri command handlers (search, feed, status)
└── main.rs             # App initialization, event system
```

## UI (`ui/`)

Svelte frontend with Netflix-style components. When working on UI, start here.

```text
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

Reference these for architecture and design decisions:

- `ROADMAP.md`: Complete architecture and implementation status
- `DATABASE.md`: Schema, maintenance, job queue design
- `JOB_MANAGER.md`: Background job system architecture
- `RATING_MANAGER.md`: Rating aggregation system details
- `UI.md`: UI component specifications
- `ACTION_PLAN.md`: Current work context and next steps

## Code Organization Principles

### Rust Modules

- **One feature per module**: Each module has a single responsibility
- **Public API via `mod.rs`**: Re-export public types and functions
- **Private by default**: Only expose what's needed
- **Traits for abstraction**: Use traits for pluggable components (indexers, metadata providers)

### Error Handling

- **Application errors**: Use `anyhow::Result<T>` for functions that can fail
- **Library errors**: Use `thiserror` for custom error types with context
- **Error propagation**: Use `?` operator, avoid `.unwrap()` in library code

### Async & Concurrency

- **All I/O is async**: Use Tokio for all blocking operations
- **Shared state**: Use `Arc<Mutex<T>>` for thread-safe shared state
- **Channels**: Prefer channels over locks for producer-consumer patterns
- **Spawning tasks**: Use `tokio::spawn` for background work, not threads

### Database

- **WAL mode**: Enabled for concurrent access
- **Foreign keys**: Enabled for referential integrity
- **Transactions**: Use for multi-step operations to ensure atomicity
- **Queries**: Use sqlx with compile-time checked queries

### UI Components

- **Self-contained**: Each component manages its own state
- **Props over globals**: Pass data via component props, not global stores
- **Reactive declarations**: Use Svelte's `$:` for computed values
- **Styling**: TailwindCSS only, no inline styles
- **TypeScript strict**: No `any` types, full type safety

### Naming Conventions

- **Rust**: `snake_case` for functions, variables, modules; `PascalCase` for types, traits
- **TypeScript/Svelte**: `camelCase` for functions, variables; `PascalCase` for types, components
- **Database**: `snake_case` for tables and columns
- **Files**: Match the primary export (e.g., `MovieCard.svelte` exports `MovieCard`)

## Data Flow

1. **UI** (Svelte) → calls Tauri command
2. **Tauri** (src-tauri) → delegates to Engine
3. **Engine** (Rust) → orchestrates business logic
4. **Database** (SQLite) → persists state
5. **External APIs** (TMDB, Kinopoisk, IMDb) → fetches metadata/ratings
6. **Response** flows back through the same layers

Keep this flow in mind when adding features—logic belongs in the engine, not in Tauri or UI.
