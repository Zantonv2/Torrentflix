<script>
  import { createEventDispatcher } from 'svelte';
  import { fly } from 'svelte/transition';
  import { t } from '../i18n.js';
  import { selectedMovie } from '../stores/movies';
  import { invoke } from '@tauri-apps/api/core';

  const dispatch = createEventDispatcher();

  $: movie = $selectedMovie;
  $: posterUrl = movie?.monna2_poster_url || movie?.tmdb_poster_url || '';
  $: backdropUrl = movie?.tmdb_backdrop_url || '';
  
  // Debug ratings
  $: if (movie) {
    console.log('🔍 DEBUG DETAIL: Full movie object received:', movie);
    console.log('🔍 DEBUG DETAIL: Monna2 metadata fields:', {
      poster_url: movie.monna2_poster_url,
      description: movie.monna2_description,
      description_length: movie.monna2_description?.length || 0,
      genres: movie.monna2_genres,
      genres_count: movie.monna2_genres?.length || 0,
      cast: movie.monna2_cast,
      cast_count: movie.monna2_cast?.length || 0,
      director: movie.monna2_director,
      year: movie.monna2_year,
      runtime: movie.monna2_runtime
    });
    console.log('Movie ratings:', {
      kinopoisk: movie.kinopoisk_rating,
      imdb: movie.imdb_rating,
      all_keys: Object.keys(movie).filter(k => k.includes('rating') || k.includes('imdb') || k.includes('kinopoisk'))
    });
  }

  function handleClose() {
    dispatch('close');
  }

  // Bookmark functionality - uses global settings from Bookmarks tab
  async function handleBookmark() {
    if (!movie) return;
    
    console.log('🔧 DEBUG: handleBookmark called for movie:', movie.title);
    
    try {
      // Use global bookmark settings (will be fetched from backend)
      console.log('🔧 DEBUG: Calling add_bookmark with movie data:', {
        title: movie.title,
        year: movie.year,
        tmdb_id: movie.tmdb_id,
        imdb_id: movie.imdb_id,
        poster_url: posterUrl,
        genres: movie.genres
      });
      
      const bookmark = await invoke('add_bookmark', {
        movie: {
          title: movie.title,
          year: movie.year,
          tmdb_id: movie.tmdb_id,
          imdb_id: movie.imdb_id,
          poster_url: posterUrl,
          genres: movie.genres
        },
        preferences: null  // Use global default settings from backend
      });
      
      console.log('✅ DEBUG: Bookmark added successfully:', bookmark);
      // TODO: Show success notification
      
    } catch (error) {
      console.error('❌ DEBUG: Failed to add bookmark:', error);
      alert(`Ошибка при добавлении закладки: ${error}`);
      // TODO: Show error notification
    }
  }

  async function handleDownload(quality) {
    // Prioritize actual magnet link over torrent file URL
    const magnetLink = movie.magnet_link || movie.torrent_file_url || movie.link;
    
    // Find the selected quality option to get correct size
    let selectedSize = null;
    let selectedSizeBytes = null;
    
    console.log('🔥 DEBUG: movie.quality_options:', movie.quality_options);
    console.log('🔥 DEBUG: Looking for quality:', quality);
    
    if (movie.quality_options && Array.isArray(movie.quality_options)) {
      console.log('🔥 DEBUG: quality_options is array with', movie.quality_options.length, 'items');
      movie.quality_options.forEach((q, i) => {
        console.log(`🔥 DEBUG: Option ${i}: quality="${q.quality}", size="${q.size}", size_bytes=${q.size_bytes}`);
      });
      
      const qualityOption = movie.quality_options.find(q => q.quality === quality);
      console.log('🔥 DEBUG: Found quality option:', qualityOption);
      
      if (qualityOption) {
        selectedSize = qualityOption.size;
        selectedSizeBytes = qualityOption.size_bytes;
        console.log('🔥 DEBUG: Extracted size:', selectedSize, 'bytes:', selectedSizeBytes);
      } else {
        console.log('🔥 DEBUG: No quality option found for:', quality);
      }
    } else {
      console.log('🔥 DEBUG: quality_options is not an array or is null:', typeof movie.quality_options, movie.quality_options);
    }
    
    console.log('🔥 DEBUG: Movie object before record_download:', {
      title: movie.title,
      quality: quality,
      size: selectedSize,
      size_bytes: selectedSizeBytes,
      seeders: movie.seeders,
      leechers: movie.leechers,
      infohash: movie.infohash
    });
    
    try {
      // Start download via qBittorrent
      const success = await invoke('download_torrent', {
        magnetLink: magnetLink,
        savePath: null,  // Use default qBittorrent path
        rename: null     // Use original filename
      });
      
      if (success) {
        console.log('✅ DEBUG: Download started successfully');
        
        // Record download in history
        try {
          await invoke('record_download', {
            title: movie.title,
            quality: quality,
            year: movie.year,
            tmdb_id: movie.tmdb_id,
            size_bytes: selectedSizeBytes,  // Use size from quality option
            size_human: selectedSize,        // Use size from quality option
            seeders: movie.seeders || 0,
            leechers: movie.leechers || 0,
            source: "Monna2",
            infohash: movie.infohash,
            magnetLink: magnetLink,
            genres: movie.genres || null,
            category: movie.category || null
          });
          console.log('✅ DEBUG: Download recorded in history');
        } catch (historyErr) {
          console.error('❌ DEBUG: Failed to record download:', historyErr);
        }
      } else {
        console.error('❌ DEBUG: Download failed to start');
      }
    } catch (downloadErr) {
      console.error('❌ DEBUG: Download error:', downloadErr);
    }
  }

  function copyMagnetLink() {
    const magnetLink = movie.link || movie.magnet_link;
    if (magnetLink) {
      navigator.clipboard.writeText(magnetLink);
      alert('Магнит-ссылка скопирована!');
    }
  }

  let showFullDescription = false;
  let showCast = false;

  // Genre colors
  const genreColors = {
    'Боевик': 'bg-red-500/20 text-red-300',
    'Драма': 'bg-blue-500/20 text-blue-300',
    'Комедия': 'bg-yellow-500/20 text-yellow-300',
    'Триллер': 'bg-purple-500/20 text-purple-300',
    'Ужасы': 'bg-red-700/20 text-red-400',
    'Фантастика': 'bg-cyan-500/20 text-cyan-300',
    'Приключения': 'bg-green-500/20 text-green-300',
    'Мультфильм': 'bg-pink-500/20 text-pink-300',
  };

  function getGenreColor(genre) {
    return genreColors[genre] || 'bg-gray-500/20 text-gray-300';
  }
</script>

{#if movie}
  <div 
    class="w-[500px] bg-[#181818]/98 backdrop-blur-2xl border-l border-netflix-border flex flex-col overflow-hidden shadow-2xl"
    transition:fly={{ x: 600, duration: 350, easing: 'cubic-bezier(0.4, 0, 0.2, 1)' }}
  >
    <!-- Hero Section with Backdrop -->
    <div class="relative h-[300px] overflow-hidden">
      {#if backdropUrl}
        <img
          src={backdropUrl}
          alt={movie.title}
          class="w-full h-full object-cover"
        />
        <div class="absolute inset-0 bg-gradient-to-t from-[#181818] via-[#181818]/60 to-transparent"></div>
      {:else if posterUrl}
        <!-- Use poster as backdrop if no backdrop available -->
        <img
          src={posterUrl}
          alt={movie.title}
          class="w-full h-full object-cover blur-md scale-110 opacity-50"
        />
        <div class="absolute inset-0 bg-gradient-to-t from-[#181818] via-[#181818]/70 to-[#181818]/40"></div>
      {:else}
        <div class="w-full h-full bg-gradient-to-br from-netflix-red/20 via-netflix-card-hover to-[#181818]"></div>
      {/if}
      
      <!-- Close Button -->
      <button
        on:click={handleClose}
        class="absolute top-4 right-4 text-white/60 hover:text-white text-2xl transition-colors z-10"
      >
        ✕
      </button>

      <!-- Poster + Title Overlay -->
      <div class="absolute bottom-0 left-0 right-0 p-6 flex items-end gap-4">
        {#if posterUrl}
          <img
            src={posterUrl}
            alt={movie.title}
            class="w-[140px] h-[210px] object-cover rounded-lg shadow-2xl border-2 border-white/10 flex-shrink-0"
          />
        {/if}
        <div class="flex-1 flex flex-col justify-end pb-1">
          <h2 class="text-white text-3xl font-bold mb-2 drop-shadow-lg">{movie.cleaned_title || movie.title}</h2>
          
          <!-- Metadata -->
          <div class="flex items-center gap-3 text-sm text-white/80">
            {#if movie.monna2_year || movie.tmdb_year}
              <span class="flex items-center gap-1">📅 {movie.monna2_year || movie.tmdb_year}</span>
            {/if}
            {#if movie.monna2_runtime}
              <span class="flex items-center gap-1">⏱️ {movie.monna2_runtime}м</span>
            {/if}
            {#if movie.seeders}
              <span class="flex items-center gap-1">🌱 {movie.seeders}</span>
            {/if}
          </div>
          
          <!-- Ratings -->
          {#if movie.kinopoisk_rating || movie.imdb_rating}
            <div class="flex items-center gap-3 mt-2">
              {#if movie.kinopoisk_rating}
                <div class="flex items-center gap-1 bg-yellow-500/20 px-2 py-1 rounded">
                  <span class="text-yellow-400">⭐</span>
                  <span class="text-white font-bold">{movie.kinopoisk_rating.toFixed(1)}</span>
                  <span class="text-white/60 text-xs">KP</span>
                </div>
              {/if}
              {#if movie.imdb_rating}
                <div class="flex items-center gap-1 bg-yellow-500/20 px-2 py-1 rounded">
                  <span class="text-yellow-400">⭐</span>
                  <span class="text-white font-bold">{movie.imdb_rating.toFixed(1)}</span>
                  <span class="text-white/60 text-xs">IMDb</span>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Sticky Action Buttons -->
    <div class="sticky top-0 z-10 bg-[#181818]/95 backdrop-blur-sm border-b border-white/10 p-4 flex gap-2">
      <button
        on:click={() => handleDownload('1080p')}
        class="flex-1 bg-netflix-red hover:bg-netflix-red-hover text-white font-bold py-3 rounded transition-colors flex items-center justify-center gap-2"
      >
        <span>▶️</span>
        <span>Скачать</span>
      </button>
      
      <button
        on:click={handleBookmark}
        class="px-4 bg-white/10 hover:bg-white/20 text-white font-bold py-3 rounded transition-colors"
        title="Добавить в закладки с настройками по умолчанию"
      >
        ⭐
      </button>
      
      <button
        on:click={copyMagnetLink}
        class="px-4 bg-white/10 hover:bg-white/20 text-white font-bold py-3 rounded transition-colors"
        title="Копировать магнит-ссылку"
      >
        🔗
      </button>
    </div>

    <!-- Scrollable Content -->
    <div class="flex-1 overflow-y-auto px-6 pb-6">

      <!-- Genres with Colors -->
      {#if movie.monna2_genres && movie.monna2_genres.length > 0}
        <div class="mb-6 mt-4">
          <div class="flex flex-wrap gap-2">
            {#each movie.monna2_genres as genre}
              <span class="{getGenreColor(genre)} px-3 py-1.5 rounded-full text-sm font-medium">
                {genre}
              </span>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Description -->
      {#if movie.monna2_description || movie.tmdb_overview}
        <div class="mb-6">
          <h3 class="text-white font-bold mb-2">{t('overview')}</h3>
          <p class="text-netflix-text-secondary text-sm leading-relaxed" class:line-clamp-4={!showFullDescription}>
            {movie.monna2_description || movie.tmdb_overview}
          </p>
          {#if (movie.monna2_description || movie.tmdb_overview).length > 200}
            <button 
              on:click={() => showFullDescription = !showFullDescription}
              class="text-netflix-red text-sm mt-2 hover:underline"
            >
              {showFullDescription ? t('showLess') : t('showMore')}
            </button>
          {/if}
        </div>
      {/if}

      <!-- Cast (Collapsible) -->
      {#if movie.monna2_cast && movie.monna2_cast.length > 0}
        <div class="mb-6">
          <button 
            on:click={() => showCast = !showCast}
            class="flex items-center justify-between w-full text-white font-bold mb-2 hover:text-netflix-red transition-colors"
          >
            <span>{t('cast')}</span>
            <span class="text-xl">{showCast ? '▼' : '▶'}</span>
          </button>
          {#if showCast}
            <p class="text-netflix-text-secondary text-sm leading-relaxed">
              {movie.monna2_cast.join(', ')}
            </p>
          {/if}
        </div>
      {/if}

      <!-- Quality Options -->
      <div class="mb-6">
        <h3 class="text-white font-bold mb-3">{t('downloadQuality')}</h3>
        <div class="space-y-2">
          {#each ['1080p', '720p', '480p'] as quality}
            <button
              on:click={() => handleDownload(quality)}
              class="w-full bg-netflix-card-hover hover:bg-netflix-card-bg border border-netflix-border rounded px-4 py-2 text-white text-sm transition-colors text-left"
            >
              {quality}
            </button>
          {/each}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .line-clamp-4 {
    display: -webkit-box;
    -webkit-line-clamp: 4;
    line-clamp: 4;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
