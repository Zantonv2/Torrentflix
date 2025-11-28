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
    >
      <!-- Backdrop image with gradient overlay -->
      <div class="relative h-[400px] bg-gradient-to-b from-transparent to-netflix-card-bg">
        {#if movie.poster_url}
          <img 
            src={movie.poster_url} 
            alt={movie.title}
            class="w-full h-full object-cover opacity-50"
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
        <!-- Action buttons -->
        <div class="flex gap-4 mb-6">
          <button 
            on:click={handleDownload}
            class="flex-1 bg-netflix-red hover:bg-red-600 text-white px-6 py-3 rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10"></path>
            </svg>
            Download
          </button>
          <button 
            on:click={handleBookmark}
            class="bg-netflix-gray hover:bg-gray-600 text-white px-6 py-3 rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"></path>
            </svg>
            Bookmark
          </button>
        </div>
        
        <!-- Movie info -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
          <!-- Left column - Basic info -->
          <div>
            <h2 class="text-white text-xl font-semibold mb-4">Movie Information</h2>
            
            <!-- Quality and torrent info -->
            <div class="bg-netflix-gray/20 rounded-lg p-4 mb-4">
              <div class="flex items-center justify-between mb-2">
                <span class="text-white font-medium">Quality:</span>
                <span class="text-netflix-red font-semibold">{movie.quality_badge}</span>
              </div>
              <div class="flex items-center justify-between mb-2">
                <span class="text-white font-medium">Size:</span>
                <span class="text-white/80">{movie.torrent_info.size_gb.toFixed(1)} GB</span>
              </div>
              <div class="flex items-center justify-between">
                <span class="text-white font-medium">Seeders:</span>
                <span class="text-white/80">{movie.torrent_info.seeders}</span>
              </div>
            </div>
            
            <!-- Additional metadata -->
            {#if movie.category}
              <div class="mb-3">
                <span class="text-white/60 text-sm">Genre:</span>
                <div class="text-white">{movie.category}</div>
              </div>
            {/if}
          </div>
          
          <!-- Right column - Description -->
          <div>
            <h2 class="text-white text-xl font-semibold mb-4">Description</h2>
            <div class="text-white/80 leading-relaxed">
              No description available for this movie.
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
