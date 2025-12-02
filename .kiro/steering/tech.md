# Tech Stack

## Backend (Rust)

- **Runtime**: Tokio async with workspace-level dependency management
- **Database**: SQLite with sqlx (WAL mode, foreign keys enabled)
- **HTTP**: reqwest 0.12 with native-tls and SOCKS5 proxy support
- **Parsing**: scraper 0.24 for HTML, custom guessit-rs port for release name parsing
- **Serialization**: serde + serde_json
- **Error handling**: anyhow + thiserror 2.0
- **Logging**: tracing + tracing-subscriber

## Frontend (Svelte + TypeScript)

- **Framework**: Svelte 4 with Vite 4 build system
- **Styling**: TailwindCSS 3 with PostCSS
- **Desktop**: Tauri 2.0 for native app wrapper
- **API**: @tauri-apps/api for Rust backend communication

## Project Structure

Cargo workspace with three members:
- `engine/`: Core business logic (indexers, metadata, ratings, deduplication, scoring)
- `src-tauri/`: Tauri app wrapper with thin command layer
- `ui/`: Svelte frontend (separate npm workspace)

## Common Commands

### Development
```bash
npm run dev                 # Start UI dev server + Tauri dev mode
npm run ui:dev              # UI only (Vite dev server on :5173)
npm run tauri:dev           # Tauri only (requires UI running)
```

### Building
```bash
npm run build               # Build UI + Tauri production bundle
npm run ui:build            # Build UI only
npm run tauri:build         # Build Tauri app only
```

### Rust
```bash
cargo build                 # Build all workspace members
cargo test                  # Run tests
cargo check                 # Fast compile check
```

### UI Development
```bash
cd ui
npm run check               # Svelte type checking
npm run lint                # ESLint
npm run format              # Prettier formatting
```

## Environment Configuration

- `.env` files: Place in project root, `engine/src/`, or `src-tauri/`
- Required vars: `ADDRESS_PORT` (SOCKS5 proxy), API keys for TMDB/Kinopoisk
- Proxy config: Set per-client in Rust code, not via global env vars

## Dependencies

All Rust dependencies use workspace-level version management in root `Cargo.toml`. Key external integrations:
- TMDB API via `tmdb_client` (git dependency)
- Kinopoisk API (unofficial)
- IMDb scraping (no official API)
