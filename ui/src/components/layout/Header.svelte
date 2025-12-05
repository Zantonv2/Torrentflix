<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { onMount } from "svelte";
  import { t } from "svelte-i18n";
  import { i18nReady } from "../../i18n/config";

  const dispatch = createEventDispatcher();

  let searchValue = "";
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  const DEBOUNCE_DELAY = 300; // 300ms debounce

  function handleSearchInput() {
    // Clear existing timer
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }

    // Set new timer for debounced search
    debounceTimer = setTimeout(() => {
      if (searchValue.trim()) {
        dispatch("search", searchValue.trim());
      }
    }, DEBOUNCE_DELAY);
  }

  function handleSearchButtonClick() {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
    if (searchValue.trim()) {
      dispatch("search", searchValue.trim());
    }
  }

  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === "Enter") {
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }
      if (searchValue.trim()) {
        dispatch("search", searchValue.trim());
      }
    } else if (event.key === "Escape") {
      // Clear search on Escape
      searchValue = "";
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }
    }
  }

  onMount(() => {
    return () => {
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }
    };
  });
</script>

<header
  class="h-[60px] bg-gradient-to-r from-netflix-black/95 to-netflix-dark/95 backdrop-blur-xl border-b border-white/10 flex items-center justify-between px-6 shadow-lg"
>
  <!-- Logo (left) -->
  <div
    class="text-netflix-red text-2xl font-bold tracking-wider w-[200px] flex-shrink-0"
  >
    TORRENTFLIX
  </div>

  <!-- Search input (center, flex-1) -->
  <div class="flex-1 max-w-2xl mx-auto px-4">
    <div class="flex gap-2">
      <input
        type="text"
        placeholder={$i18nReady ? $t("header.search_placeholder") : "Search..."}
        bind:value={searchValue}
        on:input={handleSearchInput}
        on:keypress={handleKeyPress}
        class="flex-1 bg-white/5 backdrop-blur-md border border-white/10 rounded-lg px-4 py-2.5 text-white placeholder-white/40 focus:outline-none focus:border-netflix-red/50 focus:bg-white/10 focus:ring-2 focus:ring-netflix-red/30 transition-all duration-300"
        aria-label={$i18nReady ? $t("header.search_placeholder") : "Search"}
      />
      <button
        on:click={handleSearchButtonClick}
        class="px-6 py-2.5 bg-netflix-red hover:bg-red-700 text-white rounded-lg font-medium transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-netflix-red/50"
        aria-label={$i18nReady ? $t("header.search_button") : "Search"}
      >
        🔍
      </button>
    </div>
  </div>

  <!-- Right spacer for balance -->
  <div class="w-[200px] flex-shrink-0"></div>
</header>

<style>
  :global(.from-netflix-black\/95) {
    --tw-gradient-from: rgba(20, 20, 20, 0.95);
  }

  :global(.to-netflix-dark\/95) {
    --tw-gradient-to: rgba(24, 24, 24, 0.95);
  }

  :global(.border-netflix-red\/50) {
    border-color: rgba(229, 9, 20, 0.5);
  }

  :global(.focus\:bg-white\/10:focus) {
    background-color: rgba(255, 255, 255, 0.1);
  }
</style>
