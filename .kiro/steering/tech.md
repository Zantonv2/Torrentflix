---
inclusion: always
---

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

## Rust Code Patterns

### Async & Concurrency

- All I/O operations must be async using Tokio (no blocking calls in async contexts)
- Use `tokio::spawn` for background tasks, not threads
- Shared state: Use `Arc<Mutex<T>>` for thread-safe access
- Prefer channels over locks for producer-consumer patterns
- Never use `.unwrap()` in library code; use `?` operator for error propagation

### Error Handling

- Application errors: Use `anyhow::Result<T>` for functions that can fail
- Library errors: Use `thiserror` for custom error types with context
- Always provide meaningful error messages for debugging
- Avoid panics in production code; return errors instead

### Database Operations

- WAL mode and foreign keys are enabled by default
- Use sqlx with compile-time checked queries (not string interpolation)
- Wrap multi-step operations in transactions for atomicity
- Always use parameterized queries to prevent SQL injection

### Module Organization

- One feature per module with a single responsibility
- Re-export public types and functions via `mod.rs`
- Keep modules private by default; only expose what's needed
- Use traits for pluggable components (indexers, metadata providers, rating sources)

## TypeScript/Svelte Code Patterns

### Type Safety

- No `any` types; use strict TypeScript throughout
- Define interfaces for all data structures
- Use discriminated unions for variant types
- Leverage TypeScript's type system for compile-time safety

### Component Structure

- Self-contained components that manage their own state
- Pass data via props, not global stores (except for app-wide state)
- Use Svelte's reactive declarations (`$:`) for computed values
- Keep components focused on a single responsibility

### Styling

- TailwindCSS only; no inline styles or CSS-in-JS
- Use utility classes for responsive design
- Define custom colors/spacing in `tailwind.config.js` if needed
- Avoid component-scoped styles unless absolutely necessary

## Naming Conventions

- **Rust**: `snake_case` for functions, variables, modules; `PascalCase` for types, traits
- **TypeScript/Svelte**: `camelCase` for functions, variables; `PascalCase` for types, components
- **Database**: `snake_case` for tables and columns
- **Files**: Match the primary export (e.g., `MovieCard.svelte` exports `MovieCard` component)

## Data Flow Architecture

1. **UI** (Svelte) → calls Tauri command
2. **Tauri** (src-tauri) → delegates to Engine (minimal logic here)
3. **Engine** (Rust) → orchestrates business logic and external APIs
4. **Database** (SQLite) → persists state
5. **External APIs** (TMDB, Kinopoisk, IMDb) → fetches metadata/ratings
6. Response flows back through the same layers

Keep business logic in the engine, not in Tauri or UI layers.

## Testing

- Write tests for new Rust functionality using `#[cfg(test)]` modules
- Use `cargo test` to run all tests
- For UI components, test behavior and props, not implementation details
- Aim for high coverage on critical paths (search, deduplication, scoring)
