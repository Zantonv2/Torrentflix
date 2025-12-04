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
  import SettingsTab from './components/SettingsTab.svelte';
  import LibraryBrowser from './components/LibraryBrowser.svelte';
  import MediaItemDetail from './components/MediaItemDetail.svelte';
  import CollectionManager from './components/CollectionManager.svelte';
  import StorageAnalytics from './components/StorageAnalytics.svelte';
  import ManagementPanel from './components/ManagementPanel.svelte';
  import type { UiSearchResult } from './types';

  let searchResults: UiSearchResult[] = [];
  let isLoading = false;
  let isInitialLoading = true;
  let searchQuery = '';
  let error: string | null = null;
  let selectedMovie: UiSearchResult | null = null;
  let currentTab = 'feed';
  let isFeedView = true;
  let showMediaDetail = false;
  let selectedMediaId: number | null = null;

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

  function handleCloseMediaDetail() {
    showMediaDetail = false;
    selectedMediaId = null;
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
    
    // Update individual ratings (don't overwrite with 0.0)
    if (update.rating_kinopoisk !== undefined && update.rating_kinopoisk !== null && update.rating_kinopoisk > 0) {
      movie.rating_kinopoisk = update.rating_kinopoisk;
    }
    if (update.rating_imdb !== undefined && update.rating_imdb !== null && update.rating_imdb > 0) {
      movie.rating_imdb = update.rating_imdb;
    }
    if (update.rating_tmdb !== undefined && update.rating_tmdb !== null && update.rating_tmdb > 0) {
      movie.rating_tmdb = update.rating_tmdb;
    }
    
    // Recalculate primary rating from ALL available ratings (prefer IMDb > Kinopoisk > TMDB)
    // TMDB often returns 0.0 for new movies, so prefer IMDb/Kinopoisk when available
    if (movie.rating_imdb !== undefined && movie.rating_imdb !== null && movie.rating_imdb > 0) {
      movie.rating = movie.rating_imdb;
    } else if (movie.rating_kinopoisk !== undefined && movie.rating_kinopoisk !== null && movie.rating_kinopoisk > 0) {
      movie.rating = movie.rating_kinopoisk;
    } else if (movie.rating_tmdb !== undefined && movie.rating_tmdb !== null && movie.rating_tmdb > 0) {
      movie.rating = movie.rating_tmdb;
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
  let initialized = false;

  // Use a more aggressive initialization approach
  let mountAttempts = 0;
  
  onMount(async () => {
    mountAttempts++;
    console.log(`🚀 App.svelte onMount() called (attempt ${mountAttempts})`);
    
    if (initialized) {
      console.log('⚠️ Already initialized, skipping');
      return;
    }
    initialized = true;
    
    document.addEventListener('movieSelect', handleMovieSelect as EventListener);
    
    try {
      console.log('📡 Setting up rating-update listener...');
      const unlisten = await listen<{ movie_id: string; rating_kinopoisk?: number; rating_imdb?: number; rating_tmdb?: number }>('rating-update', (event) => {
        handleRatingUpdate(event.payload);
      });
      ratingUpdateUnlisten = unlisten;
      console.log('✅ Rating-update listener set up');
    } catch (error) {
      console.error('❌ Failed to listen for rating-update events:', error);
    }
    
    // Failsafe: if loading takes more than 10 seconds, show error
    const failsafeTimeout = setTimeout(() => {
      if (isInitialLoading) {
        console.error('⏰ Failsafe triggered: Loading took too long');
        isInitialLoading = false;
        isLoading = false;
        error = 'Загрузка заняла слишком много времени. Попробуйте обновить страницу.';
      }
    }, 10000);
    
    // Add a small delay to ensure Tauri is fully ready
    console.log('⏳ Waiting 100ms for Tauri to be ready...');
    await new Promise(resolve => setTimeout(resolve, 100));
    
    console.log('🎬 Calling loadFeed()...');
    try {
      await loadFeed();
      console.log('✅ loadFeed() completed successfully');
    } catch (err) {
      console.error('❌ loadFeed() failed:', err);
      // Try one more time after a delay
      console.log('🔄 Retrying loadFeed() after 1 second...');
      await new Promise(resolve => setTimeout(resolve, 1000));
      await loadFeed();
    }
    clearTimeout(failsafeTimeout);
    console.log('✅ onMount() complete');
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
        {#if currentTab === 'library'}
          <div class="flex-1 overflow-hidden">
            <LibraryBrowser />
          </div>
        {:else if currentTab === 'history'}
          <div class="flex-1 overflow-hidden">
            <StorageAnalytics />
          </div>
        {:else if currentTab === 'bookmarks'}
          <div class="flex-1 overflow-hidden">
            <CollectionManager />
          </div>
        {:else if currentTab === 'management'}
          <div class="flex-1 overflow-hidden">
            <ManagementPanel />
          </div>
        {:else if currentTab === 'downloads'}
          <DownloadsTab />
        {:else if currentTab === 'settings'}
          <div class="flex-1 overflow-hidden flex flex-col">
            <SettingsTab />
          </div>
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

    <!-- Media Item Detail Modal -->
    {#if showMediaDetail && selectedMediaId}
      <MediaItemDetail 
        mediaId={selectedMediaId}
        onClose={handleCloseMediaDetail}
      />
    {/if}
  {/if}
</div>

<style>
  /* Ensure flex containers can scroll */
  :global(.flex-1) {
    min-height: 0;
  }
</style>
