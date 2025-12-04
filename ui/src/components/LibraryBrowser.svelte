<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import SkeletonTile from './SkeletonTile.svelte';
  import LoadingSpinner from './LoadingSpinner.svelte';

  interface MediaItem {
    id: number;
    title: string;
    normalized_title: string;
    original_title?: string;
    year?: number;
    media_type: 'movie' | 'series';
    overview?: string;
    genres?: string[];
    poster_url?: string;
    poster_path?: string;
    user_rating?: number;
    watch_status: 'unwatched' | 'in_progress' | 'watched';
    custom_notes?: string;
    created_at: string;
    updated_at: string;
  }

  interface LibraryFilters {
    title?: string;
    year?: number;
    resolution?: string;
    quality_label?: string;
    status?: string;
    tags?: string[];
    min_rating?: number;
    limit?: number;
    offset?: number;
    sort_by?: 'title' | 'year' | 'added_date' | 'size' | 'version_count' | 'rating';
    sort_order?: 'asc' | 'desc';
  }

  interface SearchOptions {
    fuzzy?: boolean;
    limit?: number;
    offset?: number;
  }

  // Component state
  let mediaItems: MediaItem[] = [];
  let isLoading = false;
  let error: string | null = null;
  let totalItems = 0;
  let currentPage = 0;
  const itemsPerPage = 20;

  // Filter state
  let viewMode: 'grid' | 'list' = 'grid';
  let searchQuery = '';
  let selectedTags: string[] = [];
  let minRating = 0;
  let sortBy: 'title' | 'year' | 'added_date' | 'size' | 'version_count' | 'rating' = 'title';
  let sortOrder: 'asc' | 'desc' = 'asc';
  let showFilters = false;

  // Load library on mount
  onMount(async () => {
    await loadLibrary();
  });

  async function loadLibrary() {
    isLoading = true;
    error = null;

    try {
      const filters: LibraryFilters = {
        limit: itemsPerPage,
        offset: currentPage * itemsPerPage,
        sort_by: sortBy,
        sort_order: sortOrder,
      };

      if (minRating > 0) {
        filters.min_rating = minRating;
      }

      if (selectedTags.length > 0) {
        filters.tags = selectedTags;
      }

      const items = await invoke<MediaItem[]>('query_library', { filters });
      mediaItems = items;
      totalItems = items.length;
    } catch (err) {
      error = `Failed to load library: ${err}`;
      console.error('Library load error:', err);
    } finally {
      isLoading = false;
    }
  }

  async function searchLibrary() {
    if (!searchQuery.trim()) {
      await loadLibrary();
      return;
    }

    isLoading = true;
    error = null;

    try {
      const options: SearchOptions = {
        fuzzy: true,
        limit: itemsPerPage,
        offset: currentPage * itemsPerPage,
      };

      const items = await invoke<MediaItem[]>('search_library', {
        query: searchQuery,
        options,
      });
      mediaItems = items;
      totalItems = items.length;
    } catch (err) {
      error = `Search failed: ${err}`;
      console.error('Search error:', err);
    } finally {
      isLoading = false;
    }
  }

  function handleSearch(event: Event) {
    const target = event.target as HTMLInputElement;
    searchQuery = target.value;
    currentPage = 0;
    if (searchQuery.length > 2) {
      searchLibrary();
    } else if (searchQuery.length === 0) {
      loadLibrary();
    }
  }

  function handleSort(newSort: typeof sortBy) {
    if (sortBy === newSort) {
      sortOrder = sortOrder === 'asc' ? 'desc' : 'asc';
    } else {
      sortBy = newSort;
      sortOrder = 'asc';
    }
    currentPage = 0;
    loadLibrary();
  }

  function toggleTag(tag: string) {
    if (selectedTags.includes(tag)) {
      selectedTags = selectedTags.filter((t) => t !== tag);
    } else {
      selectedTags = [...selectedTags, tag];
    }
    currentPage = 0;
    loadLibrary();
  }

  function clearFilters() {
    searchQuery = '';
    selectedTags = [];
    minRating = 0;
    sortBy = 'title';
    sortOrder = 'asc';
    currentPage = 0;
    loadLibrary();
  }

  function nextPage() {
    if ((currentPage + 1) * itemsPerPage < totalItems) {
      currentPage += 1;
      loadLibrary();
    }
  }

  function prevPage() {
    if (currentPage > 0) {
      currentPage -= 1;
      loadLibrary();
    }
  }

  function getWatchStatusBadge(status: string): string {
    switch (status) {
      case 'watched':
        return '✓ Watched';
      case 'in_progress':
        return '▶ In Progress';
      default:
        return 'Unwatched';
    }
  }

  function getWatchStatusColor(status: string): string {
    switch (status) {
      case 'watched':
        return 'bg-green-600';
      case 'in_progress':
        return 'bg-blue-600';
      default:
        return 'bg-gray-600';
    }
  }
</script>

<div class="library-browser w-full h-full flex flex-col bg-gray-900 text-white">
  <!-- Header -->
  <div class="bg-gray-800 border-b border-gray-700 p-4">
    <h1 class="text-2xl font-bold mb-4">Library</h1>

    <!-- Search Bar -->
    <div class="flex gap-2 mb-4">
      <input
        type="text"
        placeholder="Search library..."
        value={searchQuery}
        on:input={handleSearch}
        class="flex-1 px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
      />
      <button
        on:click={() => (showFilters = !showFilters)}
        class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition"
      >
        {showFilters ? '✕ Filters' : '⚙ Filters'}
      </button>
      <button
        on:click={() => (viewMode = viewMode === 'grid' ? 'list' : 'grid')}
        class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition"
      >
        {viewMode === 'grid' ? '≡ List' : '⊞ Grid'}
      </button>
    </div>

    <!-- Filters Panel -->
    {#if showFilters}
      <div class="bg-gray-700 p-4 rounded mb-4 space-y-4">
        <div>
          <label class="block text-sm font-semibold mb-2">Minimum Rating</label>
          <input
            type="range"
            min="0"
            max="10"
            step="0.5"
            bind:value={minRating}
            on:change={() => {
              currentPage = 0;
              loadLibrary();
            }}
            class="w-full"
          />
          <span class="text-sm text-gray-300">{minRating.toFixed(1)}</span>
        </div>

        <div>
          <label class="block text-sm font-semibold mb-2">Sort By</label>
          <div class="flex gap-2 flex-wrap">
            {#each ['title', 'year', 'added_date', 'size', 'version_count', 'rating'] as option}
              <button
                on:click={() => handleSort(option)}
                class={`px-3 py-1 rounded text-sm transition ${
                  sortBy === option
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-600 hover:bg-gray-500'
                }`}
              >
                {option.replace(/_/g, ' ')}
                {sortBy === option && (sortOrder === 'asc' ? '↑' : '↓')}
              </button>
            {/each}
          </div>
        </div>

        <button
          on:click={clearFilters}
          class="w-full px-4 py-2 bg-red-600 hover:bg-red-700 rounded transition"
        >
          Clear Filters
        </button>
      </div>
    {/if}
  </div>

  <!-- Content Area -->
  <div class="flex-1 overflow-auto p-4">
    {#if error}
      <div class="bg-red-900 border border-red-700 text-red-100 px-4 py-3 rounded mb-4">
        {error}
      </div>
    {/if}

    {#if isLoading}
      <div class="flex justify-center items-center h-full">
        <LoadingSpinner />
      </div>
    {:else if mediaItems.length === 0}
      <div class="flex justify-center items-center h-full text-gray-400">
        <div class="text-center">
          <p class="text-xl mb-2">No items found</p>
          <p class="text-sm">Try adjusting your filters or search query</p>
        </div>
      </div>
    {:else if viewMode === 'grid'}
      <!-- Grid View -->
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
        {#each mediaItems as item (item.id)}
          <div class="group cursor-pointer">
            <div class="relative bg-gray-800 rounded overflow-hidden aspect-video mb-2 hover:shadow-lg transition">
              {#if item.poster_path}
                <img
                  src={item.poster_path}
                  alt={item.title}
                  class="w-full h-full object-cover group-hover:opacity-75 transition"
                  loading="lazy"
                />
              {:else}
                <div class="w-full h-full bg-gray-700 flex items-center justify-center">
                  <span class="text-gray-500">No Image</span>
                </div>
              {/if}

              <!-- Overlay -->
              <div class="absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-60 transition flex items-center justify-center">
                <button class="opacity-0 group-hover:opacity-100 transition px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded">
                  View Details
                </button>
              </div>

              <!-- Badges -->
              <div class="absolute top-2 right-2 flex gap-1">
                {#if item.user_rating}
                  <div class="bg-yellow-600 px-2 py-1 rounded text-xs font-semibold">
                    ⭐ {item.user_rating.toFixed(1)}
                  </div>
                {/if}
                <div class={`${getWatchStatusColor(item.watch_status)} px-2 py-1 rounded text-xs font-semibold`}>
                  {getWatchStatusBadge(item.watch_status)}
                </div>
              </div>
            </div>

            <!-- Title -->
            <h3 class="font-semibold text-sm truncate group-hover:text-blue-400 transition">
              {item.title}
            </h3>
            {#if item.year}
              <p class="text-xs text-gray-400">{item.year}</p>
            {/if}
          </div>
        {/each}
      </div>
    {:else}
      <!-- List View -->
      <div class="space-y-2">
        {#each mediaItems as item (item.id)}
          <div class="bg-gray-800 hover:bg-gray-700 p-4 rounded flex items-center justify-between transition cursor-pointer">
            <div class="flex-1">
              <h3 class="font-semibold">
                {item.title}
                {#if item.year}
                  <span class="text-gray-400">({item.year})</span>
                {/if}
              </h3>
              {#if item.overview}
                <p class="text-sm text-gray-400 truncate">{item.overview}</p>
              {/if}
            </div>

            <div class="flex items-center gap-4 ml-4">
              {#if item.user_rating}
                <div class="text-yellow-400 font-semibold">⭐ {item.user_rating.toFixed(1)}</div>
              {/if}
              <div class={`${getWatchStatusColor(item.watch_status)} px-3 py-1 rounded text-sm font-semibold`}>
                {getWatchStatusBadge(item.watch_status)}
              </div>
              <button class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded transition">
                View
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Pagination -->
  {#if totalItems > itemsPerPage}
    <div class="bg-gray-800 border-t border-gray-700 p-4 flex items-center justify-between">
      <button
        on:click={prevPage}
        disabled={currentPage === 0}
        class="px-4 py-2 bg-gray-700 hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed rounded transition"
      >
        ← Previous
      </button>

      <span class="text-sm text-gray-400">
        Page {currentPage + 1} of {Math.ceil(totalItems / itemsPerPage)}
        ({totalItems} items)
      </span>

      <button
        on:click={nextPage}
        disabled={(currentPage + 1) * itemsPerPage >= totalItems}
        class="px-4 py-2 bg-gray-700 hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed rounded transition"
      >
        Next →
      </button>
    </div>
  {/if}
</div>

<style>
  .library-browser {
    display: flex;
    flex-direction: column;
  }
</style>
