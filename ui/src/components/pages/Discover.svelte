<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { t } from "svelte-i18n";
  import {
    searchStore,
    getCachedResults,
    setScrollPosition,
  } from "../../stores/searchStore";
  import { settingsStore } from "../../stores/settingsStore";
  import { handleError, handleSuccess } from "../../utils/errorHandler";
  import MovieGrid from "../MovieGrid.svelte";
  import DetailPanel from "../DetailPanel.svelte";
  import EmptyState from "../core/EmptyState.svelte";
  import type { UiSearchResult } from "../../types";

  let selectedMovie: UiSearchResult | null = null;
  let isLoading = false;
  let error: string | null = null;
  let scrollContainer: HTMLDivElement | null = null;
  let isLoadingMore = false;

  // Subscribe to search store
  let results: UiSearchResult[] = [];
  let storeLoading = false;
  let unsubscribe: (() => void) | null = null;
  let settingsUnsubscribe: (() => void) | null = null;

  // Pagination state
  let paginationMode: "infinite" | "button" = "infinite";
  let hasMoreResults = true;
  let currentPage = 1;

  // Cache invalidation timer
  let cacheInvalidationTimer: NodeJS.Timeout | null = null;

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

      searchStore.setResults(feedResults, "feed");
      hasMoreResults = feedResults.length > 0;
      currentPage = 1;

      // Set up cache invalidation timer (10 minutes)
      if (cacheInvalidationTimer) {
        clearTimeout(cacheInvalidationTimer);
      }
      cacheInvalidationTimer = setTimeout(
        () => {
          console.log("🎬 Discover: Cache invalidated after 10 minutes");
          searchStore.clearCache();
        },
        10 * 60 * 1000,
      );

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

  async function loadMore() {
    if (isLoadingMore || !hasMoreResults) return;

    console.log("🎬 Discover: loadMore() called, page:", currentPage + 1);
    isLoadingMore = true;

    try {
      const response = await invoke<any>("get_feed", {
        page: currentPage + 1,
      });

      const moreResults = response?.results || [];
      if (moreResults.length === 0) {
        hasMoreResults = false;
        return;
      }

      // Append to existing results
      results = [...results, ...moreResults];
      searchStore.setResults(results, "feed");
      currentPage++;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      console.error("❌ Discover: Load more error:", err);
      handleError(errorMsg, {
        command: "get_feed",
        userMessage: "Failed to load more results",
      });
    } finally {
      isLoadingMore = false;
    }
  }

  function handleMovieSelect(event: CustomEvent<UiSearchResult>) {
    selectedMovie = event.detail;
  }

  function handleCloseDetail() {
    selectedMovie = null;
  }

  async function handleDownload(
    event: CustomEvent<{ movie: UiSearchResult; quality?: any }>,
  ) {
    const { movie, quality } = event.detail;

    // Use selected quality's magnet link if available, otherwise use default
    const magnetLink = quality?.magnet_link || movie.torrent_info?.magnet_link;

    if (!magnetLink) {
      handleError("Magnet link not available", {
        command: "start_download",
        userMessage: "Magnet link not available",
      });
      return;
    }

    try {
      const qualityLabel = quality ? ` (${quality.resolution})` : "";
      await invoke("start_download", {
        magnetLink,
        title: movie.title,
      });
      handleSuccess(`Download started${qualityLabel}`);
      console.log("✅ Download started:", movie.title, qualityLabel);
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

  function handleRefresh() {
    console.log("🎬 Discover: Manual refresh triggered");
    searchStore.clearCache();
    loadFeed();
  }

  function handleScroll(event: Event) {
    const target = event.target as HTMLDivElement;
    setScrollPosition(target.scrollTop);

    // Infinite scroll: load more when 200px from bottom
    if (paginationMode === "infinite" && !isLoadingMore && hasMoreResults) {
      const scrollHeight = target.scrollHeight;
      const scrollTop = target.scrollTop;
      const clientHeight = target.clientHeight;

      if (scrollHeight - scrollTop - clientHeight < 200) {
        loadMore();
      }
    }
  }

  onMount(() => {
    console.log("🎬 Discover: Component mounted, calling loadFeed()");

    // Subscribe to search store in onMount to avoid initialization issues
    unsubscribe = searchStore.subscribe((state) => {
      results = state.results;
      storeLoading = state.loading;
    });

    // Subscribe to settings store for pagination mode
    settingsUnsubscribe = settingsStore.subscribe((state) => {
      paginationMode = state.data.paginationMode;
    });

    // Try to restore from cache first
    const cachedResults = getCachedResults("feed");
    if (cachedResults && cachedResults.length > 0) {
      console.log("🎬 Discover: Restoring from cache");
      searchStore.setResults(cachedResults, "feed");
    } else {
      loadFeed();
    }

    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
      if (settingsUnsubscribe) {
        settingsUnsubscribe();
      }
      if (cacheInvalidationTimer) {
        clearTimeout(cacheInvalidationTimer);
      }
    };
  });
</script>

<div
  class="flex flex-1 overflow-hidden page-transition"
  style="min-height: 0; height: 100%;"
>
  <!-- Main content -->
  <div class="flex-1 flex flex-col overflow-hidden">
    <!-- Header with refresh button -->
    <div class="flex items-center justify-between p-4 border-b border-gray-700">
      <h1 class="text-2xl font-bold text-white">Discover</h1>
      <button
        class="px-4 py-2 bg-netflix-red text-white rounded hover:bg-red-600 transition-colors"
        on:click={handleRefresh}
        disabled={isLoading}
        aria-label="Refresh feed"
      >
        🔄 Refresh
      </button>
    </div>

    <!-- Content area with scroll -->
    <div
      class="flex-1 overflow-y-auto"
      bind:this={scrollContainer}
      on:scroll={handleScroll}
    >
      {#if isLoading && results.length === 0}
        <MovieGrid
          movies={[]}
          loading={true}
          on:movieSelect={handleMovieSelect}
        />
      {:else if error}
        <div class="flex-1 flex items-center justify-center">
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
        <EmptyState
          icon="🎬"
          title="No movies found"
          description="Try searching for a different movie or check your connection"
          actionLabel="Refresh"
          actionIcon="🔄"
          onAction={handleRefresh}
        />
      {:else}
        <MovieGrid
          movies={results}
          loading={storeLoading}
          on:movieSelect={handleMovieSelect}
        />

        <!-- Load More Button (if button pagination mode) -->
        {#if paginationMode === "button" && hasMoreResults}
          <div class="flex items-center justify-center py-8">
            <button
              class="px-6 py-3 bg-netflix-red text-white rounded hover:bg-red-600 transition-colors disabled:opacity-50"
              on:click={loadMore}
              disabled={isLoadingMore}
              aria-label="Load more movies"
            >
              {isLoadingMore ? "Loading..." : "Load More"}
            </button>
          </div>
        {/if}

        <!-- End of results message -->
        {#if !hasMoreResults && results.length > 0}
          <div class="flex items-center justify-center py-8">
            <div class="text-center text-gray-500">
              <p>End of results</p>
            </div>
          </div>
        {/if}

        <!-- Loading indicator for infinite scroll -->
        {#if paginationMode === "infinite" && isLoadingMore}
          <div class="flex items-center justify-center py-8">
            <div class="text-gray-500">Loading more...</div>
          </div>
        {/if}
      {/if}
    </div>
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
