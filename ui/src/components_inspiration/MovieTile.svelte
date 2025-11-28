<script>
  import { invoke } from '@tauri-apps/api/core';
  export let movie;
  export let posterHeight = 550; // Dynamic height from parent

  $: posterUrl = movie.monna2_poster_url || movie.tmdb_poster_url || '';
  $: title = movie.cleaned_title || movie.title || 'Unknown';
  $: rating = formatRating(movie.kinopoisk_rating, movie.imdb_rating);
  $: parsedSeason = parseSeasonFromTitle(movie.title || '');
  $: seasonInfo = getSeasonInfo(movie, parsedSeason);
  
  // Debug: Log what we're actually receiving from backend
  console.log('🔍 DEBUG MOVIE:', {
    raw_title: movie.title,
    cleaned_title: movie.cleaned_title,
    using_title: title
  });
  
  let imageError = false;

  function formatRating(kp, imdb) {
    const parts = [];
    // Show both ratings when available
    if (kp) parts.push(`KP ${kp.toFixed(1)}`);
    if (imdb) parts.push(`IMDb ${imdb.toFixed(1)}`);
    if (parts.length === 0) return '⭐ N/A';
    return '⭐ ' + parts.join(' • ');
  }

  function parseSeasonFromTitle(title) {
    // Parse season info from title like "Сериал Метод 3 сезон (2025)"
    const seasonMatch = title.match(/(\d+)\s*сезон/i);
    const isSerial = title.includes('Сериал');
    
    return {
      isTVShow: isSerial || seasonMatch,
      season: seasonMatch ? parseInt(seasonMatch[1]) : null
    };
  }

  function getSeasonInfo(movie, parsedSeason) {
    if (!parsedSeason.isTVShow) return null;
    
    // Get season number from parsed title or TMDB data
    const season = movie.season_number || parsedSeason.season;
    
    // Try to get episode count from TMDB enrichment
    const episodes = movie.episodes_per_season || movie.episode_count;
    
    if (season && episodes) {
      if (episodes === Math.round(episodes)) {
        return `Сезон ${season} • ${episodes} эпизодов`;
      } else {
        return `Сезон ${season} • ~${Math.round(episodes)} эпизодов`;
      }
    } else if (season) {
      return `Сезон ${season}`;
    }
    
    return null;
  }

  function handleImageLoad(e) {
    const img = e.target;
    img.classList.remove('opacity-0');
    img.classList.add('opacity-100');
  }

  function handleImageError(e) {
    imageError = true;
  }
</script>

<div style="width: 100%; flex-shrink: 0;">
  <button
    class="bg-netflix-card-bg hover:bg-netflix-card-hover rounded-xl overflow-hidden cursor-pointer transition-all duration-300 hover:scale-105 hover:shadow-2xl hover:shadow-netflix-red/30 group block w-full relative"
    style="transform: translateZ(0);"
    on:click
  >
    <!-- Poster as background - Dynamic height for 4 films vertical layout -->
    <div 
      class="relative bg-netflix-card-hover w-full"
      style="height: {posterHeight}px; background-image: url('{posterUrl || ''}'); background-size: cover; background-position: center;"
    >
      <!-- Year Badge - Top Right Corner -->
      {#if movie.year}
        <div class="absolute top-2 right-2 bg-black/60 backdrop-blur-sm text-white text-xs px-2 py-1 rounded-md font-medium z-10">
          {movie.year}
        </div>
      {/if}
      
    {#if !posterUrl || imageError}
      <div class="flex flex-col items-center justify-center h-full gap-2">
        <span class="text-4xl">🎬</span>
        <span class="text-netflix-text-muted text-xs">No poster</span>
      </div>
    {:else}
      <!-- Enhanced gradient overlay on hover -->
      <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/40 to-transparent opacity-0 group-hover:opacity-100 transition-all duration-300"></div>
      
      <!-- Info/details indicator on hover -->
      <div class="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-300">
        <div class="bg-white/20 hover:bg-white/30 rounded-full w-14 h-14 flex items-center justify-center transition-all duration-300 transform scale-90 group-hover:scale-100 shadow-lg backdrop-blur-sm">
          <svg class="w-7 h-7 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
          </svg>
        </div>
      </div>
      
      <!-- Lazy loaded poster -->
      <img 
        src={posterUrl} 
        alt={title}
        class="w-full h-full object-cover opacity-0 transition-opacity duration-300"
        loading="lazy"
        on:load={handleImageLoad}
        on:error={handleImageError}
      />
    {/if}
  </div>

  <!-- Info Section - Clean and simple -->
  <div class="p-3 flex flex-col" style="min-height: 80px; max-height: 80px;">
    <!-- Title -->
    <div class="text-white text-sm font-semibold text-left line-clamp-2 leading-tight mb-2">
      {title}
    </div>

    <!-- Rating -->
    <div class="text-yellow-400 text-xs text-left line-clamp-1 mb-1">
      {rating}
    </div>

    <!-- Season/Episode Info for TV Shows -->
    {#if seasonInfo}
      <div class="text-blue-300 text-xs text-left line-clamp-1">
        📺 {seasonInfo}
      </div>
    {/if}
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
</style>
