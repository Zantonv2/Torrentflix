<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
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

  function handleRatingUpdate(update: { movie_id: string; rating_kinopoisk?: number; rating_imdb?: number; rating_tmdb?: number }) {
    console.log('⭐ Rating update received:', update);
    console.log('⭐ Current searchResults length:', searchResults.length);
    console.log('⭐ Looking for movie_id:', update.movie_id);
    console.log('⭐ Available movie IDs (first 10):', searchResults.map(m => m.id).slice(0, 10));
    
    // Find the movie in searchResults and update its ratings
    const movieIndex = searchResults.findIndex(m => m.id === update.movie_id);
    console.log('⭐ Movie index found:', movieIndex);
    
    if (movieIndex === -1) {
      console.warn(`⚠️ Movie with id "${update.movie_id}" not found in searchResults`);
      console.warn('⚠️ Full list of IDs:', searchResults.map(m => `"${m.id}"`));
      return;
    }
    
    if (movieIndex !== -1) {
      const movie = searchResults[movieIndex];
      
      // Update individual ratings
      if (update.rating_kinopoisk !== undefined) {
        movie.rating_kinopoisk = update.rating_kinopoisk;
      }
      if (update.rating_imdb !== undefined) {
        movie.rating_imdb = update.rating_imdb;
      }
      
      // Update primary rating (fallback: TMDB -> IMDb -> Kinopoisk)
      // Note: rating_tmdb is not stored separately, it's used to calculate primary rating
      if (update.rating_tmdb !== undefined && update.rating_tmdb !== null) {
        // TMDB has highest priority
        movie.rating = update.rating_tmdb;
      } else if (movie.rating_imdb !== undefined && movie.rating_imdb !== null) {
        // IMDb is second priority
        movie.rating = movie.rating_imdb;
      } else if (movie.rating_kinopoisk !== undefined && movie.rating_kinopoisk !== null) {
        // Kinopoisk is third priority
        movie.rating = movie.rating_kinopoisk;
      }
      
      // Trigger reactivity by reassigning the array
      searchResults = [...searchResults];
      
      // Also update selectedMovie if it's the same movie
      if (selectedMovie && selectedMovie.id === update.movie_id) {
        selectedMovie = { ...selectedMovie, ...movie };
      }
      
      console.log(`✅ Updated ratings for movie ${update.movie_id}:`, {
        kinopoisk: movie.rating_kinopoisk,
        imdb: movie.rating_imdb,
        tmdb: update.rating_tmdb,
        primary: movie.rating
      });
    } else {
      console.warn(`⚠️ Movie with id ${update.movie_id} not found in searchResults`);
    }
  }

  let ratingUpdateUnlisten: (() => void) | null = null;

  onMount(async () => {
    // Listen for custom movie selection events
    document.addEventListener('movieSelect', handleMovieSelect as EventListener);
    
    // Listen for rating-update events from Tauri
    try {
      console.log('🔍 Setting up rating-update event listener...');
      const unlisten = await listen<{ movie_id: string; rating_kinopoisk?: number; rating_imdb?: number; rating_tmdb?: number }>('rating-update', (event) => {
        console.log('🎯 EVENT RECEIVED in frontend:', event);
        console.log('🎯 Event payload:', event.payload);
        console.log('🎯 Event payload type:', typeof event.payload);
        console.log('🎯 Event payload keys:', Object.keys(event.payload || {}));
        handleRatingUpdate(event.payload);
      });
      ratingUpdateUnlisten = unlisten;
      console.log('✅ Listening for rating-update events - listener is active');
      console.log('✅ Unlisten function:', typeof unlisten);
    } catch (error) {
      console.error('❌ Failed to listen for rating-update events:', error);
      console.error('❌ Error details:', JSON.stringify(error));
    }
    
    // Load initial feed
    loadFeed();
  });

  onDestroy(() => {
    // Clean up event listeners
    document.removeEventListener('movieSelect', handleMovieSelect as EventListener);
    if (ratingUpdateUnlisten) {
      ratingUpdateUnlisten();
    }
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
