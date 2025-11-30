# Agent Instructions

_Last updated: 2025-11-28_

These instructions are for any future agent working on this project. They summarize the current state, conventions, and the exact next steps for the Monna → genres → UI work.

---

## 1. Current feature focus: Monna genres → UI

We are in the middle of wiring **genres parsed from Monna** all the way through the backend to the Svelte UI and presenting them cleanly.

### 1.1 What is already implemented

#### Backend

- **File:** `engine/src/indexers/monna.rs`
  - `parse_metadata_from_html`:
    - Uses DOM selectors on `div.fullstory table tr td` to find the "Жанр" row and split genres.
    - Uses `BR_REGEX` to treat `<br>` as separators when parsing the genre cell.
    - Falls back to `GENRE_REGEX` if DOM-based extraction yields nothing.
    - Populates `MonnaMetadata.genres: Vec<String>`.
  - `parse_movie_list_with_metadata`:
    - Calls `parse_metadata_from_html` for each movie detail page.
    - After creating `TorrentResult`, it now sets:
      - `torrent_result.category = Some(category.to_string());`
      - `torrent_result.poster_url = metadata.poster_url.clone();`
      - `torrent_result.description = metadata.description.clone();`
      - `torrent_result.cast = metadata.cast.clone();`
      - `torrent_result.runtime_minutes = metadata.runtime_minutes;`
      - `torrent_result.genres = metadata.genres.clone();`
  - `fetch_movie_details`:
    - Builds a `metadata` map with a combined `"genre"` string.
    - Creates `TorrentResult` from the parsed detail page.
    - If `metadata.get("genre")` is present:
      - Sets `torrent_result.category = Some(genre.clone());`
      - Splits the string on commas to fill `torrent_result.genres`.

- **File:** `engine/src/models/torrent.rs`
  - `TorrentResult` struct includes:
    - `pub genres: Vec<String>,`
  - `TorrentResult::new` initializes `genres` with `Vec::new()`.

- **Note:** `EnrichedMedia` already has `genres: Vec<String>`, but you must ensure everything is wired from `TorrentResult` → `EnrichedMedia` → UI DTOs (see TODOs below).

#### UI

- **File:** `ui/src/components/MovieCard.svelte`
  - **Card overlay (hover)** currently shows:
    - Title (`cleanTitle`)
    - Year (derived from `movie.year` or from title)
    - Placeholder rating badge (`★ --`) for future TMDB integration
    - Download + Info buttons
  - **Detail modal** currently:
    - Uses `backdrop_url || poster_url` as header background with a gradient overlay.
    - Shows title + year in the header.
    - In the body:
      - Description (`movie.description`) as "Overview" (if present).
      - Full cast list (`movie.cast`) as "Cast" (if present).
      - Duration block showing `Xh Ym` if `movie.runtime_minutes` is set.
      - Torrent info block: size in GB, seeders, leechers.
      - Action buttons: "Download Torrent" and "Add to Watchlist".
      - Genre pills (colorful tags) **intended** to render from `movie.genres`.

- **ROADMAP.md** has a section:
  - `## Recent work (Nov 2025) — debugged / further implementation`
  - This documents that Monna + MovieCard work is **partially implemented**, not fully done.

---

## 2. Known issues / broken pieces

1. **Svelte syntax error in `MovieCard.svelte`:**
   - There is a lint error: `Expected if, each or await` around the genre section.
   - This is caused by attempted `{#const ...}` blocks or invalid control flow syntax inside `{#each movie.genres as genre}`.
   - Svelte does **not** support `{#const}`; that logic must be moved to helpers in `<script>`.

2. **`hasGenres` reactive variable missing:**
   - The code refers conceptually to `hasGenres`, but it was not fully wired.
   - We want:
     ```ts
     $: hasDescription = movie.description && movie.description.trim().length > 0;
     $: hasCast = movie.cast && movie.cast.length > 0;
     $: hasGenres = movie.genres && movie.genres.length > 0;
     ```

3. **Backend → UI mapping may be incomplete:**
   - `TorrentResult.genres` is set in Monna indexer, but we must ensure:
     - The engine / Tauri command layer converts `TorrentResult` (or `EnrichedMedia`) into `UiSearchResult` and includes `genres`.
   - If `UiSearchResult.genres` is not set, the UI will never see them.

4. **Runtime behavior not verified:**
   - No end-to-end check has confirmed that genres parsed from real Monna pages actually appear as pills in the detailed modal.
   - Old card overlay (with GB + seeders row) might still appear in other components (`MovieTile.svelte`) depending on what the app is using in `App.svelte`.

---

## 3. Exact next steps (please follow in order)

### Step 1: Fix genre pill rendering in `MovieCard.svelte`

Goal: make Svelte compile cleanly and display colored genre pills.

1. Open `ui/src/components/MovieCard.svelte`.
2. In the `<script>` block, add:
   ```ts
   $: hasGenres = movie.genres && movie.genres.length > 0;

   function genreClass(genre: string): string {
     const g = genre.toLowerCase();
     if (g.includes('ужас') || g.includes('horror') || g.includes('хоррор')) return 'bg-red-900/80 text-red-100 border border-red-700';
     if (g.includes('комед') || g.includes('comedy')) return 'bg-yellow-500/80 text-yellow-900 border border-yellow-600';
     if (g.includes('драм') || g.includes('drama')) return 'bg-blue-900/80 text-blue-100 border border-blue-700';
     if (g.includes('фантаст') || g.includes('sci-fi') || g.includes('фэнтез')) return 'bg-purple-900/80 text-purple-100 border border-purple-700';
     if (g.includes('боевик') || g.includes('action')) return 'bg-orange-600/80 text-white border border-orange-700';
     if (g.includes('триллер') || g.includes('thriller')) return 'bg-rose-900/80 text-rose-100 border border-rose-700';
     if (g.includes('криминал') || g.includes('crime')) return 'bg-gray-700/80 text-gray-100 border border-gray-600';
     if (g.includes('мелодрам') || g.includes('romance')) return 'bg-pink-600/80 text-pink-100 border border-pink-700';
     if (g.includes('приключ') || g.includes('adventure')) return 'bg-green-700/80 text-green-100 border border-green-600';
     if (g.includes('мульт') || g.includes('аним') || g.includes('cartoon') || g.includes('anime')) return 'bg-sky-600/80 text-sky-100 border border-sky-700';
     return 'bg-netflix-gray/50 text-white border border-gray-600';
   }
   ```
3. Replace the current genres `{#each}` block with something like:
   ```svelte
   {#if hasGenres}
     <div class="flex flex-wrap gap-2 mb-6">
       {#each movie.genres as genre}
         <span class={`px-3 py-1 text-xs font-medium rounded-full ${genreClass(genre)}`}>
           {genre}
         </span>
       {/each}
     </div>
   {/if}
   ```
4. Ensure there are **no** `{#const ...}` blocks left in the file.

### Step 2: Verify backend → UI `genres` wiring

1. Find the Rust type used by the UI: `UiSearchResult` (likely in `ui/src/types.ts` and Rust side in Tauri commands).
2. Confirm that `UiSearchResult` has a `genres: Vec<String>` / `string[]` field.
3. Find where search results are mapped for the UI (probably in `src-tauri/src/commands.rs` or `engine/src/engine.rs`):
   - Locate the function that returns a list of UI results to the frontend.
   - Ensure it copies `genres` from `EnrichedMedia` or `TorrentResult` into the UI struct.
4. If `genres` is missing anywhere in the chain, add it and keep the naming consistent.

### Step 3: Validate Monna indexer genres at runtime

1. Run the dev app (`npm run tauri dev` or equivalent).
2. Perform a Monna-based search that you know has genres on the site.
3. For a few results:
   - Open their detailed modal.
   - Confirm whether genre pills appear.
4. If not, log out `torrent_result.genres` on the Rust side (both in `parse_movie_list_with_metadata` and `fetch_movie_details`) to see if they are empty.
5. If `genres` are empty:
   - Re-check the Monna HTML in `/home/trash/Coding_Projects/Torrentflix/monna_sample.html`.
   - Adjust selectors or `BR_REGEX` handling if needed.

### Step 4: Clean up overlay inconsistencies

1. Identify which component is used in the main grid:
   - `MovieTile.svelte` vs `MovieCard.svelte`.
2. Ensure only **one** of them is the canonical tile; align its overlay with the design:
   - Only: title, year, (future) rating, and actions.
   - No raw GB/seeders row on the tile itself.
3. Keep torrent stats in the **detailed modal**, not on the hover overlay.

---

## 4. Style & constraints

- Do **not** rewrite the high-level architecture in `ROADMAP.md`.
- Preserve existing comments and docstrings; only add new ones if necessary.
- Match the existing Netflix-like dark theme and utility class style.
- Avoid new dependencies unless absolutely required.

If you follow the steps above, you should be able to:

1. Make Svelte compile cleanly again.
2. Get Monna genres flowing from HTML → Rust → UI.
3. See colored genre tags in the detailed movie modal.
4. Have a clean, minimal card overlay showing only title, year, and rating placeholder.
