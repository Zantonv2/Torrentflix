<script lang="ts">
  import type { UiSearchResult } from '../types';
  
  export let movie: UiSearchResult;
  
  function getQualityClass(quality: string): string {
    if (quality.includes('4K') || quality.includes('2160p')) return 'quality-4k';
    if (quality.includes('1080p')) return 'quality-1080p';
    if (quality.includes('720p')) return 'quality-720p';
    return 'quality-default';
  }
  
  function handleDownload() {
    // TODO: Implement download functionality
    console.log('Download:', movie.title, movie.torrent_info.magnet_link);
  }
  
  function handleDetails() {
    // TODO: Show details modal
    console.log('Details:', movie.title);
  }
</script>

<div class="netflix-card group cursor-pointer">
  <div class="relative aspect-[2/3] overflow-hidden rounded-md bg-netflix-dark">
    <!-- Poster Image -->
    {#if movie.poster_url}
      <img 
        src={movie.poster_url} 
        alt={movie.title}
        class="w-full h-full object-cover transition-transform duration-300 group-hover:scale-110"
        loading="lazy"
      />
    {:else}
      <div class="w-full h-full flex items-center justify-center bg-gradient-to-br from-netflix-gray to-netflix-dark">
        <div class="text-netflix-light text-center p-4">
          <div class="text-2xl font-bold mb-2">🎬</div>
          <div class="text-sm line-clamp-3">{movie.title}</div>
        </div>
      </div>
    {/if}
    
    <!-- Quality Badge -->
    <div class="quality-badge {getQualityClass(movie.quality_badge)}">
      {movie.quality_badge}
    </div>
    
    <!-- Hover Overlay -->
    <div class="card-overlay">
      <div class="absolute bottom-0 left-0 right-0 p-4">
        <!-- Title and Year -->
        <div class="text-white font-semibold text-sm mb-1 line-clamp-2">
          {movie.title}
        </div>
        {#if movie.year}
          <div class="text-gray-300 text-xs mb-2">{movie.year}</div>
        {/if}
        
        <!-- Rating and Runtime -->
        <div class="flex items-center space-x-3 text-xs text-gray-300 mb-3">
          {#if movie.rating}
            <div class="flex items-center">
              <span class="text-yellow-400 mr-1">★</span>
              <span>{movie.rating.toFixed(1)}</span>
            </div>
          {/if}
          {#if movie.runtime_minutes}
            <div>{movie.runtime_minutes} min</div>
          {/if}
        </div>
        
        <!-- Torrent Info -->
        <div class="flex items-center justify-between text-xs text-gray-400 mb-3">
          <div>{movie.torrent_info.size_gb.toFixed(1)} GB</div>
          <div>{movie.torrent_info.seeders} seeders</div>
        </div>
        
        <!-- Action Buttons -->
        <div class="flex space-x-2">
          <button 
            on:click|stopPropagation={handleDownload}
            class="flex-1 px-3 py-2 bg-netflix-red text-white text-xs font-medium rounded hover:bg-red-600 transition-colors"
          >
            Download
          </button>
          <button 
            on:click|stopPropagation={handleDetails}
            class="px-3 py-2 bg-netflix-gray text-white text-xs font-medium rounded hover:bg-gray-500 transition-colors"
          >
            Info
          </button>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
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
</style>
