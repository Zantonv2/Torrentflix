<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  import SearchBar from './components/SearchBar.svelte';
  import MovieGrid from './components/MovieGrid.svelte';
  import LoadingSpinner from './components/LoadingSpinner.svelte';
  import type { UiSearchResult } from './types';

  let searchResults: UiSearchResult[] = [];
  let isLoading = false;
  let searchQuery = '';
  let error: string | null = null;

  async function handleSearch(query: string) {
    if (!query.trim()) return;
    
    isLoading = true;
    error = null;
    searchQuery = query;
    
    try {
      const response = await invoke('search_movies', {
        request: {
          query: query.trim(),
          media_type: 'movie',
          limit: 50
        }
      });
      
      searchResults = (response as any).results || [];
    } catch (err) {
      error = err as string;
      console.error('Search failed:', err);
    } finally {
      isLoading = false;
    }
  }

  onMount(() => {
    // Initial load or welcome state
  });
</script>

<div class="min-h-screen bg-netflix-black">
  <!-- Header -->
  <header class="bg-netflix-dark border-b border-netflix-gray">
    <div class="container mx-auto px-4 py-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-4">
          <h1 class="text-2xl font-bold text-netflix-red">TorrentFlix</h1>
          <span class="text-netflix-light text-sm">Movie Discovery</span>
        </div>
        <SearchBar onSearch={handleSearch} />
      </div>
    </div>
  </header>

  <!-- Main Content -->
  <main class="container mx-auto px-4 py-8">
    {#if isLoading}
      <div class="flex justify-center items-center h-64">
        <LoadingSpinner />
      </div>
    {:else if error}
      <div class="text-center py-12">
        <div class="text-netflix-red text-lg mb-4">Search Error</div>
        <div class="text-netflix-light">{error}</div>
        <button 
          class="mt-4 px-4 py-2 bg-netflix-red text-white rounded hover:bg-red-600 transition-colors"
          on:click={() => handleSearch(searchQuery)}
        >
          Retry
        </button>
      </div>
    {:else if searchResults.length === 0}
      <div class="text-center py-12">
        <div class="text-netflix-light text-lg mb-4">
          {searchQuery ? 'No results found' : 'Search for movies to get started'}
        </div>
        {#if searchQuery}
          <div class="text-sm text-gray-500">
            Try different keywords or check spelling
          </div>
        {/if}
      </div>
    {:else}
      <div class="mb-6">
        <h2 class="text-xl font-semibold text-netflix-white">
          {searchResults.length} results for "{searchQuery}"
        </h2>
      </div>
      <MovieGrid movies={searchResults} />
    {/if}
  </main>
</div>

<style>
  /* Additional styles if needed */
</style>
