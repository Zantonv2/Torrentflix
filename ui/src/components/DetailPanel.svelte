<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { UiSearchResult } from '../types';

  export let movie: UiSearchResult | null = null;
  export let isOpen = false;
  
  const dispatch = createEventDispatcher();

  function handleClose() {
    dispatch('close');
  }

  function handleDownload() {
    if (movie) {
      dispatch('download', movie);
    }
  }

  function handleBookmark() {
    if (movie) {
      dispatch('bookmark', movie);
    }
  }

  $: hasGenres = movie?.genres && movie.genres.length > 0;
  $: hasDescription = movie?.description && movie.description.trim().length > 0;
  $: hasCast = movie?.cast && movie.cast.length > 0;
  $: hasRuntime = movie?.runtime_minutes && movie.runtime_minutes > 0;
  
  let overviewExpanded = false;
  const OVERVIEW_PREVIEW_LENGTH = 200;
  
  // Reset expansion when movie changes
  $: if (movie) {
    overviewExpanded = false;
  }
  
  $: overviewText = movie?.description || '';
  $: overviewPreview = overviewText.length > OVERVIEW_PREVIEW_LENGTH 
    ? overviewText.substring(0, OVERVIEW_PREVIEW_LENGTH) + '...'
    : overviewText;
  $: showReadMore = overviewText.length > OVERVIEW_PREVIEW_LENGTH;

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

  function formatRuntime(minutes: number): string {
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    if (hours > 0) {
      return `${hours}h ${mins}m`;
    }
    return `${mins}m`;
  }
</script>

{#if isOpen && movie}
  <!-- Modal container with backdrop -->
  <div 
    class="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4"
    role="presentation"
    on:click={handleClose}
  >
    <!-- Dialog element -->
    <dialog 
      class="bg-netflix-card-bg rounded-2xl max-w-4xl w-full max-h-[90vh] overflow-hidden shadow-2xl transform transition-all duration-300 scale-100 p-0 border-0"
      open
      aria-labelledby="movie-title"
      on:click|stopPropagation
    >
      <!-- Backdrop image with gradient overlay -->
      <div class="relative h-[400px] bg-gradient-to-b from-transparent to-netflix-card-bg">
        {#if movie.backdrop_url || movie.poster_url}
          <img 
            src={movie.backdrop_url || movie.poster_url || ''} 
            alt={movie.title}
            class="w-full h-full object-cover opacity-50 blur-sm"
            style="filter: blur(8px);"
          />
        {/if}
        <div class="absolute inset-0 bg-gradient-to-t from-netflix-card-bg via-netflix-card-bg/50 to-transparent"></div>
        
        <!-- Close button -->
        <button 
          on:click={handleClose}
          class="absolute top-4 right-4 bg-black/60 backdrop-blur-sm text-white rounded-full w-10 h-10 flex items-center justify-center hover:bg-black/80 transition-colors"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
          </svg>
        </button>
        
        <!-- Title overlay -->
        <div class="absolute bottom-0 left-0 right-0 p-8">
          <h1 id="movie-title" class="text-white text-4xl font-bold mb-2">{movie.title}</h1>
          {#if movie.year}
            <div class="text-white/80 text-lg">{movie.year}</div>
          {/if}
        </div>
      </div>
      
      <!-- Content section -->
      <div class="p-8 overflow-y-auto" style="max-height: calc(90vh - 400px);">
        <!-- Genres with colorful tags -->
        {#if hasGenres}
          <div class="flex flex-wrap gap-2 mb-6">
            {#each (movie?.genres || []) as genre}
              <span class={`px-3 py-1 text-xs font-medium rounded-full ${genreClass(genre)}`}>
                {genre}
              </span>
            {/each}
          </div>
        {/if}

        <!-- Action buttons -->
        <div class="flex gap-4 mb-6">
          <button 
            on:click={handleDownload}
            class="flex-1 bg-netflix-red hover:bg-red-600 text-white px-6 py-3 rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10"></path>
            </svg>
            Скачать торрент
          </button>
          <button 
            on:click={handleBookmark}
            class="bg-netflix-gray hover:bg-gray-600 text-white px-6 py-3 rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"></path>
            </svg>
            Добавить в закладки
          </button>
        </div>
        
        <!-- Movie info -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
          <!-- Left column - Description and Cast -->
          <div class="md:col-span-2">
            <!-- Description -->
            {#if hasDescription}
              <div class="mb-6">
                <h2 class="text-white text-xl font-semibold mb-3">Описание</h2>
                <p class="text-white/80 leading-relaxed">
                  {overviewExpanded ? overviewText : overviewPreview}
                </p>
                {#if showReadMore}
                  <button
                    on:click|stopPropagation={() => overviewExpanded = !overviewExpanded}
                    class="mt-2 text-netflix-red hover:text-red-400 text-sm font-medium transition-colors"
                  >
                    {overviewExpanded ? 'Свернуть' : 'Читать далее'}
                  </button>
                {/if}
              </div>
            {/if}
            
            <!-- Cast -->
            {#if hasCast}
              <div class="mb-6">
                <h2 class="text-white text-xl font-semibold mb-3">В ролях</h2>
                <div class="text-white/80">
                  {(movie?.cast || []).join(', ')}
                </div>
              </div>
            {/if}
          </div>
          
          <!-- Right column - Metadata -->
          <div class="space-y-6">
            <!-- Duration -->
            {#if hasRuntime}
          <div>
                <h3 class="text-sm font-semibold text-white/60 mb-1">Длительность</h3>
                <div class="text-white">
                  {formatRuntime(movie?.runtime_minutes || 0)}
                </div>
              </div>
            {/if}
            
            <!-- Quality and torrent info -->
            <div class="bg-netflix-gray/20 rounded-lg p-4">
              <h3 class="text-sm font-semibold text-white/60 mb-3">Информация о торренте</h3>
              <div class="space-y-2">
                <div class="flex items-center justify-between">
                  <span class="text-white/80 text-sm">Качество:</span>
                  <span class="text-netflix-red font-semibold">{movie?.quality_badge}</span>
                </div>
                <div class="flex items-center justify-between">
                  <span class="text-white/80 text-sm">Размер:</span>
                  <span class="text-white">{movie?.torrent_info.size_gb.toFixed(1)} GB</span>
                </div>
                <div class="flex items-center justify-between">
                  <span class="text-white/80 text-sm">Сидеры:</span>
                  <span class="text-white">{movie?.torrent_info.seeders}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </dialog>
  </div>
{/if}

<style>
  :global(.bg-netflix-card-bg) {
    background-color: #1a1a1a;
  }
  
  :global(.bg-netflix-gray\/20) {
    background-color: rgba(64, 64, 64, 0.2);
  }
  
  :global(.bg-netflix-gray) {
    background-color: #404040;
  }
  
  :global(.text-netflix-red) {
    color: #e50914;
  }
</style>
