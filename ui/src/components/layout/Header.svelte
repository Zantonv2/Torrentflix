<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { onMount } from "svelte";
  import { searchStore } from "../../stores/searchStore";
  import { notificationStore } from "../../stores/notificationStore";
  import NotificationCenter from "../core/NotificationCenter.svelte";

  const dispatch = createEventDispatcher();

  let searchValue = $state("");
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let autocompleteTimer: ReturnType<typeof setTimeout> | null = null;
  let searchInput: HTMLInputElement | null = null;
  let showSuggestions = $state(false);
  let showHistory = $state(false);
  let suggestions = $state<string[]>([]);
  let history = $state<string[]>([]);
  let showNotificationCenter = $state(false);
  let unreadCount = $state(0);

  const unsubscribeNotifications = notificationStore.subscribe((state) => {
    unreadCount = state.unreadCount;
  });

  const DEBOUNCE_DELAY = 300; // 300ms debounce for search
  const AUTOCOMPLETE_DELAY = 150; // 150ms debounce for autocomplete

  const unsubscribeSearch = searchStore.subscribe((state) => {
    suggestions = [...state.suggestions];
    history = [...state.history];
  });

  // Export search input for parent to focus
  export function focusSearch() {
    if (searchInput) {
      searchInput.focus();
      showHistory = true;
    }
  }

  function handleSearchInput() {
    // Clear existing timers
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
    if (autocompleteTimer) {
      clearTimeout(autocompleteTimer);
    }

    // Show history if input is empty
    if (!searchValue.trim()) {
      showHistory = true;
      showSuggestions = false;
      return;
    }

    showHistory = false;

    // Set timer for autocomplete suggestions (debounce 150ms)
    autocompleteTimer = setTimeout(() => {
      // Generate suggestions based on history and input
      const query = searchValue.trim().toLowerCase();
      const filtered = history.filter((item) =>
        item.toLowerCase().includes(query),
      );
      searchStore.setSuggestions(filtered);
      showSuggestions = filtered.length > 0;
    }, AUTOCOMPLETE_DELAY);

    // Set timer for debounced search (300ms)
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
    if (autocompleteTimer) {
      clearTimeout(autocompleteTimer);
    }
    if (searchValue.trim()) {
      searchStore.addToHistory(searchValue.trim());
      dispatch("search", searchValue.trim());
      showSuggestions = false;
      showHistory = false;
    }
  }

  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === "Enter") {
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }
      if (autocompleteTimer) {
        clearTimeout(autocompleteTimer);
      }
      if (searchValue.trim()) {
        searchStore.addToHistory(searchValue.trim());
        dispatch("search", searchValue.trim());
        showSuggestions = false;
        showHistory = false;
      }
    } else if (event.key === "Escape") {
      // Close suggestions/history on Escape
      showSuggestions = false;
      showHistory = false;
    }
  }

  function handleClearSearch() {
    searchValue = "";
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
    if (autocompleteTimer) {
      clearTimeout(autocompleteTimer);
    }
    showSuggestions = false;
    showHistory = true;
    searchInput?.focus();
  }

  function selectSuggestion(suggestion: string) {
    searchValue = suggestion;
    searchStore.addToHistory(suggestion);
    dispatch("search", suggestion);
    showSuggestions = false;
    showHistory = false;
  }

  function selectHistory(item: string) {
    searchValue = item;
    searchStore.addToHistory(item);
    dispatch("search", item);
    showSuggestions = false;
    showHistory = false;
  }

  function handleInputFocus() {
    if (!searchValue.trim()) {
      showHistory = true;
    }
  }

  function handleInputBlur() {
    // Delay to allow click on suggestions
    setTimeout(() => {
      showSuggestions = false;
      showHistory = false;
    }, 200);
  }

  onMount(() => {
    return () => {
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }
      if (autocompleteTimer) {
        clearTimeout(autocompleteTimer);
      }
      unsubscribeSearch();
      unsubscribeNotifications();
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
  <div class="flex-1 max-w-2xl mx-auto px-4 relative">
    <div class="flex gap-2">
      <div class="flex-1 relative">
        <input
          bind:this={searchInput}
          type="text"
          placeholder="Search..."
          bind:value={searchValue}
          oninput={handleSearchInput}
          onkeypress={handleKeyPress}
          onfocus={handleInputFocus}
          onblur={handleInputBlur}
          class="w-full bg-white/5 backdrop-blur-md border border-white/10 rounded-lg px-4 py-2.5 text-white placeholder-white/40 focus:outline-none focus:border-netflix-red/50 focus:bg-white/10 focus:ring-2 focus:ring-netflix-red/30 transition-all duration-300"
          aria-label="Search"
          aria-autocomplete="list"
        />

        <!-- Clear button -->
        {#if searchValue}
          <button
            type="button"
            onclick={handleClearSearch}
            class="absolute right-3 top-1/2 -translate-y-1/2 text-white/50 hover:text-white transition-colors"
            aria-label="Clear search"
          >
            ✕
          </button>
        {/if}

        <!-- Suggestions dropdown -->
        {#if showSuggestions && suggestions.length > 0}
          <div
            class="absolute top-full left-0 right-0 mt-1 bg-netflix-dark border border-white/10 rounded-lg shadow-lg z-50 max-h-48 overflow-y-auto"
            role="listbox"
          >
            {#each suggestions as suggestion (suggestion)}
              <button
                type="button"
                onclick={() => selectSuggestion(suggestion)}
                class="w-full text-left px-4 py-2 text-white/80 hover:bg-white/10 hover:text-white transition-colors"
                role="option"
                aria-selected="false"
              >
                🔍 {suggestion}
              </button>
            {/each}
          </div>
        {/if}

        <!-- History dropdown -->
        {#if showHistory && history.length > 0}
          <div
            class="absolute top-full left-0 right-0 mt-1 bg-netflix-dark border border-white/10 rounded-lg shadow-lg z-50 max-h-48 overflow-y-auto"
            role="listbox"
          >
            <div class="px-4 py-2 text-xs text-white/50 font-semibold">
              RECENT SEARCHES
            </div>
            {#each history as item (item)}
              <button
                type="button"
                onclick={() => selectHistory(item)}
                class="w-full text-left px-4 py-2 text-white/80 hover:bg-white/10 hover:text-white transition-colors"
                role="option"
                aria-selected="false"
              >
                ⏱️ {item}
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <button
        onclick={handleSearchButtonClick}
        class="px-6 py-2.5 bg-netflix-red hover:bg-red-700 text-white rounded-lg font-medium transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-netflix-red/50"
        aria-label="Search"
      >
        🔍
      </button>
    </div>
  </div>

  <!-- Notification bell (right) -->
  <div class="w-[200px] flex-shrink-0 flex items-center justify-end">
    <button
      type="button"
      onclick={() => (showNotificationCenter = !showNotificationCenter)}
      class="relative p-2 text-white/70 hover:text-white transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-netflix-red/50 rounded-lg"
      aria-label="Notifications"
    >
      🔔
      <!-- Notification badge -->
      {#if unreadCount > 0}
        <span
          class="absolute top-1 right-1 w-5 h-5 bg-netflix-red rounded-full flex items-center justify-center text-white text-xs font-bold"
        >
          {unreadCount > 9 ? "9+" : unreadCount}
        </span>
      {/if}
    </button>
  </div>
</header>

<NotificationCenter
  open={showNotificationCenter}
  onClose={() => (showNotificationCenter = false)}
/>

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
