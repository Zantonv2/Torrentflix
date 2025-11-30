<script lang="ts">
  import type { UiSearchResult } from '../types';
  
  export let movie: UiSearchResult;
  export let showDetails = false;
  
  // Extract year from title if not provided
  function getCleanTitle(title: string): string {
    return title.replace(/\s*\(\d{4}\)\s*$/, '');
  }
  
  function extractYearFromTitle(title: string): number | undefined {
    const yearMatch = title.match(/\((\d{4})\)/);
    return yearMatch ? parseInt(yearMatch[1]) : undefined;
  }
  
  function handleDownload() {
    // TODO: Implement download functionality
    console.log('Download:', movie.title, movie.torrent_info.magnet_link);
  }
  
  function toggleDetails() {
    showDetails = !showDetails;
  }
  
  $: displayYear = movie.year || extractYearFromTitle(movie.title);
  $: cleanTitle = getCleanTitle(movie.title);
  $: hasDescription = movie.description && movie.description.trim().length > 0;
  $: hasCast = movie.cast && movie.cast.length > 0;
  $: hasGenres = movie.genres && movie.genres.length > 0;

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
</script>

<!-- Movie Card -->
<div class="netflix-card group cursor-pointer">
  <div class="relative aspect-[2/3] overflow-hidden rounded-md bg-netflix-dark">
    <!-- Poster Image -->
    {#if movie.poster_url}
      <img 
        src={movie.poster_url} 
        alt={cleanTitle}
        class="w-full h-full object-cover transition-transform duration-300 group-hover:scale-110"
        loading="lazy"
      />
    {:else}
      <div class="w-full h-full flex items-center justify-center bg-gradient-to-br from-netflix-gray to-netflix-dark">
        <div class="text-netflix-light text-center p-4">
          <div class="text-2xl font-bold mb-2">🎬</div>
          <div class="text-sm line-clamp-3">{cleanTitle}</div>
        </div>
      </div>
    {/if}
    
    <!-- Hover Overlay -->
    <div class="card-overlay">
      <div class="absolute bottom-0 left-0 right-0 p-4">
        <!-- Title -->
        <div class="text-white font-semibold text-sm mb-1 line-clamp-2">
          {cleanTitle}
        </div>
        
        <!-- Year and Rating -->
        <div class="flex items-center justify-between text-xs text-gray-300 mb-1">
          {#if displayYear}
            <div>{displayYear}</div>
          {/if}
          
          <!-- Rating -->
          <div class="flex items-center gap-2 text-xs">
            {#if movie.rating_kinopoisk || movie.rating_imdb}
              <span class="text-yellow-400">⭐</span>
              {#if movie.rating_kinopoisk}
                <span class="text-yellow-400">KP: {movie.rating_kinopoisk.toFixed(1)}</span>
              {/if}
              {#if movie.rating_kinopoisk && movie.rating_imdb}
                <span class="text-gray-400">|</span>
              {/if}
              {#if movie.rating_imdb}
                <span class="text-yellow-400">IMDB: {movie.rating_imdb.toFixed(1)}</span>
              {/if}
            {:else if movie.rating}
              <span class="text-yellow-400 mr-1">★</span>
              <span class="text-yellow-400">{movie.rating.toFixed(1)}</span>
            {:else}
              <span class="text-gray-400">N/A</span>
            {/if}
          </div>
          
          <!-- Placeholder for season info (will be implemented with TMDB integration) -->
          <!--
          <div class="flex items-center">
            <span class="text-blue-400 mr-1">📺</span>
            <span>S1 E1-10</span>
          </div>
          -->
        </div>
        
        <!-- Action Buttons -->
        <div class="flex space-x-2 mt-3">
          <button 
            on:click|stopPropagation={handleDownload}
            class="flex-1 px-3 py-2 bg-netflix-red text-white text-xs font-medium rounded hover:bg-red-600 transition-colors"
          >
            Скачать
          </button>
          <button 
            on:click|stopPropagation={toggleDetails}
            class="px-3 py-2 bg-netflix-gray text-white text-xs font-medium rounded hover:bg-gray-500 transition-colors"
          >
            Инфо
          </button>
        </div>
      </div>
    </div>
  </div>
</div>

<!-- Details Modal -->
{#if showDetails}
  <div class="fixed inset-0 bg-black/90 flex items-center justify-center z-50 p-4" on:click={toggleDetails}>
    <div class="bg-netflix-dark rounded-lg max-w-4xl w-full max-h-[90vh] overflow-y-auto shadow-2xl" on:click|stopPropagation>
      <!-- Header with Backdrop -->
      <div class="relative h-64 bg-cover bg-center" style="background-image: linear-gradient(to bottom, rgba(0,0,0,0.2), rgba(0,0,0,0.9)), url('{movie.poster_url || ""}')">
        <div class="absolute bottom-0 left-0 right-0 p-6">
          <h2 class="text-4xl font-bold text-white mb-1">{cleanTitle}</h2>
          {#if displayYear}
            <div class="text-gray-300 text-lg">{displayYear}</div>
          {/if}
        </div>
        <button 
          on:click|stopPropagation={toggleDetails}
          class="absolute top-4 right-4 bg-black/70 text-white rounded-full w-10 h-10 flex items-center justify-center hover:bg-white/20 transition-colors"
        >
          ✕
        </button>
      </div>
      
      <!-- Main Content -->
      <div class="p-8">
        <!-- Genres with colorful tags -->
        {#if hasGenres}
          <div class="flex flex-wrap gap-2 mb-6">
            {#each (movie.genres || []) as genre}
              <span class={`px-3 py-1 text-xs font-medium rounded-full ${genreClass(genre)}`}>
                  {genre}
                </span>
            {/each}
          </div>
        {/if}
        
        <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
          <!-- Left Column -->
          <div class="md:col-span-2">
            <!-- Description -->
            {#if hasDescription}
              <div class="mb-6">
                <h3 class="text-xl font-semibold text-white mb-3">Описание</h3>
                <p class="text-gray-300 leading-relaxed">
                  {movie.description}
                </p>
              </div>
            {/if}
            
            <!-- Cast -->
            {#if hasCast}
              <div class="mb-6">
                <h3 class="text-xl font-semibold text-white mb-3">В ролях</h3>
                <div class="text-gray-300">
                  {(movie.cast || []).join(', ')}
                </div>
              </div>
            {/if}
          </div>
          
          <!-- Right Column -->
          <div class="space-y-6">
            <!-- Duration -->
            {#if movie.runtime_minutes}
              <div>
                <h4 class="text-sm font-semibold text-gray-400 mb-1">Длительность</h4>
                <div class="text-white">
                  {Math.floor(movie.runtime_minutes / 60)}h {movie.runtime_minutes % 60}m
                </div>
              </div>
            {/if}
            
            <!-- Torrent Info -->
            <div>
              <h4 class="text-sm font-semibold text-gray-400 mb-2">Информация о торренте</h4>
              <div class="space-y-2">
                <div class="flex items-center">
                  <span class="text-blue-400 mr-2">💾</span>
                  <span class="text-white">{movie.torrent_info.size_gb.toFixed(1)} GB</span>
                </div>
                <div class="flex items-center">
                  <span class="text-green-400 mr-2">🌱</span>
                  <span class="text-white">{movie.torrent_info.seeders} сидеров</span>
                </div>
              </div>
            </div>
            
            <!-- Action Buttons -->
            <div class="pt-4 space-y-3">
              <button 
                on:click|stopPropagation={handleDownload}
                class="w-full px-6 py-3 bg-netflix-red text-white font-medium rounded-md hover:bg-red-600 transition-colors flex items-center justify-center gap-2"
              >
                <span>⬇️</span> Download Torrent
              </button>
              <button 
                class="w-full px-6 py-3 bg-netflix-gray/30 text-white font-medium rounded-md hover:bg-netflix-gray/50 transition-colors flex items-center justify-center gap-2 border border-netflix-gray/50"
              >
                <span>🔖</span> Добавить в закладки
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .netflix-card {
    position: relative;
    overflow: hidden;
    border-radius: 0.5rem;
    background-color: #1a1a1a;
    transition: all 0.3s ease;
  }
  
  .netflix-card:hover {
    transform: translateY(-4px);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  }
  
  .card-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(to top, rgba(0,0,0,0.9) 0%, rgba(0,0,0,0.7) 50%, transparent 100%);
    opacity: 0;
    transition: opacity 0.3s ease;
    display: flex;
    align-items: flex-end;
  }
  
  .netflix-card:hover .card-overlay {
    opacity: 1;
  }
  
  .quality-badge {
    position: absolute;
    top: 8px;
    right: 8px;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  
  .quality-4k {
    background-color: #f59e0b;
    color: #000;
  }
  
  .quality-1080p {
    background-color: #10b981;
    color: #fff;
  }
  
  .quality-720p {
    background-color: #3b82f6;
    color: #fff;
  }
  
  .quality-default {
    background-color: #6b7280;
    color: #fff;
  }
  
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  
  .line-clamp-3 {
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  
  .truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  :global(.bg-netflix-dark) {
    background-color: #1a1a1a;
  }
  
  :global(.bg-netflix-gray) {
    background-color: #404040;
  }
  
  :global(.bg-netflix-red) {
    background-color: #e50914;
  }
  
  :global(.text-netflix-light) {
    color: #ffffff;
  }
</style>
