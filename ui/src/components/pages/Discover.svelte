<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { t } from "svelte-i18n";
  import { searchStore } from "../../stores/searchStore";
  import { handleError, handleSuccess } from "../../utils/errorHandler";
  import MovieGrid from "../MovieGrid.svelte";
  import DetailPanel from "../DetailPanel.svelte";
  import type { UiSearchResult } from "../../types";

  let selectedMovie: UiSearchResult | null = null;
  let isLoading = false;
  let error: string | null = null;

  // Subscribe to search store
  let results: UiSearchResult[] = [];
  let storeLoading = false;

  const unsubscribe = searchStore.subscribe((state) => {
    results = state.results;
    storeLoading = state.loading;
  });

  async function loadFeed() {
    console.log("🎬 Discover: loadFeed() called");
    isLoading = true;
    error = null;

    try {
      // Wait for Tauri to be ready (check if __TAURI_INTERNALS__ exists)
      let retries = 0;
      while (!window.__TAURI_INTERNALS__ && retries < 50) {
        console.log("⏳ Discover: Waiting for Tauri to initialize...", retries);
        await new Promise((resolve) => setTimeout(resolve, 100));
        retries++;
      }

      if (!window.__TAURI_INTERNALS__) {
        throw new Error("Tauri failed to initialize");
      }

      console.log("🎬 Discover: Invoking get_feed command");
      const response = await invoke<any>("get_feed");
      console.log("🎬 Discover: get_feed response:", response);

      // Response is UiSearchResponse with { results, total, took_ms }
      const feedResults = response?.results || [];
      console.log("🎬 Discover: feedResults count:", feedResults.length);

      if (feedResults.length > 0) {
        console.log("🎬 Discover: First result:", feedResults[0]);
      }

      searchStore.setResults(feedResults);
      console.log("🎬 Discover: Results set in store");
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      console.error("❌ Discover: Feed error:", err);
      error = errorMsg;
      searchStore.setError(errorMsg);
      handleError(errorMsg, {
        command: "get_feed",
        userMessage: "Failed to load feed",
      });
    } finally {
      isLoading = false;
    }
  }

  function handleMovieSelect(event: CustomEvent<UiSearchResult>) {
    selectedMovie = event.detail;
  }

  function handleCloseDetail() {
    selectedMovie = null;
  }

  async function handleDownload(event: CustomEvent<UiSearchResult>) {
    const movie = event.detail;

    if (!movie.torrent_info?.magnet_link) {
      handleError("Magnet link not available", {
        command: "start_download",
        userMessage: "Magnet link not available",
      });
      return;
    }

    try {
      await invoke("start_download", {
        magnetLink: movie.torrent_info.magnet_link,
        title: movie.title,
      });
      handleSuccess("Download started");
      console.log("✅ Download started:", movie.title);
      selectedMovie = null;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      handleError(errorMsg, {
        command: "start_download",
        userMessage: `Download failed: ${errorMsg}`,
      });
      console.error("Download failed:", err);
    }
  }

  function handleBookmark(event: CustomEvent<UiSearchResult>) {
    console.log("Bookmark:", event.detail.title);
  }

  onMount(() => {
    console.log("🎬 Discover: Component mounted, calling loadFeed()");
    loadFeed();
    return () => {
      unsubscribe();
    };
  });
</script>

<div class="flex flex-1 overflow-hidden" style="min-height: 0; height: 100%;">
  <!-- Main content -->
  <div class="flex-1 flex flex-col overflow-hidden">
    {#if isLoading && results.length === 0}
      <MovieGrid
        movies={[]}
        loading={true}
        on:movieSelect={handleMovieSelect}
      />
    {:else if error}
      <div class="flex-1 flex items-center justify-center overflow-y-auto">
        <div class="text-center py-12">
          <div class="text-netflix-red text-lg mb-4">Error loading feed</div>
          <div class="text-netflix-light">{error}</div>
          <button
            class="mt-4 px-4 py-2 bg-netflix-red text-white rounded hover:bg-red-600 transition-colors"
            on:click={loadFeed}
            aria-label="Retry"
          >
            Retry
          </button>
        </div>
      </div>
    {:else if results.length === 0}
      <div class="flex-1 flex items-center justify-center overflow-y-auto">
        <div class="text-center py-12">
          <div class="text-netflix-light text-lg mb-4">No movies found</div>
          <div class="text-sm text-gray-500">Connection error</div>
        </div>
      </div>
    {:else}
      <MovieGrid
        movies={results}
        loading={storeLoading}
        on:movieSelect={handleMovieSelect}
      />
    {/if}
  </div>

  <!-- Detail panel (slides in) -->
  {#if selectedMovie}
    <DetailPanel
      movie={selectedMovie}
      isOpen={true}
      on:close={handleCloseDetail}
      on:download={handleDownload}
      on:bookmark={handleBookmark}
    />
  {/if}
</div>

<style>
  :global(.netflix-red) {
    color: #e50914;
  }

  :global(.netflix-light) {
    color: #ffffff;
  }
</style>
