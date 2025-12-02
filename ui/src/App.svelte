<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import TopBar from './components/TopBar.svelte';
  import TabBar from './components/TabBar.svelte';
  import MovieGrid from './components/MovieGrid.svelte';
  import DetailPanel from './components/DetailPanel.svelte';
  import LoadingSpinner from './components/LoadingSpinner.svelte';
  import DownloadsTab from './components/DownloadsTab.svelte';
  import type { UiSearchResult } from './types';

  let searchResults: UiSearchResult[] = [];
  let isLoading = false;
  let isInitialLoading = true;
  let searchQuery = '';
  let error: string | null = null;
  let selectedMovie: UiSearchResult | null = null;
  let currentTab = 'feed';
  let isFeedView = true;

  async function loadFeed(retryCount = 0) {
    console.log('🎬 loadFeed() called, attempt:', retryCount + 1);
    isLoading = true;
    error = null;
    isFeedView = true;
    
    try {
      console.log('📡 Calling invoke(get_feed)...');
      const response = await invoke('get_feed');
      console.log('✅ Feed response:', response);
      searchResults = (response as any).results || [];
      console.log('📊 Loaded', searchResults.length, 'results');
    } catch (err) {
      console.error('❌ Feed error:', err);
      
      // Retry up to 3 times with exponential backoff
      if (retryCount < 3) {
        const delay = Math.pow(2, retryCount) * 500; // 500ms, 1s, 2s
        console.log(`⏳ Retrying in ${delay}ms...`);
        setTimeout(() => loadFeed(retryCount + 1), delay);
        return; // Don't set isLoading to false yet
      }
      
      error = err as string;
    } finally {
      if (retryCount >= 3 || searchResults.length > 0) {
        isLoading = false;
        isInitialLoading = false;
        console.log('🏁 loadFeed() complete, isLoading:', isLoading, 'isInitialLoading:', isInitialLoading);
      }
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
    
    // Load feed when switching back to feed tab
    if (currentTab === 'feed' && !isFeedView) {
      loadFeed();
    }
  }

  async function handleDownload(event: CustomEvent<UiSearchResult>) {
    const movie = event.detail;
    console.log('Download:', movie.title);
    
    if (!movie.torrent_info?.magnet_link) {
      alert('Нет magnet-ссылки для этого фильма');
      return;
    }
    
    try {
      const hash = await invoke('start_download', {
        magnetLink: movie.torrent_info.magnet_link,
        title: movie.title
      });
      console.log('✅ Download started:', hash);
      alert(`Загрузка начата: ${movie.title}`);
      
      // Switch to downloads tab
      currentTab = 'downloads';
    } catch (err) {
      console.error('❌ Download failed:', err);
      alert(`Ошибка загрузки: ${err}`);
    }
  }

  function handleBookmark(event: CustomEvent<UiSearchResult>) {
    console.log('Bookmark:', event.detail.title);
  }

  function handleRatingUpdate(update: { movie_id: string; rating_kinopoisk?: number; rating_imdb?: number; rating_tmdb?: number }) {
    console.log('⭐ Rating update received:', update);
    const movieIndex = searchResults.findIndex(m => m.id === update.movie_id);
    
    if (movieIndex === -1) {
      console.warn('⚠️ Movie not found:', update.movie_id);
      return;
    }
    
    const movie = searchResults[movieIndex];
    console.log('📝 Before update:', { 
      title: movie.title, 
      kp: movie.rating_kinopoisk, 
      imdb: movie.rating_imdb, 
      rating: movie.rating 
    });
    
    // Update ratings
    if (update.rating_kinopoisk !== undefined) {
      movie.rating_kinopoisk = update.rating_kinopoisk;
    }
    if (update.rating_imdb !== undefined) {
      movie.rating_imdb = update.rating_imdb;
    }
    
    // Update primary rating (prefer TMDB > IMDb > Kinopoisk)
    if (update.rating_tmdb !== undefined && update.rating_tmdb !== null && update.rating_tmdb > 0) {
      movie.rating = update.rating_tmdb;
    } else if (movie.rating_imdb !== undefined && movie.rating_imdb !== null) {
      movie.rating = movie.rating_imdb;
    } else if (movie.rating_kinopoisk !== undefined && movie.rating_kinopoisk !== null) {
      movie.rating = movie.rating_kinopoisk;
    }
    
    console.log('✅ After update:', { 
      title: movie.title, 
      kp: movie.rating_kinopoisk, 
      imdb: movie.rating_imdb, 
      rating: movie.rating 
    });
    
    // Trigger Svelte reactivity
    searchResults = [...searchResults];
    
    // Update selectedMovie if needed
    if (selectedMovie && selectedMovie.id === update.movie_id) {
      selectedMovie = { ...selectedMovie, ...movie };
    }
  }

  let ratingUpdateUnlisten: (() => void) | null = null;

  onMount(async () => {
    document.addEventListener('movieSelect', handleMovieSelect as EventListener);
    
    try {
      const unlisten = await listen<{ movie_id: string; rating_kinopoisk?: number; rating_imdb?: number; rating_tmdb?: number }>('rating-update', (event) => {
        handleRatingUpdate(event.payload);
      });
      ratingUpdateUnlisten = unlisten;
    } catch (error) {
      console.error('Failed to listen for rating-update events:', error);
    }
    
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

<div class="flex flex-col bg-netflix-dark-bg" style="height: 100vh; overflow: hidden;">
  {#if isInitialLoading}
    <div class="flex items-center justify-center h-screen bg-netflix-dark-bg">
      <LoadingSpinner />
    </div>
  {:else}
    <!-- Full app content -->
    <!-- Netflix-style Top Bar -->
    <TopBar on:search={(e) => handleSearch(e.detail)} />
    
    <!-- Main Content -->
    <div class="flex flex-1 overflow-hidden" style="min-height: 0; height: calc(100vh - 70px);">
      <!-- Left Side: Tab Bar + Content -->
      <div class="flex-1 flex flex-col overflow-hidden" style="min-height: 0;">
        <TabBar currentTab={currentTab} on:tabChange={handleTabChange} />
        
        <!-- Simple switch based on currentTab only -->
        {#if currentTab === 'downloads'}
          <DownloadsTab />
        {:else if currentTab === 'feed'}
          {#if isLoading && searchResults.length === 0}
            <MovieGrid movies={[]} loading={true} on:movieSelect={handleMovieSelect} />
          {:else if error}
            <div class="flex-1 flex items-center justify-center overflow-y-auto">
              <div class="text-center py-12">
                <div class="text-netflix-red text-lg mb-4">
                  {searchQuery ? 'Ошибка поиска' : 'Ошибка загрузки ленты'}
                </div>
                <div class="text-netflix-light">{error}</div>
                <button 
                  class="mt-4 px-4 py-2 bg-netflix-red text-white rounded hover:bg-red-600 transition-colors"
                  on:click={() => searchQuery ? handleSearch(searchQuery) : loadFeed()}
                >
                  Повторить
                </button>
              </div>
            </div>
          {:else if searchResults.length === 0}
            <div class="flex-1 flex items-center justify-center overflow-y-auto">
              <div class="text-center py-12">
                <div class="text-netflix-light text-lg mb-4">
                  {searchQuery ? 'Ничего не найдено' : 'Нет фильмов в ленте'}
                </div>
                {#if searchQuery}
                  <div class="text-sm text-gray-500">
                    Попробуйте другие ключевые слова или проверьте правописание
                  </div>
                {:else}
                  <div class="text-sm text-gray-500">
                    Проверьте подключение или попробуйте позже
                  </div>
                {/if}
              </div>
            </div>
          {:else}
            <MovieGrid movies={searchResults} loading={isLoading} on:movieSelect={handleMovieSelect} />
          {/if}
        {:else}
          <!-- Other tabs -->
          <div class="flex-1 flex items-center justify-center">
            <div class="text-center py-12">
              <div class="text-6xl mb-4">🚧</div>
              <p class="text-xl text-gray-400 mb-2">В разработке</p>
              <p class="text-sm text-gray-500">Эта вкладка скоро будет готова</p>
            </div>
          </div>
        {/if}
      </div>

      <!-- Right Side: Detail Panel (slides in) -->
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
  {/if}
</div>

<style>
  /* Ensure flex containers can scroll */
  :global(.flex-1) {
    min-height: 0;
  }
</style>
