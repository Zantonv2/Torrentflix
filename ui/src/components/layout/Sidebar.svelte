<script lang="ts">
  import { uiStore, setCurrentPage, type Page } from "../../stores/uiStore";
  import { t } from "svelte-i18n";
  import { i18nReady } from "../../i18n/config";

  export let sidebarCollapsed = false;

  let currentPage: Page = "discover";
  let navButtons: HTMLButtonElement[] = [];
  let currentFocusIndex = 0;

  // Navigation items - must be declared before subscribe callback
  const navItems: Array<{ id: Page; labelKey: string; icon: string }> = [
    { id: "discover", labelKey: "nav.discover", icon: "🎬" },
    { id: "library", labelKey: "nav.library", icon: "📚" },
    { id: "downloads", labelKey: "nav.downloads", icon: "📥" },
    { id: "settings", labelKey: "nav.settings", icon: "⚙️" },
  ];

  // Subscribe to current page
  const unsubscribe = uiStore.subscribe((state) => {
    currentPage = state.currentPage;
    // Update focus index when page changes
    const index = navItems.findIndex((item) => item.id === state.currentPage);
    if (index !== -1) {
      currentFocusIndex = index;
    }
  });

  function handleNavClick(page: Page) {
    setCurrentPage(page);
  }

  function handleKeyDown(event: KeyboardEvent) {
    // Arrow Up/Down for navigation
    if (event.key === "ArrowUp") {
      event.preventDefault();
      currentFocusIndex =
        (currentFocusIndex - 1 + navItems.length) % navItems.length;
      navButtons[currentFocusIndex]?.focus();
      setCurrentPage(navItems[currentFocusIndex].id);
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      currentFocusIndex = (currentFocusIndex + 1) % navItems.length;
      navButtons[currentFocusIndex]?.focus();
      setCurrentPage(navItems[currentFocusIndex].id);
    } else if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      setCurrentPage(navItems[currentFocusIndex].id);
    }
  }

  import { onDestroy } from "svelte";
  onDestroy(() => {
    unsubscribe();
  });
</script>

<!-- Sidebar (200px on desktop, hidden on mobile) -->
<aside
  class="hidden md:flex flex-col w-[200px] bg-netflix-dark border-r border-white/5 overflow-y-auto flex-shrink-0"
  role="navigation"
  aria-label="Main navigation"
>
  <!-- Navigation items -->
  <nav class="flex flex-col gap-2 p-4">
    {#each navItems as item, index (item.id)}
      <button
        bind:this={navButtons[index]}
        on:click={() => handleNavClick(item.id)}
        on:keydown={handleKeyDown}
        class="flex items-center gap-3 px-4 py-3 rounded-lg text-left transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-netflix-red {currentPage ===
        item.id
          ? 'bg-netflix-red text-white font-medium'
          : 'text-white/70 hover:text-white hover:bg-white/10'}"
        aria-current={currentPage === item.id ? "page" : undefined}
        aria-label={$i18nReady ? $t(item.labelKey) : item.id}
      >
        <span class="text-xl">{item.icon}</span>
        <span class="text-sm font-medium"
          >{$i18nReady ? $t(item.labelKey) : item.id}</span
        >
      </button>
    {/each}
  </nav>
</aside>

<!-- Mobile hamburger menu (shown on mobile) -->
<div class="md:hidden flex items-center px-4">
  <button
    class="p-2 hover:bg-white/10 rounded-lg transition-colors"
    aria-label="Toggle navigation menu"
  >
    ☰
  </button>
</div>

<style>
  :global(.netflix-dark) {
    background-color: #181818;
  }

  :global(.netflix-red) {
    background-color: #e50914;
  }
</style>
