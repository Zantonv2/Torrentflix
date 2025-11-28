<script lang="ts">
  import MovieTile from './MovieTile.svelte';
  import SkeletonTile from './SkeletonTile.svelte';
  import LoadingDots from './LoadingDots.svelte';
  import type { UiSearchResult } from '../types';
  
  export let movies: UiSearchResult[] = [];
  export let loading = false;
  
  function handleMovieClick(movie: UiSearchResult) {
    // Dispatch custom event for movie selection
    const event = new CustomEvent('movieSelect', { detail: movie });
    document.dispatchEvent(event);
  }
</script>

<div class="flex-1 flex flex-col overflow-hidden">
  <!-- Movie Grid -->
  <div class="flex-1 overflow-y-auto p-8">
    {#if loading && movies.length === 0}
      <!-- Initial skeleton loading state -->
      <div class="grid gap-6 justify-center" style="grid-template-columns: repeat(5, 1fr);">
        {#each Array(10) as _}
          <SkeletonTile />
        {/each}
      </div>
    {:else if movies.length === 0}
      <!-- Empty state -->
      <div class="flex items-center justify-center h-64 text-white/60">
        <div class="text-center">
          <div class="text-4xl mb-4">🎬</div>
          <div>No movies found</div>
        </div>
      </div>
    {:else}
      <!-- Movies grid -->
      <div class="grid gap-6 justify-center" style="grid-template-columns: repeat(5, 1fr);">
        {#each movies as movie, i (`${movie.id}-${i}`)}
          <MovieTile {movie} on:click={() => handleMovieClick(movie)} />
        {/each}
        
        <!-- Loading skeletons at bottom -->
        {#if loading}
          {#each Array(5) as _}
            <SkeletonTile />
          {/each}
        {/if}
      </div>
      
      <!-- Loading dots indicator for pagination -->
      {#if loading && movies.length > 0}
        <LoadingDots />
      {/if}
    {/if}
  </div>
</div>

<style>
  :global(.bg-netflix-card-bg) {
    background-color: #1a1a1a;
  }
  
  :global(.bg-netflix-card-hover) {
    background-color: #2a2a2a;
  }
  
  :global(.bg-netflix-gray) {
    background-color: #404040;
  }
  
  :global(.text-netflix-text-muted) {
    color: #999999;
  }
</style>
