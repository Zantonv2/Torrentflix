# 🚀 TorrentFlix Quick Reference

## What Works RIGHT NOW

### Search & Discovery
```bash
# Search for movies
POST /search?query=Dune

# Get feed of recent movies
GET /get_feed

# Returns: List of movies with metadata, ratings, and download links
```

### Ratings (Real-Time)
- ✅ Kinopoisk ratings (Russian database)
- ✅ IMDb ratings (web scraping)
- ✅ TMDB ratings (API)
- ✅ All 3 ratings display in UI: `KP: 6.5 | IMDB: 7.2 | TMDB: 7.8`
- ✅ Fetched in background (non-blocking)
- ✅ Cached in SQLite for fast subsequent loads

### Downloads
```bash
# Add torrent to qBittorrent
POST /add_download?magnet=magnet:?xt=urn:btih:...

# Get active downloads
GET /get_active_downloads

# Pause/Resume/Delete
POST /pause_download?hash=...
POST /resume_download?hash=...
POST /delete_download?hash=...
```

### UI Tabs (2/6 Complete)
- ✅ **Feed** - Browse recent movies
- ✅ **Downloads** - Monitor active downloads
- ❌ **Library** - Browse downloaded files (not started)
- ❌ **History** - Download statistics (not started)
- ❌ **Bookmarks** - Wishlist (not started)
- ❌ **Settings** - Configuration (not started)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Svelte UI (TypeScript)                   │
│  Feed | Downloads | Library | History | Bookmarks | Settings│
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                  Tauri Commands Bridge                       │
│  search | get_feed | add_download | get_active_downloads   │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                    Rust Engine (Tokio)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Indexers    │  │  Metadata    │  │  Ratings     │      │
│  │  (Monna2)    │  │  (TMDB)      │  │ (KP, IMDB)   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Normalizer  │  │  Deduplicator│  │  Scorer      │      │
│  │  (guessit)   │  │  (grouping)  │  │  (ranking)   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Downloads   │  │  Job Manager │  │  Database    │      │
│  │  (qBittorrent)│  │  (RatingMgr) │  │  (SQLite)    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                  External APIs & Services                    │
│  TMDB | Kinopoisk | IMDb | qBittorrent | Monna2             │
└─────────────────────────────────────────────────────────────┘
```

---

## Key Files & Modules

### Backend (Rust)
```
engine/src/
├── engine.rs              # Main orchestrator
├── indexers/              # Torrent indexers
│   ├── monna.rs          # Monna2 indexer (needs metadata fix)
│   └── manager.rs        # Indexer orchestration
├── metadata/              # TMDB client
│   └── tmdb.rs           # TMDB API integration
├── ratings/               # Rating aggregation
│   ├── manager.rs        # RatingManager (3 workers)
│   ├── kinopoisk.rs      # Kinopoisk API
│   ├── imdb.rs           # IMDb scraper
│   └── cache.rs          # Rating cache
├── normalize/             # Title parsing
│   └── guessit.rs        # Pattern-based normalizer
├── torrent/               # Download management
│   └── client.rs         # qBittorrent WebAPI
├── database/              # SQLite
│   └── ratings.rs        # Rating cache table
└── jobs/                  # Job system
    └── manager.rs        # JobManager (wraps RatingManager)
```

### Frontend (Svelte)
```
ui/src/
├── App.svelte            # Root component
├── components/
│   ├── TabBar.svelte     # Navigation (6 tabs)
│   ├── MovieGrid.svelte  # Grid layout
│   ├── MovieTile.svelte  # Movie card
│   ├── DetailPanel.svelte # Movie details
│   ├── DownloadsTab.svelte # Downloads (✅ working)
│   ├── SearchBar.svelte  # Search input
│   └── ... (other components)
└── types.ts              # TypeScript interfaces
```

### Tauri Bridge
```
src-tauri/src/
├── main.rs               # App initialization
└── commands.rs           # Tauri command handlers
```

---

## Environment Variables

```bash
# Required
TMDB_API_KEY=your_tmdb_api_key

# Optional
KINOPOISK_API_KEY=your_kinopoisk_api_key
ADDRESS_PORT=127.0.0.1:1080  # SOCKS5 proxy

# qBittorrent (defaults shown)
QBITTORRENT_URL=http://localhost:5555
QBITTORRENT_USERNAME=admin
QBITTORRENT_PASSWORD=adminadmin

# Database
DATABASE_URL=sqlite://.data/torrentflix.db
```

---

## Common Commands

### Development
```bash
npm run dev              # Start dev server + Tauri
npm run ui:dev          # UI only (Vite on :5173)
npm run tauri:dev       # Tauri only (requires UI running)
```

### Building
```bash
npm run build           # Build UI + Tauri
npm run ui:build        # Build UI only
npm run tauri:build     # Build Tauri only
```

### Testing
```bash
cargo test --package engine              # Run Rust tests
cargo test --package engine -- --nocapture  # With output
```

### Linting
```bash
cargo clippy --workspace  # Rust linter
npm run lint              # ESLint (UI)
npm run format            # Prettier (UI)
```

---

## Data Flow Examples

### Search Flow
```
User types "Dune" in SearchBar
    ↓
Tauri command: search("Dune")
    ↓
Engine.search("Dune")
    ├─ IndexerManager.search_all() → Monna2
    ├─ Normalizer.normalize() → Parse titles
    ├─ MetadataManager.search() → TMDB lookup
    ├─ fetch_ratings_for_results() → Background rating fetch
    └─ Return results immediately (ratings update in background)
    ↓
UI displays results with skeleton loaders
    ↓
Ratings arrive via Tauri event → UI updates
```

### Rating Fetch Flow
```
Engine.fetch_ratings_for_results([movies])
    ↓
RatingManager.check_and_fetch_ratings()
    ├─ Check cache for each movie
    ├─ Spawn workers for missing ratings:
    │   ├─ Kinopoisk Worker (independent)
    │   ├─ TMDB Worker 1 (metadata + imdb_id)
    │   └─ TMDB Worker 2 (IMDb scraper, waits for Worker 1)
    ├─ Wait for all workers
    ├─ Merge results
    ├─ Update cache
    └─ Send updates to UI via channel
    ↓
UI receives RatingUpdate events
    ↓
UI updates movie cards with new ratings
```

### Download Flow
```
User clicks "Download" on movie
    ↓
Tauri command: add_download(magnet_url)
    ↓
Engine.add_download(magnet_url)
    ├─ QBittorrentClient.login()
    ├─ QBittorrentClient.add_torrent(magnet_url)
    └─ Return download hash
    ↓
UI adds to downloads list
    ↓
Every 2 seconds: get_active_downloads()
    ├─ QBittorrentClient.get_all_torrents()
    └─ Return progress, speed, ETA, seeders, leechers
    ↓
UI updates progress bars in real-time
```

---

## What's NOT Implemented Yet

### Settings Tab
- No UI for API key configuration
- No UI for qBittorrent settings
- No UI for library paths
- Users must edit .env files

### Library Tab
- No file scanner
- No library database queries
- No UI for browsing downloaded files

### History Tab
- No download history tracking
- No statistics aggregation
- No charts/graphs

### Bookmarks Tab
- No wishlist functionality
- No quality filter UI
- No periodic monitoring

### Persistent Job Queue
- Jobs execute immediately (no persistence)
- No retry logic
- No scheduling
- No job dependencies

---

## Performance Tips

### For Users
1. **First load is slow** - Ratings are fetched in background
2. **Subsequent loads are fast** - Ratings cached in SQLite
3. **Large result sets** - Virtual scrolling handles 100+ movies
4. **Network dependent** - Slow internet = slow rating fetch

### For Developers
1. **Indexer search** - Monna2 indexer
2. **Non-blocking rating fetch** - Background task doesn't block UI
3. **Aggressive caching** - TMDB metadata, ratings, posters all cached
4. **Batch operations** - Use transactions for multi-step DB operations

---

## Debugging Tips

### Enable Logging
```bash
RUST_LOG=debug npm run dev
```

### Check Database
```bash
sqlite3 .data/torrentflix.db
SELECT * FROM rating_cache LIMIT 5;
```

### Monitor qBittorrent
```bash
# Web UI
http://localhost:5555
# Default: admin / adminadmin
```

### Check Tauri Events
```javascript
// In browser console
window.__TAURI__.event.listen('rating-update', (event) => {
  console.log('Rating update:', event.payload);
});
```

---

## Known Limitations

1. **Monna metadata parsing** - Fails on field label variations
2. **No Settings UI** - Configuration via .env only
3. **No Library management** - Can't browse downloaded files
4. **No Bookmarks** - Can't wishlist movies
5. **No History** - No download statistics
6. **No Job persistence** - Jobs lost on crash
7. **No Notifications** - No desktop alerts (yet)

---

## Next Steps

**Priority 1** (2-3 hours): Fix Monna metadata parsing  
**Priority 2** (1-2 days): Implement Settings UI  
**Priority 3** (3-4 days): Implement Library management  
**Priority 4** (2-3 days): Implement History/Analytics  

See `docs/IMPLEMENTATION_STATUS.md` for detailed breakdown.

---

*Last updated: December 2, 2025*
