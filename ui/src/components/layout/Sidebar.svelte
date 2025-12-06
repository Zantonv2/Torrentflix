<script lang="ts">
  import {
    uiStore,
    setCurrentPage,
    toggleSidebar,
    type Page,
  } from "../../stores/uiStore";
  import { onMount, onDestroy } from "svelte";

  let currentPage: Page = "discover";
  let sidebarCollapsed = false;
  let navButtons: HTMLButtonElement[] = [];
  let currentFocusIndex = 0;
  let unsubscribe: (() => void) | null = null;
  let hoveredItem: Page | null = null;

  // Navigation items with labels
  const navItems: Array<{ id: Page; label: string; icon: string }> = [
    { id: "discover", label: "Discover", icon: "🎬" },
    { id: "library", label: "Library", icon: "📚" },
    { id: "downloads", label: "Downloads", icon: "📥" },
    { id: "settings", label: "Settings", icon: "⚙️" },
  ];

  function handleNavClick(page: Page) {
    setCurrentPage(page);
  }

  function handleCollapseToggle() {
    toggleSidebar();
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

  onMount(() => {
    // Subscribe to UI store for page and sidebar state
    unsubscribe = uiStore.subscribe((state) => {
      currentPage = state.currentPage;
      sidebarCollapsed = state.sidebarCollapsed;
      // Update focus index when page changes
      const index = navItems.findIndex((item) => item.id === state.currentPage);
      if (index !== -1) {
        currentFocusIndex = index;
      }
    });
  });

  onDestroy(() => {
    if (unsubscribe) {
      unsubscribe();
    }
  });
</script>

<!-- Sidebar (200px expanded, 60px collapsed on desktop, hidden on mobile) -->
<aside
  class="hidden md:flex flex-col bg-gradient-to-b from-netflix-dark/95 to-netflix-dark border-r border-white/5 overflow-y-auto flex-shrink-0 transition-all duration-300 ease-out {sidebarCollapsed
    ? 'w-[60px]'
    : 'w-[200px]'}"
  role="navigation"
  aria-label="Main navigation"
  aria-expanded={!sidebarCollapsed}
>
  <!-- Collapse toggle button -->
  <div class="flex items-center justify-between p-4 border-b border-white/5">
    {#if !sidebarCollapsed}
      <span class="text-xs font-semibold text-white/50 uppercase tracking-wider"
        >Menu</span
      >
    {/if}
    <button
      on:click={handleCollapseToggle}
      class="p-1.5 hover:bg-white/10 rounded-lg transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-netflix-red/50 ml-auto"
      aria-label={sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}
      title={sidebarCollapsed ? "Expand" : "Collapse"}
    >
      {sidebarCollapsed ? "→" : "←"}
    </button>
  </div>

  <!-- Navigation items -->
  <nav class="flex flex-col gap-1 p-3 flex-1">
    {#each navItems as item, index (item.id)}
      <div class="relative group">
        <button
          bind:this={navButtons[index]}
          on:click={() => handleNavClick(item.id)}
          on:keydown={handleKeyDown}
          on:mouseenter={() => (hoveredItem = item.id)}
          on:mouseleave={() => (hoveredItem = null)}
          class="w-full flex items-center gap-3 px-3 py-3 rounded-lg text-left transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-netflix-red/50 {currentPage ===
          item.id
            ? 'bg-netflix-red text-white font-medium'
            : 'text-white/70 hover:text-white hover:bg-white/10'}"
          aria-current={currentPage === item.id ? "page" : undefined}
          aria-label={item.label}
        >
          <span class="text-lg flex-shrink-0">{item.icon}</span>
          {#if !sidebarCollapsed}
            <span class="text-sm font-medium truncate">{item.label}</span>
          {/if}
        </button>

        <!-- Tooltip (shown when collapsed and hovering) -->
        {#if sidebarCollapsed && hoveredItem === item.id}
          <div
            class="absolute left-full ml-2 top-1/2 -translate-y-1/2 bg-white/95 text-netflix-dark text-xs font-semibold px-2 py-1 rounded whitespace-nowrap pointer-events-none z-50 shadow-lg"
            role="tooltip"
          >
            {item.label}
          </div>
        {/if}
      </div>
    {/each}
  </nav>
</aside>

<!-- Mobile hamburger menu (shown on mobile) -->
<div class="md:hidden flex items-center px-4">
  <button
    on:click={handleCollapseToggle}
    class="p-2 hover:bg-white/10 rounded-lg transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-netflix-red/50"
    aria-label="Toggle navigation menu"
    aria-expanded={!sidebarCollapsed}
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

  :global(.from-netflix-dark\/95) {
    --tw-gradient-from: rgba(24, 24, 24, 0.95);
  }

  :global(.to-netflix-dark) {
    --tw-gradient-to: #181818;
  }
</style>
