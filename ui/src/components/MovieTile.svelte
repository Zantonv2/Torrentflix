<script lang="ts">
  import type { UiSearchResult } from '../types';
  
  export let movie: UiSearchResult;
  export let posterHeight = 400;
  
  let imageError = false;
  
  function handleImageLoad(e: Event) {
    const img = e.target as HTMLImageElement;
    if (img) {
      img.classList.remove('opacity-0');
      img.classList.add('opacity-100');
    }
  }
  
  function handleImageError(_e: Event) {
    imageError = true;
  }
  
  $: displayRating = (() => {
    const parts = [];
    if (movie.rating_kinopoisk) {
      parts.push(`KP: ${movie.rating_kinopoisk.toFixed(1)}`);
    }
    if (movie.rating_imdb) {
      parts.push(`IMDB: ${movie.rating_imdb.toFixed(1)}`);
    }
    if (movie.rating_tmdb) {
      parts.push(`TMDB: ${movie.rating_tmdb.toFixed(1)}`);
    }
    if (parts.length > 0) {
      return parts.join(' | ');
    }
    return movie.rating ? movie.rating.toFixed(1) : 'N/A';
  })();
  
  // Debug: Log TV show info
  $: if (movie.number_of_seasons || movie.number_of_episodes) {
    console.log('TV Show detected:', {
      title: movie.title,
      seasons: movie.number_of_seasons,
      episodes: movie.number_of_episodes,
      fullMovie: movie
    });
  }
  
</script>

<div style="width: 100%; flex-shrink: 0;">
  <button
    class="bg-netflix-card-bg hover:bg-netflix-card-hover rounded-xl overflow-hidden cursor-pointer transition-all duration-300 hover:scale-105 hover:shadow-2xl hover:shadow-netflix-red/30 group block w-full relative"
    style="transform: translateZ(0);"
    on:click
  >
    <!-- Poster section with dynamic height -->
    <div 
      class="relative bg-netflix-card-hover w-full"
      style="height: {posterHeight}px;"
    >
      <!-- Year badge - top right -->
      {#if movie.year}
        <div class="absolute top-2 right-2 bg-black/60 backdrop-blur-sm text-white text-xs px-2 py-1 rounded-md font-medium z-10">
          {movie.year}
        </div>
      {/if}
      
      {#if !movie.poster_url || imageError}
        <!-- No poster fallback -->
        <div class="flex flex-col items-center justify-center h-full gap-2">
          <span class="text-4xl">🎬</span>
          <span class="text-netflix-text-muted text-xs">No poster</span>
        </div>
      {:else}
        <!-- Gradient overlay on hover -->
        <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/40 to-transparent opacity-0 group-hover:opacity-100 transition-all duration-300"></div>
        
        <!-- Info indicator on hover -->
        <div class="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-300">
          <div class="bg-white/20 hover:bg-white/30 rounded-full w-14 h-14 flex items-center justify-center transition-all duration-300 transform scale-90 group-hover:scale-100 shadow-lg backdrop-blur-sm">
            <svg class="w-7 h-7 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
            </svg>
          </div>
        </div>
        
        <!-- Lazy loaded poster -->
        <img 
          src={movie.poster_url} 
          alt={movie.title}
          class="w-full h-full object-cover opacity-0 transition-opacity duration-300"
          loading="lazy"
          on:load={handleImageLoad}
          on:error={handleImageError}
        />
      {/if}
    </div>

    <!-- Info section -->
    <div class="p-3 flex flex-col" style="min-height: 80px; max-height: 80px;">
      <!-- Title -->
      <div class="text-white text-sm font-semibold text-left line-clamp-2 leading-tight mb-2">
        {movie.title}
      </div>

      <!-- Rating and TV Show Info -->
      <div class="text-netflix-text-muted text-xs text-left line-clamp-1 flex items-center gap-1 flex-wrap">
        {#if movie.rating_kinopoisk || movie.rating_imdb || movie.rating_tmdb}
          <span class="text-yellow-400">⭐</span>
          <span class="text-yellow-400">{displayRating}</span>
        {:else if movie.rating}
          <span class="text-yellow-400">★</span>
          <span class="text-yellow-400">{displayRating}</span>
        {:else}
          <span class="text-gray-400">N/A</span>
        {/if}
        
        <!-- TV Show Season/Episode Info -->
        {#if movie.number_of_seasons || movie.number_of_episodes}
          <span class="text-white/60">•</span>
          {#if movie.number_of_seasons}
            <span class="text-white/80">{movie.number_of_seasons} сезон</span>
          {/if}
          {#if movie.number_of_episodes}
            <span class="text-white/80">{movie.number_of_episodes} эп.</span>
          {/if}
        {/if}
      </div>
    </div>
  </button>
</div>

<style>
  .line-clamp-2 {
    display: -webkit-box;
    line-clamp: 2;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .line-clamp-1 {
    display: -webkit-box;
    line-clamp: 1;
    -webkit-line-clamp: 1;
    -webkit-box-orient: vertical;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .quality-badge {
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  
  .quality-4k {
    color: #60a5fa; /* blue-400 */
  }
  
  .quality-1080p {
    color: #34d399; /* emerald-400 */
  }
  
  .quality-720p {
    color: #fbbf24; /* amber-400 */
  }
  
  .quality-default {
    color: #9ca3af; /* gray-400 */
  }
  
  :global(.bg-netflix-card-bg) {
    background-color: #1a1a1a;
  }
  
  :global(.bg-netflix-card-hover) {
    background-color: #2a2a2a;
  }
  
  :global(.text-netflix-text-muted) {
    color: #999999;
  }
  
  :global(.hover\:shadow-netflix-red\/30):hover {
    box-shadow: 0 25px 50px -12px rgba(239, 68, 68, 0.3);
  }
</style>
