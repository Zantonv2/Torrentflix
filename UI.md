# In-Depth UI Analysis — MovieDownloader (Tauri + Svelte / Solid friendly)

Below is a thorough, practical, implementation-ready analysis of the UI layer: responsibilities, architecture, state patterns, IPC, performance engineering, UX details, accessibility, testing, developer workflows, telemetry, packaging, and an actionable rollout checklist. Treat this as the canonical UI spec you can hand to frontend engineers and designers.

---

## 1 — High-level responsibilities & constraints

**Primary role:** present the domain model from the Rust engine and offer user controls (search, download, rename, delete, library management).
**Must not:** perform heavy parsing/dedupe/FS operations.
**Constraints / goals:**

* Extremely low latency and smooth animation on weak hardware
* Minimal memory use and fast cold start
* Single authoritative source of truth: engine/DB; UI mirrors it
* Offline-capable for cached data
* Accessible and localizable
* Easy to test, maintain, and extend

---

## 2 — Overall architecture & data flow

```
User ⇄ UI (Tauri WebView) ⇄ Tauri Commands (invoke) ⇄ Rust Engine
                     ⇡                            ⇣
                Events (event emit / subscribe)    Background updates
```

* **Commands (invoke)**: short-lived actions (search, download, rename, apply cleanup).
* **Events**: streaming/long-running updates (download progress, library-changed, job updates).
* **DTO contract**: UI only uses small, stable DTOs (UiSearchResult, UiLibraryItem, UiDownloadStatus, UiJob). Version them.

---

## 3 — Component model & decomposition

Design components bottom-up (atomic → composite → pages).

### Atomic components

* `Button`, `Icon`, `Badge`, `Pill`, `Tooltip`, `Input`, `Select`, `Avatar`, `Spinner`, `Skeleton`, `Modal`, `Toast`.

### Small composites

* `Poster` (image + placeholder + fallback)
* `QualityBadge` (1080p, 4K, codec icon)
* `TorrentInfoRow` (seeders, size, source)
* `ProgressIndicator` (linear + percentage + ETA)

### Page components

* `SearchPage` (SearchBar, Filters, ResultsGrid)
* `ResultsGrid` (virtualized cards)
* `DetailsPane` (metadata + actions)
* `DownloadManager` (queue + per-download control)
* `LibraryView` (filters, sort, grid/list toggle)
* `CleanupView` (candidate list + preview)
* `Settings` (indexers config, clients, keys)
* `DevConsole` (logs, indexer health)

---

## 4 — State management patterns

**Principle:** UI stores ephemeral local state only; authoritative state from engine.

### Two tiers of state

1. **Server state:** library pages, search results, download statuses — always fetched from engine and subscribed via events.
2. **Local UI state:** UI-only (open modals, current filter selections, pagination cursors, optimistic updates).

### Implementation approach

* Use lightweight Svelte stores (or Solid signals) for server state caches.
* Keep a small in-memory LRU cache of DTOs to avoid repeated IPC roundtrips.
* For long-running jobs, use job IDs and incremental updates (poll/subscribe).
* Use derived stores for computed values (e.g., filtered library subset).

### Immutable DTOs

Treat engine DTOs as immutable — updates come via events or fresh invokes. This simplifies concurrency.

---

## 5 — IPC & communication patterns

### Commands (one-off, request/response)

* `engine.search(query, opts) -> UiSearchResult[]`
* `engine.start_download(quality_id) -> { job_id }`
* `engine.get_library(page, filters) -> UiLibraryPage`

**Best practices**

* Keep payloads small; pass IDs, not whole objects.
* Commands return job id for long tasks; UI subscribes to job events.

### Events (streaming)

* `event.download-progress` { download_id, progress, speed, eta }
* `event.job-update` { job_id, status, progress, message }
* `event.library-changed` { media_item_id, change_type }
* `event.indexer-health` { indexer_name, score }

**Delivery**

* Use Tauri `emit` + `listen` per window.
* Deduplicate events in UI (coalesce updates to avoid heavy re-renders).

---

## 6 — Rendering & performance engineering

### Goals

* Sub-16ms frame updates when interacting
* Low CPU when idle
* Fast initial paint & first contentful paint

### Techniques

**Virtualization**

* Use virtualization for lists/grids (windowing) — only render visible cards.
* Keep card height predictable; variable-height layouts complicate virtualization.

**Image handling**

* Provide multiple poster sizes (thumb → medium → large). Use LQIP placeholders (tiny blurred data URIs).
* Lazy load with `IntersectionObserver`. Preload items just before scroll into view.
* Cache posters in engine (file cache) and serve via `file://` or small HTTP server Tauri can host.

**Avoid unnecessary re-renders**

* Use `key`ed lists and pure components.
* Keep components granular; only update components whose props changed.
* Coalesce frequent events (download progress) — e.g., throttle UI updates to ~12–20Hz for progress bars.

**CPU-bound work**

* Offload heavy UI compute (sorting huge arrays for display) to worker threads or do it in Rust engine (preferable).
* Avoid complex array operations on the main thread for large datasets.

**Animations**

* Use `transform` + `opacity` (GPU friendly).
* Avoid layout-triggering animations on large lists.

**Memory**

* Keep DOM count low. Reuse nodes in lists if possible.

---

## 7 — Images, caching & network

**Poster caching**

* Engine should download and cache posters to disk; UI requests `get_poster(media_id)` and receives `file://` path.
* For network fallback, UI can fetch remote CDN but engine served cached poster is best.

**Cache control**

* Use TTLs (e.g., 7 days) for posters; allow manual purge.
* Use ETags/If-Modified for remote fetches if engine fetches posters.

**Placeholder images**

* Use small inline SVG or base64 LQIP for immediate paint.

---

## 8 — UX & microinteraction details

**Search**

* As-you-type suggestions (local + TMDB) using debounce (150–250ms).
* Allow `Enter` to submit; keyboard navigation through suggestions.

**Results**

* Card hover shows quick actions: download, watchlist, details.
* Right-click context menu for advanced actions (copy magnet, open folder, mark watched).

**Downloads**

* Show per-download row with action icons (pause, open folder, open client, view files).
* Progress bars that animate smoothly; show speed, ETA, peers.

**Library**

* Toggle grid/list; show sort options, filters (genre, year, quality).
* Bulk actions: select many items, delete / export / pin.

**Cleanup flow**

* Preview with reasons and recovered space estimate.
* Provide safe defaults, confirm dialog, undo (trash with grace period).

**Notifications**

* Lightweight toasts + Desktop notifications.
* Configurable channels (Desktop, Telegram, webhook).

**Error handling**

* Friendly messages with suggested actions and “retry” button.
* For fatal operations (delete), require explicit confirmation with checkbox.

---

## 9 — Accessibility (A11y)

**Principles**

* Keyboard-first navigation, focus management
* Screen reader friendly labels and semantics
* Contrast ratios compliant with WCAG AA
* Logical tab order and skip-links

**Implementation**

* Use `aria-*` attributes on dynamic controls.
* Provide accessible names for poster cards (title + year).
* Ensure modal focus trap and return focus after close.
* Test with keyboard-only navigation and a screen reader (NVDA/VoiceOver).

---

## 10 — Theming & Localization

**Theming**

* Support light/dark modes with CSS variables.
* Provide an accent color selector.
* Use Tailwind theme variables or design tokens to centralize colors/spacing.

**Localization**

* All strings from engine and UI should be internationalized (i18n).
* Use ICU-style pluralization (svelte-i18n or similar).
* Handle RTL languages: layout flip and mirroring of icons where appropriate.
* Keep language preference in engine config and allow UI override.

---

## 11 — Offline behavior & resilience

* Show cached search results / library when offline.
* Display offline banner and graceful fallback for failed operations.
* Offer “retry” for failed network operations.
* Queue actions that can be retried when network returns (e.g., fetching metadata).

---

## 12 — Security & data privacy

* Never store API keys in plaintext; use OS keyring when possible (Tauri supports secure storage).
* Validate user inputs (no path traversal).
* Avoid eval/string template usage for CSS or HTML.
* Sanitize any HTML content displayed from external sources before injecting (prefer no raw HTML).

---

## 13 — Testing & QA

**Unit tests**

* Components (render + interactions)
* Helper utilities (formatters, size converters)

**Integration tests**

* Flow tests (search → add to queue → download progress)
* Mock engine to simulate events and error cases

**E2E**

* Use Playwright to test app flows in instrumented Tauri builds.
* Test on target OSes (Windows/macOS/Linux) for UI scale/fallbacks.

**Accessibility tests**

* Run axe audits in CI for critical components.

**Performance tests**

* Measure FPS during large scroll, memory while idle, time-to-first-paint.
* Stress test with large library (10k+ items) using headless UI harness.

---

## 14 — Developer experience & tooling

* **Local mock engine:** a small Node or local Rust mock server that exposes the same commands/events. This speeds UI dev without running the full engine.
* **Storybook/Component catalog:** develop components in isolation.
* **TypeScript + DTO types:** share DTO TS definitions from a generation script or copy from engine schema to keep types consistent.
* **Linting & formatting:** ESLint, Prettier, Tailwind linter.
* **CI pipelines:** run unit tests, accessibility checks, build test for Tauri packaging.

---

## 15 — Packaging for Tauri & resource budgets

**Resource budgets (example)**

* Initial memory target: ≤ 150MB on startup (including WebView)
* Idle CPU: < 2% on average hardware
* Startup time: FCP under 300ms ideally, < 1s acceptable
* Build artifact size: keep JS/CSS < 2–3 MB combined where possible

**Packaging notes**

* Tree-shake and purge CSS (Tailwind JIT purge).
* Avoid heavy runtime libraries; prefer compiled effects (Svelte).
* Use Vite for fast builds and HMR.

---

## 16 — Observability & telemetry

* Local logs panel for errors and indexer health.
* Optional opt-in telemetry: anonymized crash reports, feature usage, error rates.
* Metrics exposed by engine for UI to visualize: job queue, indexer latencies, download rates.

---

## 17 — Animation & micro-interaction system

* Prefer motion for affordance (hover elevation, soft transitions).
* Use a small, consistent motion scale across app (e.g., 120ms for micro transitions, 240ms for major).
* For large reflows use fade + translate instead of height anims.
* Add microinteractions for successes (toasts), failures (shake), and confirmations.

---

## 18 — Accessibility & keyboard shortcuts (essential set)

**Global**

* `Ctrl/Cmd+K` → quick search
* `Ctrl/Cmd+,` → settings
* `Space` or `P` → pause/resume selected download
* `Delete` → delete item (with confirmation modal)
* `Arrow keys` → navigate grid/list
* `Enter` → open details

Provide a shortcuts help modal and allow remapping.

---

## 19 — Error & fallback UX patterns

* **Transient errors**: show a toast with “Retry”.
* **Blocking errors (disk full)**: modal with clear instructions and direct CTA to cleanup tool.
* **Indexers failing**: show a small badge on indexer list; allow “disable until next app start”.
* **Long running jobs**: progress modal + background job system; show ETA if available.

---

## 20 — Performance measurement & profiling guide

**Measure**

* TTI, FCP, TTFB (for any engine-served content)
* Time to interactive (first search response)
* Frame rate while scrolling
* Memory snapshots

**Tools**

* DevTools Performance tab
* Playwright + performance tracing
* Lighthouse for web-app style checks (optional)
* OS profilers (macOS Instruments, Windows PerfView) for end-to-end

**Where to profile**

* Image-heavy grid scrolling
* Massive library load & filter/sort operations
* Heavy concurrent events (many downloads updating simultaneously)

---

## 21 — Implementation roadmap (practical steps)

1. Create DTO spec & TypeScript interfaces (from engine schemas).
2. Scaffold Vite + Svelte app with Tailwind JIT. Add Tauri template.
3. Implement mock engine for UI dev + storybook.
4. Build atomic components and style system (tokens).
5. Implement ResultsGrid with virtualization + lazy images.
6. Implement search flow + indexing of results (use real engine later).
7. Implement Downloads UI + event handling for progress.
8. Implement Library view + filters + pagination.
9. Implement Cleanup flow & DeleteCandidate UI.
10. Add accessibility, keyboard shortcuts, and settings.
11. Add logs/DevConsole and health panel.
12. Run performance tests (10k library), optimize.
13. Package Tauri builds, test cross-platform.
14. Add CI for unit/E2E/axe tests and bundle checks.

---

## 22 — Checklist (must-have before release)

* [ ] DTOs and IPC versioning in place
* [ ] Virtualized grid for search & library
* [ ] Poster caching strategy implemented (engine + UI)
* [ ] Throttled/coalesced event handling for progress updates
* [ ] Accessibility audit passed (keyboard + screen reader)
* [ ] Offline/cached fallback behavior implemented
* [ ] Safe delete/undo & audit log wired to UI
* [ ] Health & developer console exposed
* [ ] Tailwind purge & small CSS bundle verified
* [ ] CI runs tests + builds for target OSes

---

## 23 — Final practical tips / gotchas

* **Coalesce frequent events**: progress updates at 100Hz → throttle to 12–20Hz for UI. Keep engine publishing raw data if needed.
* **Small DTOs**: avoid serializing heavy fields; always pass IDs then fetch details if user opens a panel.
* **Image thrash**: load only necessary poster variants; avoid loading many large images at once.
* **Use the engine for heavy sorting / filtering**: don’t sort 100k items in the UI; request paged results.
* **Dev mode**: ship a debug toggle that connects to a mock engine or enables verbose logs — speeds development.
* **Keep UI deterministic**: side effects only from engine events — makes testing easy.

---