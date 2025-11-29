<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import TopBar from './components/TopBar.svelte';
  import TabBar from './components/TabBar.svelte';
  import MovieGrid from './components/MovieGrid.svelte';
  import DetailPanel from './components/DetailPanel.svelte';
  import LoadingSpinner from './components/LoadingSpinner.svelte';
  import type { UiSearchResult } from './types';

  let searchResults: UiSearchResult[] = [];
  let isLoading = false;
  let isInitialLoading = true;
  let searchQuery = '';
  let error: string | null = null;
  let selectedMovie: UiSearchResult | null = null;
  let currentTab = 'feed';
  let isFeedView = true;

  async function loadFeed() {
    console.log('🎬 Starting feed load...');
    
    // First test if IPC bridge works at all
    try {
      console.log('🧪 Testing IPC bridge...');
      const testResponse = await invoke('test_connection');
      console.log('✅ IPC test result:', testResponse);
    } catch (err) {
      console.error('❌ IPC bridge broken:', err);
      error = 'Мост IPC не работает';
      isInitialLoading = false;
      return;
    }
    
    isLoading = true;
    error = null;
    isFeedView = true;
    
    try {
      console.log('📡 Calling invoke get_feed...');
      const response = await invoke('get_feed');
      console.log('✅ Feed response received:', response);
      searchResults = (response as any).results || [];
      console.log(`📊 Loaded ${searchResults.length} movies in feed`);
    } catch (err) {
      console.error('❌ Feed load failed:', err);
      error = err as string;
    } finally {
      isLoading = false;
      isInitialLoading = false;
      console.log('🏁 Feed load completed, isLoading:', isLoading);
    }
  }

  async function handleSearch(query: string) {
    if (!query.trim()) return;
    
    isLoading = true;
    error = null;
    searchQuery = query;
    isFeedView = false;
    
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

  function handleMovieSelect(event: CustomEvent<UiSearchResult>) {
    selectedMovie = event.detail;
  }

  function handleCloseDetail() {
    selectedMovie = null;
  }

  function handleTabChange(event: CustomEvent<string>) {
    currentTab = event.detail;
    console.log('Tab changed to:', currentTab);
    
    // Load feed when switching back to feed tab
    if (currentTab === 'feed' && !isFeedView) {
      loadFeed();
    }
  }

  function handleDownload(event: CustomEvent<UiSearchResult>) {
    console.log('Download:', event.detail.title);
  }

  function handleBookmark(event: CustomEvent<UiSearchResult>) {
    console.log('Bookmark:', event.detail.title);
  }

  onMount(() => {
    // Listen for custom movie selection events
    document.addEventListener('movieSelect', handleMovieSelect as EventListener);
    
    // Load initial feed
    loadFeed();
  });
</script>

<div class="min-h-screen bg-black">
  {#if isInitialLoading}
    <!-- Initial loading state - prevents white screen -->
    <div class="flex items-center justify-center min-h-screen bg-black">
      <LoadingSpinner />
    </div>
  {:else}
    <!-- Full app content -->
    <!-- Netflix-style Top Bar -->
    <TopBar on:search={(e) => handleSearch(e.detail)} />
    
    <!-- Netflix-style Tab Bar -->
    <TabBar currentTab={currentTab} on:tabChange={handleTabChange} />

    <!-- Main Content -->
    <main class="container mx-auto px-4 py-8">
      {#if isLoading}
        <div class="flex justify-center items-center h-64">
          <!-- Loading state handled by MovieGrid with SkeletonTile -->
          <MovieGrid movies={[]} loading={true} />
        </div>
      {:else if error}
        <div class="text-center py-12">
          <div class="text-netflix-red text-lg mb-4">
            {isFeedView ? 'Ошибка загрузки ленты' : 'Ошибка поиска'}
          </div>
          <div class="text-netflix-light">{error}</div>
          <button 
            class="mt-4 px-4 py-2 bg-netflix-red text-white rounded hover:bg-red-600 transition-colors"
            on:click={() => isFeedView ? loadFeed() : handleSearch(searchQuery)}
          >
            Повторить
          </button>
        </div>
      {:else if searchResults.length === 0}
        <div class="text-center py-12">
          <div class="text-netflix-light text-lg mb-4">
            {isFeedView ? 'Нет фильмов в ленте' : (searchQuery ? 'Ничего не найдено' : 'Начните поиск фильмов')}
          </div>
          {#if searchQuery && !isFeedView}
            <div class="text-sm text-gray-500">
              Попробуйте другие ключевые слова или проверьте правописание
            </div>
          {:else if isFeedView}
            <div class="text-sm text-gray-500">
              Проверьте подключение или попробуйте позже
            </div>
          {/if}
        </div>
      {:else}
        <div class="mb-6">
          <h2 class="text-xl font-semibold text-netflix-white">
            {isFeedView ? 'Недавние фильмы' : `${searchResults.length} результатов по запросу "${searchQuery}"`}
          </h2>
          {#if isFeedView}
            <div class="text-sm text-gray-400 mt-1">
              Последние фильмы с Monna2
            </div>
          {/if}
        </div>
        <MovieGrid movies={searchResults} loading={false} />
      {/if}
    </main>

    <!-- Movie Detail Panel -->
    {#if selectedMovie}
      <DetailPanel 
        movie={selectedMovie} 
        isOpen={true}
        on:close={handleCloseDetail}
        on:download={handleDownload}
        on:bookmark={handleBookmark}
      />
    {/if}
  {/if}
</div>

<style>
  /* Additional styles if needed */
</style>
