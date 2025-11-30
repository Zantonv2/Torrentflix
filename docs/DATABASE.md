---

# 📄 DATABASE MODULE SPEC

## 1. Purpose & Role of Database Module

The database module serves as the **single source of truth** for all persistent application state:

* Media library metadata (movies/shows, versions, file paths, metadata, quality, status)
* File-version records (path, fingerprint, status, history)
* Download & import history
* **Job queue / background tasks** (pending, working, succeeded, failed jobs) - See [JOB_MANAGER.md](./JOB_MANAGER.md) for details
* Configuration / user settings (enabled indexers, paths, preferences, policies)
* Audit / history log of user and system actions (deletions, cleanup, import, dedupe, renames)
* (Optional) Cache metadata (e.g. poster/artwork cache links), trash / soft-delete records, other state
* **Rating cache** (persistent cache table for Kinopoisk, IMDb, and TMDB ratings)

The database must support:

* ACID semantics (atomic transactions across multi-step operations)
* Concurrency: simultaneous reads (UI, queries) + writes (jobs, metadata updates, cleanup)
* Durability: survive app / system crashes, maintain consistency between DB and filesystem state
* Performance: remain responsive under heavy load (large library, many metadata entries, frequent writes/reads)
* Flexibility: schema evolves over time, ability to add tables/fields, migrations, support version/edition, audit history, etc.

Given all this, the recommended backend is **SQLite** (embedded SQL database).

---

## 2. SQLite Configuration & Initialization (Pragmas & Defaults)

To maximize concurrency, performance and acceptable durability trade-offs, the following configuration should be applied on every DB connection (or on initialization):

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
```

### Why these settings

* **WAL (Write-Ahead Logging) mode** — allows concurrent reads and writes; readers do not block writers and vice versa, significantly improving concurrency. ([SQLite][1])
* **synchronous = NORMAL** — reduces I/O overhead while still ensuring crash-resilience in most cases; durability of WAL + checkpoint ensures consistency under normal shutdowns. ([Android Developers][2])
* **foreign_keys = ON** — ensures referential integrity (e.g. between media items and file-versions, deletes cascade properly, etc.)

### Additional recommended practices

* Use **short transactions** and **batched writes** (whenever possible) to avoid long-running locks or contention. ([Sling Academy][3])
* Use **statement caching / prepared statements** rather than dynamic SQL strings in loops for performance. ([Sling Academy][3])
* Run **`ANALYZE`** periodically (after significant changes) to update internal statistics and help the query planner optimize performance. ([Howik][4])
* When many deletions / updates have been done (e.g. cleanup, dedupe), consider **`VACUUM`** to reduce file size / fragmentation — but only in maintenance windows (not during active usage). ([Howik][4])

---

## 3. Core Schema — Tables, Purpose & Relations

Here is a recommended starting schema. You may evolve, add columns, or adjust according to features. Use UUIDs (or strings) for primary keys, avoiding DB-generated integer IDs where cross-references outside DB (e.g. file paths) matter.

```sql
-- Enable foreign-keys (if not globally enabled)
PRAGMA foreign_keys = ON;

-- Logical media items (movies / series / shows)
CREATE TABLE media_items (
    id TEXT PRIMARY KEY,
    canonical_title TEXT NOT NULL,
    release_year INTEGER,
    metadata_snapshot JSON,        -- merged metadata from external providers + manual edits
    user_rating REAL,              -- optional user rating
    created_at INTEGER NOT NULL,   -- timestamp
    updated_at INTEGER NOT NULL    -- timestamp
);

-- Physical file versions on disk
CREATE TABLE file_versions (
    id TEXT PRIMARY KEY,
    media_item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    absolute_path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    resolution TEXT,               -- e.g. '1080p', '2160p'
    codec TEXT,                    -- optional codec info string or JSON
    fingerprint_fast TEXT,         -- e.g. hash of first/last chunks
    fingerprint_full TEXT,         -- optional full-file hash
    quality_label TEXT,            -- e.g. 'WEB-DL', 'BluRay-Remux', etc.
    added_at INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL, -- for rescans / existence checks
    status TEXT NOT NULL,          -- e.g. 'present', 'missing', 'deleted', 'trashed'
    is_preferred INTEGER NOT NULL DEFAULT 0, -- which version is preferred
    download_id TEXT NULL,         -- if file came from a download
    extra_metadata JSON            -- e.g. subtitle info, audio tracks, etc.
);

-- Downloads / Import history (optional)
CREATE TABLE downloads (
    download_id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    completed_at INTEGER NULL
);

-- Background job queue for async / maintenance tasks
-- Supports persistent job execution, retries, dependencies, and scheduling
CREATE TABLE jobs (
    job_id TEXT PRIMARY KEY,              -- UUID identifier
    kind TEXT NOT NULL,                    -- Job type: 'FetchRatings', 'CommitFile', 'CleanupStale', 'MetadataRefresh', etc.
    params JSON NOT NULL,                  -- Serialized job parameters (arguments, context)
    status TEXT NOT NULL,                  -- 'pending', 'waiting', 'working', 'succeeded', 'failed', 'cancelled'
    priority INTEGER NOT NULL DEFAULT 0,   -- Higher = run earlier when scheduling
    attempts INTEGER NOT NULL DEFAULT 0,  -- Number of times this job has been attempted
    max_attempts INTEGER NOT NULL DEFAULT 3, -- How many times to retry before marking dead
    depends_on TEXT NULL,                  -- job_id of another job this one depends on (optional)
    scheduled_at INTEGER NOT NULL,         -- When job is eligible to run (for delayed jobs/backoff)
    created_at INTEGER NOT NULL,           -- When job was created/enqueued
    started_at INTEGER NULL,               -- When job execution began
    finished_at INTEGER NULL,              -- When job execution ended (success or failure)
    last_error TEXT NULL,                  -- Error message or serialized error info if failed
    result JSON NULL                       -- Optional result data or metadata from job execution
);

-- Audit / history log (for destructive or important operations)
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    action TEXT NOT NULL,
    object_type TEXT NOT NULL,   -- e.g. 'file_version', 'media_item', 'delete', 'rename'
    object_id TEXT NOT NULL,
    before_state JSON,
    after_state JSON
);

-- Configuration / Settings (key-value or JSON blob per user)
CREATE TABLE config (
    key TEXT PRIMARY KEY,
    value JSON NOT NULL
);

-- (Optional) Trash / Soft-delete buffer — for file_versions moved to trash
CREATE TABLE trash_versions (
    id TEXT PRIMARY KEY,
    file_version_id TEXT NOT NULL REFERENCES file_versions(id),
    trashed_at INTEGER NOT NULL,
    original_path TEXT NOT NULL
    -- possibly other metadata
);

-- (Optional) Poster/Artwork Cache
CREATE TABLE poster_cache (
    media_item_id TEXT PRIMARY KEY REFERENCES media_items(id),
    local_path TEXT NOT NULL,
    fetched_at INTEGER NOT NULL
);

-- Rating Cache (persistent cache for ratings from external APIs)
CREATE TABLE rating_cache (
    movie_id TEXT PRIMARY KEY,              -- Identity key: "movie:Title:Year" or "series:Title:S01E01"
    rating_kinopoisk REAL NULL,             -- Kinopoisk rating (0-10 scale)
    rating_imdb REAL NULL,                   -- IMDb rating (0-10 scale)
    rating_tmdb REAL NULL,                   -- TMDB rating (0-10 scale)
    cached_at INTEGER NOT NULL,              -- When rating was first cached
    updated_at INTEGER NOT NULL              -- When rating was last updated
);

-- (Optional) Job History / Archive
-- Store completed or failed jobs for audit, metrics or debugging
-- Can use same `jobs` table (keep all) or separate archive table
CREATE TABLE jobs_history (
    job_id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    params JSON NOT NULL,
    status TEXT NOT NULL,
    attempts INTEGER NOT NULL,
    finished_at INTEGER NOT NULL,
    last_error TEXT NULL,
    result JSON NULL,
    archived_at INTEGER NOT NULL
);
```

### Index recommendations

Create indexes on columns frequently used in queries / filtering / joins:

* `media_items(canonical_title)` — for search by title
* `file_versions(media_item_id)` — for joining versions to media item
* `file_versions(status)` — for filtering present/missing/deleted versions
* `file_versions(is_preferred)` — to quickly find preferred versions
* `file_versions(absolute_path)` — for rescans / FS operations / path lookup
* `jobs(status, priority, scheduled_at)` — to quickly fetch pending jobs (status='pending' AND scheduled_at <= now, ordered by priority DESC)
* `jobs(depends_on)` — to find jobs waiting on dependencies (WHERE depends_on = ?)
* `jobs(status, kind)` — for filtering jobs by type and status
* `jobs(created_at)` — for job history queries and cleanup of old jobs
* `rating_cache(cached_at)` — for cleanup of old rating cache entries
* `rating_cache(updated_at)` — for finding recently updated ratings

You may add additional indexes depending on UI/filtering needs (e.g. resolution, quality_label, last_seen_at).

---

## 4. Maintenance & Integrity Practices

To keep DB healthy over time, and avoid corruption or performance degradation, follow these practices:

### 🔄 Periodic Maintenance Tasks

* **Checkpoint / WAL file trimming**

  * SQLite automatically checkpoints when WAL file grows to ~1000 pages by default. ([SQLite][1])
  * However: if there are long-lived read transactions (e.g. UI browsing), checkpoint may lag → WAL file may grow unbounded → degrade performance. In that case, schedule **manual or timed checkpoint** (e.g. full / truncate) during idle periods.

* **`ANALYZE`** — after large number of inserts/deletes/updates or schema changes, run `ANALYZE` so SQLite updates query planner statistics. Helps maintain query performance over time. ([Howik][4])

* **`VACUUM`** — after many deletions (e.g. cleanup, dedupe, trash purge), optionally vacuum database to reclaim space and reduce fragmentation. But do this during downtime/maintenance — VACUUM may block access while running. ([Howik][4])

* **Job Queue Cleanup** — periodically archive or delete old completed jobs (`status = 'succeeded'` or `status = 'failed'` with `finished_at` older than retention period) to prevent `jobs` table from growing unbounded. Move to `jobs_history` table or delete after archival.

* **Integrity checks / backups** — periodically (or on startup / shutdown) run `PRAGMA integrity_check;` to verify DB integrity; support exporting / backing-up DB (copying file + WAL + SHM) to allow recovery in catastrophic failures; archive backups occasionally.

### 🧪 Transaction & Access Best Practices

* **Batch writes**: combine related DB operations (e.g. file_commit + metadata insertion + version registration) in a single transaction — reduces overhead, avoids partial state.
* **Keep transactions short** — long-running transactions hinder concurrency, block checkpoints.
* **Use prepared statements or parameterized queries** for repeated operations (in job workers) to reduce SQL parsing overhead.
* **Handle `SQLITE_BUSY` / lock contention** gracefully: implement retry logic with some backoff (job system), or queue writes serially if needed. Concurrent writes are serialized anyway. ([SQLite][1])
* **Job claiming must be atomic**: When claiming a job (`status = 'pending'` → `status = 'working'`), use a transaction with `SELECT ... FOR UPDATE` or `UPDATE ... WHERE status = 'pending'` to ensure only one worker claims the job. This prevents duplicate execution.

---

## 5. Integration Points & Module Interaction

### Filesystem ↔ DB Coordination

* When a file is committed / moved / renamed on filesystem, update `file_versions.absolute_path`, `added_at`, etc., in same or subsequent transaction.
* If a file is deleted / trashed / moved out — mark `file_versions.status` as `deleted` or `trashed`, log in `audit_log` (with before/after), optionally move to `trash_versions`.
* On rescans (job): walk filesystem, compare actual files to DB records; for missing files — mark as `missing`; for new files — insert new `file_versions`, optionally link to existing `media_item` or create new.

### Job System ↔ DB

* **Job Queue**: Job queue and job state stored in `jobs` table. Background tasks (rating fetches, cleanup, rescan, metadata refresh, dedupe, trash purge) operate by reading/writing this table.
* **Job Lifecycle**: Jobs transition through states: `pending` → `waiting` (if dependencies) → `working` → `succeeded`/`failed`. Failed jobs with `attempts < max_attempts` are retried with backoff (updated `scheduled_at`).
* **Job Dependencies**: Jobs can depend on other jobs via `depends_on` field. Scheduler only runs jobs when dependencies are resolved (`status = 'succeeded'`).
* **Job Execution**: Job handlers perform DB operations (inserting/updating metadata, file_versions, audit logs) as part of job execution. Use transactions, ensure idempotence (so if job fails or is retried, DB remains consistent).
* **Job Observability**: UI may observe job statuses (pending, working, failed) via DB to show progress, history, failures. Query by `status`, `kind`, `priority` for filtering.
* **Job Recovery**: On startup, detect jobs with `status = 'working'` but stale `started_at` (no `finished_at`) — treat as failed and retry if `attempts < max_attempts`.
* **Job Kinds**: Current job kinds include:
  - `FetchRatings` - Fetch Kinopoisk/IMDb ratings (RatingManager)
  - `FetchTMDBMetadata` - Fetch TMDB metadata (RatingManager)
  - `ScrapeIMDbRating` - Scrape IMDb rating (RatingManager)
  - Future: `CommitFile`, `HashFile`, `RefreshMetadata`, `SyncLibrary`, `CleanupTrash`

### Config & Settings ↔ DB

* On application startup, load config/settings from `config` table; if missing, write defaults.
* Settings influence behavior of other modules (which indexers enabled, dedupe policy, cleanup thresholds, library root, storage paths, etc.).

### Library & Metadata Layer ↔ DB

* `media_items` store canonical metadata per movie/show.
* `file_versions` store per-file metadata (quality, path, status).
* UI queries join `media_items` ↔ `file_versions` to build library view: available versions, preferred version, status, quality, etc.
* When metadata updated (e.g. from external metadata provider), update `metadata_snapshot` in `media_items`.

### Audit & History

* Any destructive or important state-changing operation writes to `audit_log`, with before/after state — useful for debugging, undo, tracking.
* For soft-delete/trash — move record to `trash_versions`, store deletion time — allows restoration if needed, or purge later.

---

## 6. Schema Evolution & Migration Guidelines

As your application evolves, you will likely need to extend schema (new tables, columns) — here’s how to manage safely:

* Use **migrations scripts** rather than manual SQL ad hoc; sequence of migrations with version tags, so installations can upgrade smoothly.
* Before applying migration: **backup DB file + WAL + SHM** (full copy) — allow rollback on failure or corruption.
* For backward-incompatible changes (removing columns), consider deprecating first (mark as unused), or copy old data to new tables/columns then drop; document in migration notes.
* After large structural changes or bulk operations (deletions, expansions), run `ANALYZE`, maybe `VACUUM`, to reset statistics and clean fragmentation.
* Keep code that interacts with DB abstracted (data-access layer or repository pattern), so changing schema requires minimal code changes, and centralizes migration logic.

---

## 7. Edge-Case Handling & Known Pitfalls with SQLite + Workarounds

| Issue / Risk                                                                         | Mitigation / Best Practice                                                                                                 |
| ------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------- |
| **WAL file grows indefinitely** (if long-lived read transactions prevent checkpoint) | Schedule periodic manual checkpoints (e.g. full/truncate) during idle time or job system maintenance.                      |
| **“Database is locked” (SQLITE_BUSY)** during high write load + contention           | Keep transactions short, batch writes, use retry/backoff logic, limit concurrent writers.                                  |
| **Performance degradation with many deletes/inserts** → DB bloat / fragmentation     | Periodic VACUUM / incremental maintenance, avoid excessive deletes; recycle IDs if possible.                               |
| **Schema migration risk** (data loss, incompatible changes)                          | Use migration scripts, versioned migration plan, backup DB before migration, write forward/backward compatible migrations. |
| **Disk I/O or fsync overhead, especially on HDD / slow storage**                     | Combine writes, reduce frequency, avoid heavy writes while user is watching media / doing FS heavy operations.             |
| **Network filesystem incompatibility (WAL shared-memory not supported)**             | Use local filesystem for DB; avoid storing DB on NFS/SMB; or fallback to rollback-journal mode if necessary. ([SQLite][1]) |
| **Job queue table growth** (many completed jobs accumulate)                          | Implement job archival: move old completed jobs to `jobs_history` table or delete after retention period. Run as periodic maintenance job. |
| **Stuck jobs** (jobs with `status = 'working'` but no progress)                      | On startup, detect jobs with stale `started_at` (no `finished_at` after timeout) — mark as failed and retry if `attempts < max_attempts`. |

---

## 8. Backups, Integrity & Recovery Strategy

To protect user data (library metadata, file-version tracking, history) and allow recovery if errors / corruption occur:

* Provide a **“backup” command / UI action** — copy DB file + WAL + SHM to user-specified location, timestamped backup.
* On startup (or periodic maintenance), run `PRAGMA integrity_check;` — if corruption detected, alert user, possibly restore from last backup / repair.
* Avoid storing DB on volatile or unreliable storage (e.g. external drives prone to disconnect, network shares without proper locking).
* After large schema migrations or batch deletes, create backup, run vacuum if needed, then checkpoint.

---

## 9. Summary & Recommendations

* **SQLite + WAL + tuned pragmas** is a strong, lightweight, embedded DB solution for your media-library + job-system + metadata needs.
* A well-designed schema (media_items, file_versions, jobs, audit, config, etc.) supports versioning, dedupe, history, library metadata, and background tasks.
* **Job Queue Schema**: The `jobs` table supports persistent job execution with retries, dependencies, scheduling, and observability. See [JOB_MANAGER.md](./JOB_MANAGER.md) for detailed job system architecture.
* **Current Implementation**: RatingManager uses immediate execution (no queue yet). Future migration to persistent queue will use this schema.
* Maintenance tasks (checkpointing, analyze, vacuum, integrity check, backups, job archival) are essential to maintain performance and reliability over time.
* Edge-case handling (locking, fsync, backup, network FS issues, stuck jobs, queue growth) should be considered early to avoid data loss or corruption.
* Use migration scripts + data-access abstraction + backup strategy to evolve schema safely as features grow.

With this module spec, your database layer becomes a **robust, production-ready foundation** — reliable, performant, maintainable, and scalable.

## 10. Job Queue Schema Details

### Job Status Lifecycle

```
pending → waiting (if depends_on) → working → succeeded
                                      ↓
                                   failed (if attempts >= max_attempts)
                                      ↓
                                   pending (if attempts < max_attempts, with backoff)
```

### Job Priority & Scheduling

* Jobs with higher `priority` are selected first when multiple jobs are ready
* `scheduled_at` allows delayed execution (for retries with backoff, or scheduled maintenance)
* Scheduler selects jobs where: `status = 'pending'` AND `scheduled_at <= now()` AND `depends_on IS NULL OR depends_on IN (SELECT job_id FROM jobs WHERE status = 'succeeded')`

### Job Dependencies

* Jobs can depend on other jobs via `depends_on` field
* Dependent jobs remain in `waiting` or `pending` state until parent job succeeds
* If parent job fails permanently, dependent jobs should be marked as failed or cancelled (implementation decision)

### Job Retries & Backoff

* Failed jobs with `attempts < max_attempts` are retried
* Backoff delay: `scheduled_at = now() + backoff_duration(attempts)`
* Exponential backoff recommended: `backoff = base_delay * (2 ^ attempts)`
* After `max_attempts` exhausted, job marked as `failed` (dead-letter)

### Job Recovery on Startup

* Detect jobs with `status = 'working'` but stale `started_at` (e.g., older than 1 hour with no `finished_at`)
* These indicate jobs that were running when application crashed
* Options:
  1. Mark as failed and retry if `attempts < max_attempts`
  2. Attempt recovery (job-specific logic)
  3. Mark as failed permanently (if not idempotent)

---

