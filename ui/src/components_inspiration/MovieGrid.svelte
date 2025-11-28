<script>
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import MovieTile from './MovieTile.svelte';
  import SkeletonTile from './SkeletonTile.svelte';
  import LoadingDots from './LoadingDots.svelte';
  import { loading, movies, currentPage } from '../stores/movies.js';

  export let loadFeed;
  let dispatch = createEventDispatcher();
  let scrollContainer;
  let containerWidth = 0;
  let containerHeight = 0;
  let scrollTop = 0;
  let lastMovieElement;
  let intersectionObserver;
  let lastLoadTime = 0;

  // Virtual scrolling constants
  const TILE_WIDTH = 250; // Good balance for 4 per row
  const TILE_GAP = 24; // gap-6 = 24px
  const INFO_SECTION_HEIGHT = 80; // Title + rating section
  const BUFFER_ROWS = 2; // render 2 extra rows above/below

  // Dynamic tile height calculation for exactly 1 row vertical with reduced spacing
  $: TILE_HEIGHT = containerHeight > 0 ? Math.floor((containerHeight - 150 - (0 * TILE_GAP)) / 1) - INFO_SECTION_HEIGHT : 550;
  $: rowHeight = TILE_HEIGHT + INFO_SECTION_HEIGHT + TILE_GAP;

  // Virtual scrolling calculations (only active when containerWidth > 0)
  $: columns = 5; // Fixed 5 columns for 5x1=5 films total
  $: totalRows = Math.ceil($movies.length / columns);
  $: visibleRows = Math.ceil(containerHeight / rowHeight || 10);
  $: startRow = Math.max(0, Math.floor(scrollTop / rowHeight) - BUFFER_ROWS);
  $: endRow = Math.min(totalRows, Math.floor((scrollTop + containerHeight) / rowHeight) + BUFFER_ROWS);
  $: startIndex = startRow * columns;
  $: endIndex = Math.min($movies.length, endRow * columns);
  $: visibleMovies = $movies.slice(startIndex, endIndex);
  $: topPadding = startRow * rowHeight;
  $: bottomPadding = (totalRows - endRow) * rowHeight;
  $: totalContentHeight = totalRows * rowHeight;
  $: useVirtualScrolling = containerWidth > 0 && $movies.length > 20 && totalContentHeight > containerHeight; // Lower threshold since we show fewer movies

  async function enrichRatings() {
    if (!$movies || $movies.length === 0) {
      console.log('No movies to enrich ratings for');
      return;
    }
    
    try {
      const { enrich_ratings } = await import('@tauri-apps/api/core');
      const movieData = $movies.map(movie => ({
        title: movie.title,
        year: movie.year,
        kinopoisk_id: movie.kinopoisk_id
      }));
      
      console.log('Enriching ratings for', movieData.length, 'movies...');
      await enrich_ratings(movieData);
      console.log(' Ratings enriched!');
    } catch (error) {
      console.error('Failed to enrich ratings:', error);
    }
  }

  async function loadMore() {
    const now = Date.now();
    if (now - lastLoadTime < 2000) {
      console.log('🛑 Load more throttled - too soon');
      return;
    }
    
    console.log('🔍 DEBUG: Before increment - currentPage:', $currentPage);
    lastLoadTime = now;
    console.log('📄 Loading more movies...');
    
    // Fix race condition: increment page and pass the new value directly
    let newPage;
    currentPage.update((p) => {
      newPage = p + 1;
      console.log('🔍 DEBUG: Incrementing page from', p, 'to', newPage);
      return newPage;
    });
    
    console.log('🔍 DEBUG: After increment - currentPage:', $currentPage, 'newPage:', newPage);
    
    // Pass the new page directly to avoid race condition
    await loadFeed(true, newPage);
  }

  function handleMovieClick(movie) {
    dispatch('movieSelect', movie);
  }

  function handleScroll() {
    if (scrollContainer) {
      scrollTop = scrollContainer.scrollTop;
      
      // Only use scroll-based loadMore for basic grid or when virtual scrolling is off
      if (!useVirtualScrolling) {
        const scrollPercentage = (scrollTop + scrollContainer.clientHeight) / scrollContainer.scrollHeight;
        if (scrollPercentage > 0.8 && !$loading) {
          loadMore();
        }
      }
    }
  }

  function setupIntersectionObserver() {
    if (intersectionObserver) {
      intersectionObserver.disconnect();
    }

    intersectionObserver = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && !$loading) {
          console.log('🔍 Last movie visible, loading more...');
          loadMore();
        }
      },
      {
        root: scrollContainer,
        rootMargin: '100px', // Start loading 100px before last element is visible
        threshold: 0.1
      }
    );
  }

  function observeLastMovie(element) {
    if (element && intersectionObserver) {
      intersectionObserver.observe(element);
    }
  }

  onMount(() => {
    console.log('🔍 DEBUG: MovieGrid onMount fired');
    console.log('🔍 DEBUG: loadFeed function:', typeof loadFeed);
    console.log('🔍 DEBUG: Initial $movies:', $movies);
    
    setupIntersectionObserver();
    loadFeed();
  });

  onDestroy(() => {
    if (intersectionObserver) {
      intersectionObserver.disconnect();
    }
  });
</script>

<div class="flex-1 flex flex-col overflow-hidden">
  <!-- Movie Grid with conditional virtual scrolling -->
  <div 
    class="flex-1 overflow-y-auto" 
    bind:this={scrollContainer} 
    bind:clientWidth={containerWidth}
    bind:clientHeight={containerHeight}
    on:scroll={handleScroll}
  >
    {#if $loading && $movies.length === 0}
      <!-- Initial skeleton loading state -->
      <div class="p-8">
        <div class="grid gap-6 justify-center" style="grid-template-columns: repeat(5, 1fr);">
          {#each Array(10) as _, i}
            <SkeletonTile />
          {/each}
        </div>
      </div>
    {:else if $movies.length === 0}
      <!-- Empty state -->
      <div class="flex items-center justify-center h-64 text-white/60">
        <div class="text-center">
          <div class="text-4xl mb-4">🎬</div>
          <div>No movies found</div>
        </div>
      </div>
    {:else if useVirtualScrolling}
      <!-- Virtual scrolling for large lists -->
      <div class="relative" style="height: {totalRows * rowHeight}px;">
        <!-- Top padding spacer -->
        <div style="height: {topPadding}px;"></div>
        
        <!-- Visible movies grid -->
        <div class="absolute left-0 right-0 p-8">
          <div 
            class="grid gap-6 justify-center" 
            style="grid-template-columns: repeat(5, 1fr);"
          >
            {#each visibleMovies as movie, i (`${movie.infohash || movie.title}-${i}`)}
              <div class="movie-tile-wrapper">
                <MovieTile {movie} posterHeight={TILE_HEIGHT} on:click={() => handleMovieClick(movie)} />
              </div>
              {#if i === visibleMovies.length - 1}
                <div bind:this={lastMovieElement} style="position: absolute; visibility: hidden;"></div>
              {/if}
            {/each}
          </div>
          
          <!-- Loading skeletons at bottom -->
          {#if $loading}
            <div class="grid gap-6 justify-center mt-6" style="grid-template-columns: repeat({columns}, minmax(200px, 200px));">
              {#each Array(Math.min(10, columns * 2)) as _, i}
                <SkeletonTile />
              {/each}
            </div>
          {/if}
        </div>
        
        <!-- Loading dots indicator for pagination -->
        {#if $loading && $movies.length > 0}
          <LoadingDots />
        {/if}
        
        <!-- Bottom padding spacer -->
        <div style="height: {bottomPadding}px;"></div>
      </div>
    {:else}
      <!-- Basic grid for small lists or when container not measured -->
      <div class="p-8">
        <div class="grid gap-6 justify-center" style="grid-template-columns: repeat(5, 1fr);">
          {#each $movies as movie, i (`${movie.infohash || movie.title}-${i}`)}
            <div class="movie-tile-wrapper">
              <MovieTile {movie} on:click={() => handleMovieClick(movie)} />
            </div>
            {#if i === $movies.length - 1}
              <div bind:this={lastMovieElement} style="position: absolute; visibility: hidden;"></div>
            {/if}
          {/each}
          
          <!-- Loading skeletons at bottom -->
          {#if $loading}
            {#each Array(10) as _, i}
              <SkeletonTile />
            {/each}
          {/if}
        </div>
        
        <!-- Loading dots indicator for pagination -->
        {#if $loading && $movies.length > 0}
          <LoadingDots />
        {/if}
      </div>
    {/if}
  </div>
</div>
