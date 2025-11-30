---

# 📄 JOB_MANAGER.md — Job Manager / Job System Specification

## Status: 🚧 Architecture Defined | ⚠️ Implementation In Progress

### Current Implementation Status

**Implemented Worker Managers:**
- ✅ **RatingManager** (`engine/src/ratings/manager.rs`) - Immediate execution with tokio workers
  - Fetches ratings from Kinopoisk and IMDb
  - Uses 3 workers: Kinopoisk (independent), TMDB Worker 1 (metadata), TMDB Worker 2 (IMDb scraper)
  - In-memory caching
  - Currently executes immediately when called (not queued)

**Planned Worker Managers:**
- ⏳ **DownloadManager** - Torrent download coordination
- ⏳ **MetadataRefreshManager** - Periodic metadata updates
- ⏳ **LibrarySyncManager** - Library scanning and synchronization
- ⏳ **FilesystemManager** - File operations (moves, renames, cleanup)

**Job Queue System:**
- ⏳ **Persistent Job Queue** - Database-backed job persistence (not yet implemented)
- ⏳ **Job Scheduler** - Scheduled and delayed job execution
- ⏳ **Job Dependencies** - Job chaining and dependencies

### Evolution Path

**Phase 1 (Current):** Immediate-execution worker managers
- RatingManager executes workers immediately using `tokio::spawn`
- No persistence, no queue
- Suitable for on-demand operations (fetch ratings when movies are loaded)

**Phase 2 (Future):** Persistent job queue integration
- Migrate worker managers to use JobManager
- Jobs persist to database
- Support retries, dependencies, scheduling
- Suitable for long-running operations (downloads, library sync, cleanup)

---

## 1. Purpose & Role of the Job Manager

The job manager’s main goal is to provide a **reliable, persistent, asynchronous background task processing system** for your application, to handle all long-running, IO-bound, filesystem, metadata-fetching, cleanup, maintenance tasks — decoupled from UI or request-handling logic.

Responsibilities:

* Accept new “jobs” (units of work) from various parts of the system (UI commands, download completion, maintenance triggers, rescans, cleanup, etc.).
* Persist jobs in a database (so jobs survive application restarts, crashes, power loss).
* Dispatch jobs to worker threads/tasks (or worker-groups) in a controlled, resource-aware way (throttling, concurrency limits).
* Collect results, update job statuses, record metadata (success/failure, timestamps, error messages).
* Support retries, backoff, failure handling, dead-letter / permanent failure marking.
* Allow for dependencies among jobs (job A must finish before job B starts).
* Provide visibility into job queue (pending, running, failed), history, logs — for debugging, monitoring, user feedback.
* Enable background maintenance: cleanup, rescans, metadata refresh, dedupe runs, trash purge, file-system integrity checks, orphan detection, etc.
* Ensure filesystem + database consistency when jobs modify files (move, delete, rename, commit) — so incomplete operations can be recovered or rolled back.
* Provide a stable, extensible, type-safe API for scheduling new job types, without needing global dedicated queue infrastructure (like Redis, external brokers) — embedded + self-contained.

In short: Job Manager = **the backbone of all asynchronous workflows** in your media library application.

---

## 2. Current Architecture: Worker Managers

### RatingManager (First Implementation)

The **RatingManager** serves as the first concrete implementation of a worker manager pattern. It demonstrates the architecture that will be used for all future worker managers.

**Location:** `engine/src/ratings/manager.rs`

**Architecture:**
```
RatingManager
  ├─ Kinopoisk Worker (tokio::spawn)
  ├─ TMDB Worker 1 - Metadata Fetcher (tokio::spawn)
  └─ TMDB Worker 2 - IMDb Scraper (tokio::spawn, depends on Worker 1)
```

**Key Features:**
- **Immediate Execution**: Workers spawn immediately when `check_and_fetch_ratings()` is called
- **Async Coordination**: Uses `tokio::spawn` and `tokio::try_join!` for parallel execution
- **Cache-Aware**: Checks cache before spawning workers
- **Error Handling**: Gracefully handles failures, continues with available data
- **Result Merging**: Combines worker results back into `EnrichedMedia`

**Worker Types:**
1. **Kinopoisk Worker**: Independent, fetches Kinopoisk ratings
2. **TMDB Worker 1**: Fetches TMDB metadata (episodes, seasons, external_id including imdb_id)
3. **TMDB Worker 2**: Waits for Worker 1, then scrapes IMDb ratings using imdb_id

**Integration:**
- Currently integrated into `Engine` struct
- Called automatically after metadata enrichment in search methods
- Returns updated `EnrichedMedia` with ratings filled in

**Future Migration Path:**
When persistent job queue is implemented, RatingManager can be refactored to:
- Enqueue rating fetch jobs instead of immediate execution
- Use JobManager for coordination
- Support retries, dependencies, and scheduling
- Persist jobs to database

### Other Managers in Codebase

**IndexerManager** (`engine/src/indexers/manager.rs`):
- Manages torrent indexer searches
- Uses semaphore for concurrency control
- Not a worker manager (different pattern)

**MetadataManager** (`engine/src/metadata/cache.rs`):
- Manages metadata provider coordination
- Caching layer for metadata
- Not a worker manager (different pattern)

**FilesystemManager** (`engine/src/filesystem/manager.rs`):
- Placeholder for filesystem operations
- Future candidate for worker manager pattern

## 3. Why Database-Backed Persistent Queue (SQLite / DB)

Based on similar projects and libraries:

* Using an embedded SQL database (e.g. SQLite) avoids external dependencies (no need for Redis, message broker, external queue) — simplifies distribution and deployment. This is especially beneficial for desktop / single-user applications. ([docs.deploystack.io][1])
* Persistent queue ensures that jobs survive restarts, crashes, or shutdowns. No job is lost, and operations can resume. ([softwareontheroad.com][2])
* With proper schema and transactional updates, queue operations (enqueue, claim, complete, fail) can be atomic and safe — reduces risk of corruption or inconsistent state.

Libraries such as Backlite (Rust / SQLite embedded job queue) show that such designs are viable, type-safe and production-ready. ([GitHub][3])

Given that your system already uses SQLite for metadata / library DB, integrating job queue into the same database (or a companion DB) keeps things simple and self-contained.

---

## 4. Core Schema / Data Model for Job Manager

Here is a recommended schema for the job queue table(s) and related metadata. This builds on what we already have, but with dedicated fields to support retries, dependencies, state transitions, logging, etc.

### Table: `jobs`

| Field          | Type                          | Description                                                                        |
| -------------- | ----------------------------- | ---------------------------------------------------------------------------------- |
| `job_id`       | TEXT (UUID) PRIMARY KEY       | Unique identifier of the job                                                       |
| `kind`         | TEXT                          | Job type / kind (e.g. `"CommitFile"`, `"CleanupStale"`, `"MetadataRefresh"`, etc.) |
| `params`       | JSON / TEXT                   | Serialized parameters for the job (arguments, context)                             |
| `status`       | TEXT                          | One of: `pending`, `waiting`, `working`, `succeeded`, `failed`, `cancelled`        |
| `priority`     | INTEGER                       | Priority value — higher = run earlier when scheduling                              |
| `attempts`     | INTEGER                       | Number of times this job has been attempted                                        |
| `max_attempts` | INTEGER (optional)            | How many times to retry (if retryable) before marking dead                         |
| `depends_on`   | TEXT (nullable)               | `job_id` of another job this one depends on (optional)                             |
| `scheduled_at` | INTEGER (timestamp)           | When job is eligible to run (for delayed jobs/backoff)                             |
| `created_at`   | INTEGER (timestamp)           | When job was created/enqueued                                                      |
| `started_at`   | INTEGER (timestamp, nullable) | When job execution began                                                           |
| `finished_at`  | INTEGER (timestamp, nullable) | When job execution ended (success or failure)                                      |
| `last_error`   | TEXT (nullable)               | Error message or serialized error info if failed                                   |
| `result`       | JSON / TEXT (optional)        | Optional result data or metadata from job execution                                |

### Optional / Auxiliary Tables

* **Job history / archive** — store completed or failed jobs for audit, metrics or debugging. Could use same `jobs` table (keep all) or a separate `jobs_history/archive` table.
* **Locks / Resource locks table** — especially for jobs that need to operate on shared resources (filesystem paths, files, disks), to prevent conflicting operations.
* **Rate-limiting / concurrency tracking table (optional)** — to track resource usage, throttling, quotas (e.g. maximum simultaneous filesystem-heavy jobs).

---

### Job Kinds (Examples)

Based on current and planned managers:

| Job Kind | Manager | Description | Priority | Retryable |
|----------|---------|-------------|----------|-----------|
| `FetchRatings` | RatingManager | Fetch Kinopoisk/IMDb ratings for movies | Medium | Yes |
| `FetchTMDBMetadata` | RatingManager | Fetch TMDB metadata (episodes, seasons, external_id) | Medium | Yes |
| `ScrapeIMDbRating` | RatingManager | Scrape IMDb rating using imdb_id | Medium | Yes |
| `CommitFile` | FilesystemManager | Move file from staging to final location | High | Yes |
| `HashFile` | FilesystemManager | Calculate file hash for verification | Low | Yes |
| `RefreshMetadata` | MetadataRefreshManager | Refresh metadata for library items | Low | Yes |
| `SyncLibrary` | LibrarySyncManager | Scan filesystem and sync with library DB | Medium | Yes |
| `CleanupTrash` | FilesystemManager | Remove old files from trash | Low | No |

## 5. Job Lifecycle & State Machine

Here’s a typical lifecycle flow for a job managed by Job Manager:

1. **Enqueue**

   * A part of application (UI command, download event, maintenance scheduler) creates a new job with `status = pending`, `created_at = now`, optional `priority`, `params`, etc.
2. **Waiting (dependency / scheduled delay)**

   * If job has `depends_on` or `scheduled_at` in future, its status may be `waiting` until dependent job finishes or scheduled time arrives.
3. **Ready / Pending**

   * Scheduler picks up pending jobs (whose dependencies are resolved and scheduled_at ≤ now), considers priority, resource availability, concurrency limits.
4. **Working / Running**

   * Worker claims the job (atomically), updates `status → working`, sets `started_at`.
   * Worker executes the job logic.
5. **Complete or Fail**

   * On success: set `status → succeeded`, `finished_at`, optional `result`.
   * On failure: if `attempts < max_attempts` and error classified retryable → increment `attempts`, compute backoff delay → set `scheduled_at = now + backoff`, `status → pending` (or `waiting`). If not retryable, or retries exhausted → `status → failed`, `last_error` set.
6. **Optional Post-processing**

   * For long-term history, optionally archive job to history table (or keep in jobs table), or mark for deletion/purge after some retention time.

This is aligned with common job-queue patterns (see Sidekiq-style retry + dead-letter queue, or general persistent-job-queue best practices). ([DEV Community][4])

---

## 6. Scheduler & Worker Architecture — How Jobs Are Executed

### Current Implementation: Immediate Execution (RatingManager)

**Pattern:**
```rust
// Immediate execution - no queue
pub async fn check_and_fetch_ratings(&self, movies: Vec<EnrichedMedia>) -> Result<Vec<EnrichedMedia>> {
    // Spawn workers immediately
    let kinopoisk_handle = tokio::spawn(...);
    let tmdb_handle = tokio::spawn(...);
    
    // Wait for completion
    let (kinopoisk_results, tmdb_results) = tokio::try_join!(kinopoisk_handle, tmdb_handle)?;
    
    // Merge results
    // ...
}
```

**Pros:**
- Simple, immediate execution
- No database overhead
- Good for on-demand operations

**Cons:**
- No persistence (lost on crash)
- No retry mechanism
- No scheduling/delayed execution
- No job dependencies

### Future Implementation: Persistent Queue

**Pattern:**
```rust
// Queue-based execution
pub async fn enqueue_rating_fetch(&self, movies: Vec<EnrichedMedia>) -> Result<Vec<JobId>> {
    let mut job_ids = Vec::new();
    
    for movie in movies {
        // Enqueue job
        let job_id = self.job_manager.enqueue(Job {
            kind: JobKind::FetchRatings,
            params: serde_json::to_value(movie)?,
            priority: 5,
            max_attempts: 3,
        }).await?;
        job_ids.push(job_id);
    }
    
    Ok(job_ids)
}
```

**Pros:**
- Jobs survive crashes
- Automatic retries
- Scheduling support
- Job dependencies
- Better observability

**Cons:**
- More complex
- Database overhead
- Requires job queue implementation

Your job manager should consist of two main parts:

* **Scheduler / Dispatcher** — module that scans the `jobs` table periodically (or is triggered on enqueue) to identify ready jobs (pending / scheduled_at ≤ now / dependencies resolved), locks/claims them and assigns to worker-groups.
* **Worker-Groups / Worker Pool** — actual execution threads/tasks that perform the job logic (filesystem ops, metadata fetch, cleanup, etc.), report back success/failure.

Key design aspects:

* Use a **thread pool or async-task pool** to cap maximum concurrency, prevent resource exhaustion. Thread pools are standard pattern for concurrency to avoid thread explosion & overhead. ([The Rust Programming Language Forum][5])
* Jobs must be **idempotent** where possible (or at least resilient) — because retries and failures may cause repeated execution (e.g. after crash) — a common requirement in persistent queues. ([MoldStud][6])
* Claiming a job must be **atomic**: only one worker can claim a job at a time. This means job status update (pending → working) should occur as part of a database transaction or lock to avoid race conditions / duplicate execution. Many reliable queues rely on DB-level locking or SELECT … FOR UPDATE / skip-locked patterns. ([GitHub][7])
* After job success/failure, the scheduler or worker must update job status and optionally trigger dependent jobs (if any) — e.g. metadata fetch after commit, cleanup after hash, etc.
* For resource-heavy jobs (filesystem operations, hashing, metadata fetch), you should consider **concurrency limits per job type / pool** to avoid saturating disk, CPU, network — especially since multiple job types may conflict (FS vs metadata vs network).

---

## 7. Retry, Backoff, Failure Handling, and Dead-Letter

Failures will happen (IO errors, network timeouts, corrupted files, permissions), so robust handling is critical:

* **Retry logic** with configurable `max_attempts`, exponential (or linear) backoff. On transient failures (e.g. network, temporary resource busy), re-queue job. On permanent failures (permission denied, invalid data), mark `failed`. This pattern is broadly recommended in job queue design. ([DEV Community][4])
* **Dead-letter queue or “failed jobs pool”** — keep failed jobs (after exhausted retries) for manual inspection or reprocessing.
* **Idempotent handlers** — each job handler should be designed so that repeated / retried execution does not corrupt state (e.g. file commit should check if file already moved, skip or resume; metadata refresh should merge gracefully).
* **Error logging & diagnostics** — store error messages, maybe stack traces or context, to debug failures. For filesystem-critical jobs, record before/after state to enable manual recovery if something goes wrong.

---

## 8. Dependencies Between Jobs & Job Chaining

### Current Implementation: Worker Dependencies (RatingManager)

**Example from RatingManager:**
```rust
// TMDB Worker 2 depends on TMDB Worker 1
let tmdb_metadata_results = tmdb_metadata_handle.await?;  // Wait for Worker 1
let imdb_handle = spawn_imdb_worker(imdb_ids_from_worker_1);  // Then spawn Worker 2
```

**Pattern:**
- Workers use `tokio::spawn` and `await` for dependencies
- Manual coordination in manager code
- No persistence of dependencies

### Future Implementation: Job Dependencies

**Example:**
```rust
// Job 2 depends on Job 1
let job1_id = job_manager.enqueue(Job {
    kind: JobKind::FetchTMDBMetadata,
    params: ...,
}).await?;

let job2_id = job_manager.enqueue(Job {
    kind: JobKind::ScrapeIMDbRating,
    params: ...,
    depends_on: Some(job1_id),  // Automatic dependency
}).await?;
```

**Benefits:**
- Dependencies stored in database
- Automatic scheduling when dependencies resolve
- Survives crashes
- Can chain complex workflows

For complex workflows (multiple sequential steps, e.g. download → commit → hash → metadata → dedupe), job dependencies are unavoidable. Job Manager should support:

* Specifying `depends_on: job_id` when creating a job — ensures that dependent job does not run before parent job succeeds.
* Optionally, `multiple dependencies` (or a small DAG) — could be implemented via multiple `depends_on` records or a separate dependency table. For simplicity, single dependency per job is acceptable, but for future-proofing, you may support array of dependencies.
* On job completion, automatically enqueue next job(s) or mark dependent job(s) as ready.

This aligns with the pattern of “job chaining” or workflow orchestration used in many background task systems.

---

## 9. Integration with Filesystem, Library & Other Modules — Safety & Consistency

Because many jobs will perform filesystem operations (file moves, renames, deletes, commit, cleanup), integration with filesystem and library metadata must be carefully coordinated:

* Use **transactions + locks**: when performing file operations + metadata/DB update, operations should be atomic — either both succeed or neither (with rollback or cleanup) — to avoid inconsistent state (file exists but DB missing, or vice versa).
* Use **locks or resource locking** before dispatching jobs that operate on the same resource (e.g. same file path, same directory) to avoid concurrent conflicting operations. Manager should coordinate locking.
* On application start or recovery: detect “in-flight / working” jobs (jobs with `status = working`) — treat them as failed or retry them, or check for partial state (e.g. file half committed) and resume or rollback. This ensures reliability after crashes.
* Ensure **idempotency** and **safe cleanup** — e.g. if job fails mid-copy, staging / temp files should be cleaned or reused; failed jobs should not leave orphaned files or partial writes.

---

## 10. Monitoring, Logging & Metrics

### Current Implementation: RatingManager Logging

**Logging Pattern:**
```rust
info!("Rating Manager: Checking ratings for {} movies", movies.len());
info!("Rating Manager: Found {} movies missing Kinopoisk ratings", missing.len());
info!("Rating Manager: Spawning Kinopoisk worker ({} movies)", movies.len());
info!("Kinopoisk Worker: Fetched {} ratings", results.len());
warn!("Rating Manager: Movie \"{}\" missing Kinopoisk rating", title);
```

**Metrics Tracked:**
- Number of movies checked
- Cache hits/misses
- Worker completion times
- Missing ratings (warnings)
- API errors

### Future: Job Queue Metrics

**Additional Metrics:**
- Job queue length (pending, working, failed)
- Job duration statistics
- Retry counts
- Failure rates per job kind
- Throughput (jobs/minute)

For maintainability and debugging, the Job Manager should provide visibility:

* Expose a **job queue inspector** (in UI or separate debug panel) showing: pending jobs, working jobs, failed jobs, history, durations, error counts.
* Track metrics: job durations, throughput (jobs per minute), failure rate, retry count, queue lengths — useful to detect bottlenecks or mis-behaving job types.
* Logging: record start/finish times, parameters, errors, resource usage; for filesystem jobs — record paths, before/after states, locks acquired/released.
* Optionally: support **job prioritization, queue triggers / alerts** — e.g. alert if too many failed jobs, or queue backlog growing too high.

---

## 11. Implementation Notes & Recommendations (especially in Rust + SQLite context)

* Use a **typed, generic job API**: define a `Job` trait or enum per job type (with parametrized payload), serialize params to JSON (or binary) for storage, deserialize when executing. This gives flexibility to support arbitrary job kinds (filesystem operations, metadata fetch, cleanup, maintenance). This is similar to frameworks described by jmmv.dev when creating persistent task queue in generic fashion. ([Julio Merino (jmmv.dev)][8])
* Use **thread pool or async runtime** (depending on workload) for worker-groups. Thread pools are a standard pattern for concurrency-heavy workloads. ([The Rust Programming Language Forum][5])
* Keep **transactions short** and **avoid long-running DB locks** — frequent long transactions or readers will degrade performance, especially under load.
* Ensure **job claiming is atomic and isolated** — use database transaction to mark job “working” and claim it, before releasing to worker. Avoid race conditions or duplicate execution.
* Support **retry/backoff + dead-letter** out of the box. Provide configuration per job kind for `max_attempts`, backoff strategy, maybe failure classification.
* Provide **job schema versioning / migration logic** — as job parameters or kinds evolve over time, ensure older job records remain valid or are migrated gracefully.
* For **resource-heavy or IO-intensive jobs** (file operations, hashing), consider limiting concurrency (e.g. only N jobs at once), to prevent disk or CPU overload.
* On startup/application initialization: scan DB for jobs with `status = working` (or locked) — treat them as failed or attempt recovery — to avoid stuck jobs after crash.

---

## 12. Example Job Manager + Schema (Markdown Collectible Example)

Here’s a sample minimal schema + pseudocode outline for job management (for your repo’s `job_manager.rs` / module):

```text
-- SQL Schema (simplified)
CREATE TABLE jobs (
  job_id          TEXT PRIMARY KEY,
  kind            TEXT NOT NULL,
  params          JSON NOT NULL,
  status          TEXT NOT NULL,
  priority        INTEGER NOT NULL DEFAULT 0,
  attempts        INTEGER NOT NULL DEFAULT 0,
  max_attempts    INTEGER NOT NULL DEFAULT 3,
  depends_on      TEXT NULL,
  scheduled_at    INTEGER NOT NULL,
  created_at      INTEGER NOT NULL,
  started_at      INTEGER NULL,
  finished_at     INTEGER NULL,
  last_error      TEXT NULL,
  result          JSON NULL
);
```

```rust
enum JobKind {
  CommitFile { staging_path: String, target_path: String },
  HashFile { path: String },
  MetadataFetch { media_id: Uuid },
  CleanupTrash { older_than: Duration },
  // ... other kinds
}

struct Job {
  id: Uuid,
  kind: JobKind,
  params: serde_json::Value,
  // ... metadata fields
}

// Pseudocode: scheduling a job
fn enqueue_job(kind: JobKind, priority: i32, depends_on: Option<Uuid>, scheduled_at: Option<SystemTime>) {
  let now_ts = now();
  let job = Job {
     id: Uuid::new_v4(),
     kind,
     params: serialize(kind),
     status: "pending",
     priority,
     attempts: 0,
     max_attempts: DEFAULT_MAX,
     depends_on,
     scheduled_at: scheduled_at.unwrap_or(now_ts),
     created_at: now_ts,
     // other fields null
  };
  db.insert(job)
}
```

Worker loop (simplified):

```text
loop {
   let job = fetch_and_claim_next_job();  // Atomic select + update to status=working
   match job {
     None => sleep(POLL_INTERVAL),
     Some(job) => {
       let result = execute_job(job.clone()); // run job logic
       if result.ok() {
         mark_job_succeeded(job.id);
       } else if job.attempts < job.max_attempts {
         schedule_retry(job.id, backoff_duration);
       } else {
         mark_job_failed(job.id, error_info);
       }
     }
   }
}
```

---

## 13. RatingManager Integration Example

### Current Architecture

```rust
// Engine integration
pub struct Engine {
    // ... other fields ...
    rating_manager: Option<Arc<RatingManager>>,
}

// Usage in search methods
pub async fn search(&self, query: &str) -> Result<Vec<MediaSearchResult>> {
    // ... normalization ...
    
    // Fetch ratings for all movies
    let results_with_ratings = self.fetch_ratings_for_results(results).await?;
    Ok(results_with_ratings)
}

async fn fetch_ratings_for_results(&self, mut results: Vec<MediaSearchResult>) -> Result<Vec<MediaSearchResult>> {
    if let Some(rating_manager) = &self.rating_manager {
        let enriched_media: Vec<EnrichedMedia> = results.iter().map(|r| r.enriched.clone()).collect();
        
        match rating_manager.check_and_fetch_ratings(enriched_media).await {
            Ok(updated_media) => {
                // Update results with ratings
                for (result, updated) in results.iter_mut().zip(updated_media.iter()) {
                    result.enriched.rating_kinopoisk = updated.rating_kinopoisk;
                    result.enriched.rating_imdb = updated.rating_imdb;
                }
            }
            Err(e) => {
                warn!("Rating Manager: Failed to fetch ratings: {}", e);
                // Continue without ratings - don't fail the whole search
            }
        }
    }
    Ok(results)
}
```

### Future: JobManager Integration

```rust
// Unified JobManager
pub struct JobManager {
    rating_manager: Arc<RatingManager>,
    download_manager: Arc<DownloadManager>,  // Future
    // ... other managers ...
    job_queue: Arc<JobQueue>,
}

impl JobManager {
    pub async fn process_ratings(&self, movies: Vec<EnrichedMedia>) -> Result<Vec<EnrichedMedia>> {
        // Option 1: Immediate execution (current)
        self.rating_manager.check_and_fetch_ratings(movies).await
        
        // Option 2: Queue-based (future)
        // self.enqueue_rating_jobs(movies).await
    }
    
    pub async fn enqueue_rating_jobs(&self, movies: Vec<EnrichedMedia>) -> Result<Vec<JobId>> {
        // Enqueue jobs for persistent execution
        // ...
    }
}
```

## 14. FAQs & Edge-Cases — What to Watch Out For

**Q: What if the application crashes while a job is “working”?**
A: On startup, detect jobs with `status = working` but no `finished_at` time (and possibly stale `started_at` timestamp). You can choose to: mark them failed (so dependent jobs aren’t blocked), optionally re-enqueue (if idempotent), or attempt recovery (depending on job kind).

**Q: How to prevent duplicate execution / race-conditions?**
A: Use atomic claim: in a DB transaction, select a pending job with appropriate conditions (scheduled_at ≤ now, dependencies resolved), update status → working, commit — ensuring only one worker gets it. Also ensure job handlers are idempotent or safe to retry.

**Q: What about resource-heavy jobs (disk I/O, hashing)?**
A: Limit concurrency per worker-group for those job kinds. Optionally track resource usage and throttle dynamically (e.g. do not run more than N hashing jobs simultaneously). Use separate worker-groups/pools per job category.

**Q: How to deal with schema evolution?**
A: Use migration scripts, backups, versioned schema. On DB open, check schema version; if mismatched — apply migrations; always backup DB before destructive changes.

**Q: What about job priority and starvation?**
A: Use a priority field. Scheduler should always pick highest-priority ready jobs. To prevent starvation of low-priority jobs, optionally implement **aging / priority boost** (increase priority over time) so long-waiting jobs eventually get executed. This is a common pattern in priority-queue schedulers. ([Medium][9])

---

## 15. Migration Path: From Immediate Execution to Persistent Queue

### Phase 1: Current (RatingManager)
- ✅ Immediate execution with `tokio::spawn`
- ✅ In-memory caching
- ✅ Error handling
- ❌ No persistence
- ❌ No retries
- ❌ No scheduling

### Phase 2: Unified JobManager (Next)
- Create `JobManager` that wraps `RatingManager`
- Add interface for other worker managers
- Keep immediate execution for now
- Prepare for queue integration

### Phase 3: Persistent Queue (Future)
- Implement database-backed job queue
- Migrate RatingManager to use queue
- Add retry, scheduling, dependencies
- Migrate other managers

### Migration Strategy

**Step 1:** Create JobManager interface
```rust
pub trait WorkerManager: Send + Sync {
    async fn process(&self, params: serde_json::Value) -> Result<serde_json::Value>;
    fn job_kind(&self) -> &str;
}
```

**Step 2:** Implement for RatingManager
```rust
impl WorkerManager for RatingManager {
    async fn process(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let movies: Vec<EnrichedMedia> = serde_json::from_value(params)?;
        let results = self.check_and_fetch_ratings(movies).await?;
        Ok(serde_json::to_value(results)?)
    }
    
    fn job_kind(&self) -> &str {
        "FetchRatings"
    }
}
```

**Step 3:** Integrate into JobManager
```rust
pub struct JobManager {
    workers: HashMap<String, Box<dyn WorkerManager>>,
    queue: Arc<JobQueue>,
}

impl JobManager {
    pub fn new() -> Self {
        let mut workers = HashMap::new();
        workers.insert("FetchRatings".to_string(), Box::new(RatingManager::new()?) as Box<dyn WorkerManager>);
        // ... other workers ...
        Self { workers, queue: Arc::new(JobQueue::new()) }
    }
}
```

## 16. Why This Design Matters — Benefits & Guarantees

* **Durability & Persistence**: jobs survive restarts, crashes; no work lost.
* **Correctness & Atomicity**: by using database transactions and careful state transitions, you avoid partial-work / inconsistent states when jobs modify filesystem + DB.
* **Scalability & Flexibility**: job kinds are generic, new jobs can be added easily; supports dependencies, retries, scheduling, background maintenance.
* **Observability & Maintainability**: job history, logs, metrics — you can monitor backlog, failures, performance; debug issues; provide user feedback.
* **Resource Safety & Stability**: by controlling concurrency, grouping jobs, limiting heavy workloads — you avoid overloading disk / CPU / network.
* **Simplicity & Minimal Dependencies**: relies only on SQLite + standard Rust concurrency primitives — no external broker required; easy to package and distribute.

---
