<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  export let onSearch: (query: string) => void;
  
  const dispatch = createEventDispatcher();
  let query = '';
  let isLoading = false;

  async function handleSubmit() {
    if (!query.trim() || isLoading) return;
    
    isLoading = true;
    dispatch('search', query.trim());
    
    // Simulate loading state for better UX
    setTimeout(() => {
      isLoading = false;
    }, 500);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      handleSubmit();
    }
  }
</script>

<div class="flex items-center space-x-2">
  <div class="relative">
    <input
      type="text"
      bind:value={query}
      placeholder="Search movies..."
      class="netflix-search w-64 px-4 py-2 rounded-md border transition-all duration-200"
      class:border-gray-600={!query}
      class:border-red-500={query}
      on:keydown={handleKeydown}
      disabled={isLoading}
    />
    {#if isLoading}
      <div class="absolute right-3 top-1/2 transform -translate-y-1/2">
        <div class="animate-spin h-4 w-4 border-2 border-gray-300 border-t-red-500 rounded-full"></div>
      </div>
    {/if}
  </div>
  
  <button
    on:click={handleSubmit}
    disabled={!query.trim() || isLoading}
    class="px-4 py-2 bg-netflix-red text-white rounded-md hover:bg-red-600 disabled:bg-gray-600 disabled:cursor-not-allowed transition-colors duration-200 font-medium"
  >
    {isLoading ? 'Searching...' : 'Search'}
  </button>
</div>
