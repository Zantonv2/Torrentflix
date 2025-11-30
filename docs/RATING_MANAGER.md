# Rating Manager Architecture

## Status: ✅ Core Implementation Complete | ⚠️ Integration Pending

## Overview

The Rating Manager is a background service that ensures all movies in the application have ratings from both Kinopoisk and IMDB. It operates automatically on every page load, checking for missing ratings and fetching them using separate workers. The system uses 3 workers total: 1 Kinopoisk worker, and 2 TMDB workers (one for TMDB metadata, one for IMDb scraping).

### Implementation Status
- ✅ **Core Components**: All workers, manager, and cache implemented
- ✅ **Data Structures**: All models and result types defined
- ✅ **Caching**: In-memory cache fully functional
- ⚠️ **Engine Integration**: Manager not yet integrated into `Engine`
- ❌ **UI Integration**: Not yet connected to frontend

## Architecture Flow

### 1. Initialization (App Load)

When the application loads, the Rating Manager is initialized and immediately begins checking ratings for all movies in the current dataset.

```
App Loads → Rating Manager Initialized → Check All Movies for Ratings
```

### Worker Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Rating Manager                            │
│              (Checks for missing ratings)                    │
└─────────────────────────────────────────────────────────────┘
                            │
                            ├─────────────────┐
                            │                 │
                            ▼                 ▼
        ┌──────────────────────────┐  ┌──────────────────────────┐
        │   Kinopoisk Worker       │  │   TMDB Worker 1           │
        │   (Independent)          │  │   (Metadata Fetcher)       │
        │                          │  │                          │
        │  - Fetches Kinopoisk     │  │  - Fetches TMDB metadata │
        │    ratings               │  │  - Gets episodes/seasons  │
        │  - No dependencies       │  │  - Gets external_id      │
        │                          │  │    (including imdb_id)    │
        └──────────────────────────┘  └──────────────────────────┘
                                              │
                                              │ (waits for completion)
                                              ▼
                                    ┌──────────────────────────┐
                                    │   TMDB Worker 2           │
                                    │   (IMDb Scraper)          │
                                    │                          │
                                    │  - Uses imdb_id from      │
                                    │    TMDB Worker 1          │
                                    │  - Scrapes IMDb page      │
                                    │  - Extracts rating        │
                                    └──────────────────────────┘
                                              │
                                              ▼
                                    ┌──────────────────────────┐
                                    │   Merge Results          │
                                    │   Update EnrichedMedia   │
                                    └──────────────────────────┘
```

### 2. Rating Check Phase

The manager examines each movie's `EnrichedMedia` structure and identifies which ratings are missing:

- **Kinopoisk Rating**: `rating_kinopoisk: Option<f32>`
- **IMDB Rating**: `rating_imdb: Option<f32>`
- **TMDB Rating**: `rating_tmdb: Option<f32>` (already fetched during initial enrichment)

The manager creates two separate lists:
- Movies missing Kinopoisk ratings
- Movies missing IMDB ratings

### 3. Worker Spawning

Based on the missing ratings, the manager spawns workers:

**Worker Types:**
1. **Kinopoisk Worker**: Fetches Kinopoisk ratings independently
2. **TMDB Worker 1**: Fetches TMDB metadata (episodes, seasons, external_id including imdb_id)
3. **TMDB Worker 2 (IMDb Scraper)**: Waits for TMDB Worker 1 to complete, then uses the imdb_id to scrape IMDb ratings

**Scenario A: Both ratings missing**
- Spawns 3 workers: 1 for Kinopoisk, 2 for TMDB (metadata + IMDb scraping)
- Kinopoisk worker operates independently
- TMDB Worker 2 waits for TMDB Worker 1 to complete

**Scenario B: Only Kinopoisk missing**
- Spawns 1 worker: Kinopoisk worker only

**Scenario C: Only IMDB missing**
- Spawns 2 workers: TMDB Worker 1 (to get imdb_id) and TMDB Worker 2 (IMDb scraper)
- TMDB Worker 2 waits for TMDB Worker 1 to complete

**Scenario D: All ratings present**
- No workers spawned, manager completes immediately

### 4. Worker Execution

Each worker receives its respective list of movies and processes them:

```
Kinopoisk Worker:
  Input: [Movie1, Movie2, Movie3, Movie4, Movie5]  (5 movies missing Kinopoisk)
  Process: Fetch ratings for all 5 movies in parallel/batch
  Output: Map<MovieID, Option<f32>>  (ratings or None if not found)
  Dependency: None (independent)

TMDB Worker 1 (Metadata Fetcher):
  Input: [Movie1, Movie2, Movie3]  (movies needing TMDB metadata)
  Process: Fetch TMDB metadata (episodes, seasons, external_id including imdb_id)
  Output: Map<MovieID, TMDBMetadata>  (metadata including imdb_id)
  Dependency: None (independent)

TMDB Worker 2 (IMDb Scraper):
  Input: [Movie1, Movie2, Movie3]  (movies missing IMDB rating)
  Process: 
    1. Wait for TMDB Worker 1 to complete
    2. Extract imdb_id from TMDB Worker 1 results
    3. Scrape IMDb ratings using imdb_id
  Output: Map<MovieID, Option<f32>>  (ratings or None if not found)
  Dependency: TMDB Worker 1 (must complete first)
```

**Note**: 
- Kinopoisk worker operates independently
- TMDB Worker 1 and Kinopoisk worker can run in parallel
- TMDB Worker 2 waits for TMDB Worker 1 to complete before starting
- The manager waits for all workers to complete before merging results

### 5. Manager Coordination

The Rating Manager uses async coordination to wait for all spawned workers:

```rust
// Pseudocode
let kinopoisk_handle = if needs_kinopoisk {
    Some(spawn_kinopoisk_worker(movies_missing_kinopoisk))
} else {
    None
};

let tmdb_metadata_handle = if needs_imdb {
    Some(spawn_tmdb_metadata_worker(movies_missing_imdb))
} else {
    None
};

// Wait for TMDB Worker 1 to complete first (if needed)
let tmdb_metadata_results = if let Some(handle) = tmdb_metadata_handle {
    Some(handle.await?)
} else {
    None
};

// Now spawn IMDb scraper with imdb_ids from TMDB Worker 1
let imdb_handle = if needs_imdb && tmdb_metadata_results.is_some() {
    let imdb_ids = extract_imdb_ids(tmdb_metadata_results.unwrap());
    Some(spawn_imdb_scraper_worker(imdb_ids))
} else {
    None
};

// Wait for all independent workers to complete
let (kinopoisk_results, imdb_results) = tokio::try_join!(
    kinopoisk_handle,
    imdb_handle
).await?;
```

### 6. Missing Rating Handling

Some movies may not have ratings available on certain platforms:

- **Kinopoisk**: Russian/CIS content may not exist on IMDB
- **IMDB**: International content may not exist on Kinopoisk
- **Both missing**: Very rare, but possible for obscure content

**Behavior**:
- If a rating is not found, the worker returns `None` for that movie
- The manager logs the missing rating (with movie title and source)
- The manager does NOT fail or block on missing ratings
- The movie continues with available ratings only

**Logging Format**:
```
[WARN] Rating Manager: Movie "Title (Year)" missing Kinopoisk rating
[WARN] Rating Manager: Movie "Title (Year)" missing IMDB rating
[INFO] Rating Manager: Movie "Title (Year)" missing both ratings (using TMDB only)
```

### 7. Result Merging and Caching

Once all workers complete:

1. **Merge Results**: Combine worker results back into the `EnrichedMedia` structures
2. **Update Models**: Update each movie's rating fields and metadata (episodes, seasons, external_ids)
3. **Cache Results**: Store full worker responses in cache (not just ratings):
   - **TMDB Worker 1**: Cache metadata (episodes, seasons, external_id including imdb_id)
   - **TMDB Worker 2**: Cache IMDb ratings
   - **Kinopoisk Worker**: Cache Kinopoisk ratings
4. **Notify UI**: Pass updated movies to UI (or trigger UI refresh)

### 8. Caching Strategy

**Full worker responses are cached** to prevent unnecessary API calls and improve performance:

#### Cache Structure

- **TMDB Metadata Cache**: 
  - **Cache Key**: `tmdb_metadata:{tmdb_id}` or `tmdb_metadata:{title}:{year}`
  - **Cached Data**: Full metadata response (episodes, seasons, external_id including imdb_id)
  - **Cache Duration**: TBD (suggested: 24 hours or permanent for stable metadata)

- **IMDb Rating Cache**:
  - **Cache Key**: `imdb_rating:{imdb_id}` (e.g., `imdb_rating:tt1234567`)
  - **Cached Data**: IMDb rating (f32)
  - **Cache Duration**: TBD (suggested: 24 hours or permanent for stable ratings)

- **Kinopoisk Rating Cache**:
  - **Cache Key**: `kinopoisk_rating:{kinopoisk_id}` or `kinopoisk_rating:{title}:{year}`
  - **Cached Data**: Kinopoisk rating (f32)
  - **Cache Duration**: TBD (suggested: 24 hours or permanent for stable ratings)

#### Cache Lookup Flow

On subsequent page loads:

1. **Check TMDB Metadata Cache First**:
   - If cached metadata exists → use `imdb_id` from cache
   - If not cached → spawn TMDB Worker 1 to fetch metadata

2. **Check IMDb Rating Cache**:
   - If `imdb_id` is available (from cache or Worker 1) → check IMDb rating cache
   - If cached → use cached rating, skip TMDB Worker 2
   - If not cached → spawn TMDB Worker 2 (IMDb scraper)

3. **Check Kinopoisk Rating Cache**:
   - If cached → use cached rating, skip Kinopoisk worker
   - If not cached → spawn Kinopoisk worker

#### Benefits

- **Reduces API calls**: Avoids redundant requests to TMDB, IMDb, and Kinopoisk
- **Faster page loads**: Cached data is available immediately
- **Preserves metadata**: TMDB metadata (episodes, seasons) is cached, not just ratings
- **Efficient lookups**: IMDb scraper can skip if rating is already cached

## Data Structures

### EnrichedMedia (Existing)
```rust
pub struct EnrichedMedia {
    // ... other fields ...
    pub rating_imdb: Option<f32>,
    pub rating_tmdb: Option<f32>,
    pub rating_kinopoisk: Option<f32>,
    // ... other fields ...
}
```

### Rating Worker Result
```rust
pub struct RatingWorkerResult {
    pub movie_id: String,  // Identity key from ParsedMedia
    pub rating: Option<f32>,
    pub source: RatingSource,
    pub fetched_at: DateTime<Utc>,
}

pub enum RatingSource {
    Kinopoisk,
    Imdb,
}

pub struct TMDBMetadataResult {
    pub movie_id: String,
    pub imdb_id: Option<String>,
    pub episodes: Option<u32>,
    pub seasons: Option<u32>,
    pub fetched_at: DateTime<Utc>,
}
```

### Rating Manager State
```rust
pub struct RatingManager {
    cache: Arc<Mutex<RatingCache>>,
    client: reqwest::Client,
    // Configuration
}

pub struct RatingCache {
    // TMDB Worker 1 cache: Full metadata responses
    tmdb_metadata: HashMap<String, CachedTMDBMetadata>,
    
    // TMDB Worker 2 cache: IMDb ratings
    imdb_ratings: HashMap<String, CachedRating>,
    
    // Kinopoisk Worker cache: Kinopoisk ratings
    kinopoisk_ratings: HashMap<String, CachedRating>,
}

pub struct CachedTMDBMetadata {
    pub imdb_id: Option<String>,
    pub episodes: Option<u32>,
    pub seasons: Option<u32>,
    pub cached_at: DateTime<Utc>,
}

pub struct CachedRating {
    pub rating: Option<f32>,
    pub cached_at: DateTime<Utc>,
}
```

## Implementation Phases

### Phase 1: Rating Fetching ✅ COMPLETE
- ✅ Implement Kinopoisk rating fetcher (`engine/src/ratings/kinopoisk.rs`)
- ✅ Implement TMDB metadata fetcher (episodes, seasons, external_id) - uses `engine/src/metadata/tmdb.rs`
- ✅ Implement IMDB rating scraper (uses imdb_id from TMDB) (`engine/src/ratings/imdb.rs`)
- ✅ Handle API errors and missing ratings gracefully (all workers handle errors, return `None` on failure)
- ✅ Basic logging (tracing::info, warn, debug throughout)

### Phase 2: Rating Manager ✅ COMPLETE
- ✅ Implement manager coordination logic (`engine/src/ratings/manager.rs`)
- ✅ Worker spawning and async coordination (using `tokio::spawn`)
- ✅ Result merging (merges worker results back into `EnrichedMedia`)
- ⚠️ Integration with existing EnrichedMedia flow - **TODO**: Integrate into `engine.rs`

### Phase 3: Caching ✅ COMPLETE (In-Memory)
- ✅ Implement cache storage for all worker responses (`engine/src/ratings/cache.rs`)
  - ✅ TMDB metadata cache (episodes, seasons, external_id)
  - ✅ IMDb rating cache
  - ✅ Kinopoisk rating cache
- ✅ Cache lookup and invalidation
- ✅ Cache-aware worker spawning (skip workers if data is cached)
- ⚠️ Persistent cache (optional) - **TODO**: Add disk-backed cache (SQLite/JSON)

### Phase 4: UI Integration ❌ NOT STARTED
- ❌ Trigger manager on page load
- ❌ Update UI when ratings are fetched
- ❌ Show loading states for ratings

## Remaining Tasks

### High Priority
1. **Integrate Rating Manager into Engine** (`engine/src/engine.rs`)
   - Add `RatingManager` to `Engine` struct
   - Call `check_and_fetch_ratings()` after metadata enrichment
   - Ensure ratings are updated in `EnrichedMedia` before returning to UI

2. **UI Integration**
   - Trigger rating manager when app loads
   - Update UI components to show ratings as they're fetched
   - Add loading indicators for ratings

### Medium Priority
3. **Persistent Cache**
   - Add disk-backed cache (SQLite or JSON file)
   - Load cache on startup, save on updates
   - Cache expiration/invalidation logic

4. **Error Handling Improvements**
   - Add retry logic with exponential backoff
   - Better rate limiting handling
   - More detailed error logging

### Low Priority
5. **Performance Optimizations**
   - Batch API requests where possible
   - Add request timeouts
   - Optimize cache key generation

## API Integration Notes

### Kinopoisk API
- Requires API key
- Rate limiting considerations
- Search by title/year or use existing `kinopoisk_id` if available

### IMDB Scraping
- No API key required (web scraping)
- Uses imdb_id obtained from TMDB Worker 1
- Scrapes HTML from `https://www.imdb.com/title/{imdb_id}/`
- Parses rating from `div[data-testid="hero-rating-bar__aggregate-rating__score"]`
- Rate limiting considerations (respectful scraping)

### Error Handling
- Network errors: Retry with exponential backoff
- Rate limit errors: Queue requests, respect rate limits
- Not found: Return `None`, log warning
- Invalid response: Log error, skip movie, continue with others

## Performance Considerations

- **Parallel Fetching**: Workers fetch data in parallel when possible (Kinopoisk + TMDB Worker 1)
- **Batch Requests**: If APIs support it, batch multiple movies in single request
- **Comprehensive Caching**: Caches full worker responses (metadata + ratings), not just ratings
- **Cache-Aware Workers**: Workers are skipped if their data is already cached
- **Non-blocking**: Manager doesn't block UI while fetching
- **Timeout**: Set reasonable timeouts for API requests (e.g., 5-10 seconds)

## Logging and Monitoring

The manager should log:
- Manager initialization
- Number of movies checked
- Number of missing ratings/metadata detected
- Cache lookup results (hits/misses for each cache type)
- Worker spawning (including which movies are skipped due to cache)
- Worker completion (with timing)
- Missing ratings/metadata (warnings)
- Cache writes (when new data is cached)
- API errors

Example log output:
```
[INFO] Rating Manager: Checking ratings for 50 movies
[INFO] Rating Manager: Found 5 movies missing Kinopoisk ratings
[INFO] Rating Manager: Found 3 movies missing IMDB ratings
[INFO] Rating Manager: Cache lookup: 2 Kinopoisk ratings found in cache
[INFO] Rating Manager: Cache lookup: 0 TMDB metadata found in cache
[INFO] Rating Manager: Spawning Kinopoisk worker (3 movies, 2 from cache)
[INFO] Rating Manager: Spawning TMDB metadata worker (3 movies)
[INFO] Kinopoisk Worker: Fetched 2 ratings, 1 not found
[WARN] Rating Manager: Movie "Obscure Film (2020)" missing Kinopoisk rating
[INFO] TMDB Metadata Worker: Fetched metadata for 3 movies
[INFO] Rating Manager: Caching TMDB metadata for 3 movies
[INFO] Rating Manager: Cache lookup: 1 IMDb rating found in cache
[INFO] Rating Manager: Spawning IMDb scraper worker (2 movies with imdb_id, 1 from cache)
[INFO] IMDb Scraper Worker: Fetched 2 ratings, 0 not found
[INFO] Rating Manager: Caching IMDb ratings for 2 movies
[INFO] Rating Manager: All workers completed in 2.3s
[INFO] Rating Manager: Updated 7 movies with new ratings and metadata
```

## Quick Start / Integration Guide

### Current Implementation Location
- **Rating Manager**: `engine/src/ratings/manager.rs`
- **Workers**: `engine/src/ratings/kinopoisk.rs`, `engine/src/ratings/imdb.rs`
- **Cache**: `engine/src/ratings/cache.rs`
- **Models**: `engine/src/ratings/models.rs`

### Integration Steps

1. **Add RatingManager to Engine**:
   ```rust
   // In engine.rs
   use crate::ratings::RatingManager;
   
   pub struct Engine {
       // ... existing fields ...
       rating_manager: Arc<RatingManager>,
   }
   ```

2. **Initialize RatingManager**:
   ```rust
   let rating_manager = Arc::new(RatingManager::new(tmdb_client)?);
   ```

3. **Call after metadata enrichment**:
   ```rust
   let enriched_movies = self.enrich_with_metadata(parsed_media).await?;
   let enriched_with_ratings = self.rating_manager
       .check_and_fetch_ratings(vec![enriched_movies])
       .await?;
   ```

### Next Steps
See "Remaining Tasks" section above for integration and UI work.

