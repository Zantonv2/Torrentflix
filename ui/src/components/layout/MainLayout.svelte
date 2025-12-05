<script lang="ts">
  import { uiStore } from "../../stores/uiStore";
  import { searchStore, setLoading, setQuery } from "../../stores/searchStore";
  import { invoke } from "@tauri-apps/api/core";
  import Header from "./Header.svelte";
  import Sidebar from "./Sidebar.svelte";

  let sidebarCollapsed = false;
  let mobileMenuOpen = false;
  let previousFocus: HTMLElement | null = null;

  // Subscribe to UI store for sidebar state
  const unsubscribe = uiStore.subscribe((state) => {
    sidebarCollapsed = state.sidebarCollapsed;
  });

  function handleMobileMenuToggle() {
    if (!mobileMenuOpen) {
      previousFocus = document.activeElement as HTMLElement;
    }
    mobileMenuOpen = !mobileMenuOpen;
    if (!mobileMenuOpen && previousFocus) {
      previousFocus.focus();
    }
  }

  function handleEscapeKey(e: KeyboardEvent) {
    if (e.key === "Escape" && mobileMenuOpen) {
      mobileMenuOpen = false;
      if (previousFocus) {
        previousFocus.focus();
      }
    }
  }

  async function handleSearch(event: CustomEvent<string>) {
    const query = event.detail;
    setQuery(query);
    setLoading(true);

    try {
      const response = await invoke("search_movies", {
        query,
        media_type: "movie",
        limit: 50,
      });
      const results = (response as any).results || [];
      searchStore.setResults(results);
    } catch (err) {
      console.error("Search error:", err);
      searchStore.setError(err as string);
    }
  }

  // Cleanup on destroy
  import { onDestroy, onMount } from "svelte";

  onMount(() => {
    document.addEventListener("keydown", handleEscapeKey);
    return () => {
      document.removeEventListener("keydown", handleEscapeKey);
    };
  });

  onDestroy(() => {
    unsubscribe();
    document.removeEventListener("keydown", handleEscapeKey);
  });
</script>

<div class="flex flex-col h-screen bg-netflix-black overflow-hidden">
  <!-- Header (60px fixed) -->
  <Header on:search={handleSearch} />

  <!-- Main content area (flex-1 with flex row) -->
  <div class="flex flex-1 overflow-hidden relative">
    <!-- Desktop Sidebar (200px on desktop, hidden on mobile) -->
    <div class="hidden md:block">
      <Sidebar {sidebarCollapsed} />
    </div>

    <!-- Mobile Menu (slides in from left on mobile) -->
    {#if mobileMenuOpen}
      <div
        class="fixed inset-0 bg-black/50 z-40 md:hidden"
        on:click={handleMobileMenuToggle}
        role="presentation"
      ></div>
      <div
        class="fixed left-0 top-[60px] bottom-0 w-[200px] bg-netflix-dark border-r border-white/5 z-50 md:hidden overflow-y-auto"
      >
        <Sidebar {sidebarCollapsed} />
      </div>
    {/if}

    <!-- Mobile hamburger button (shown on mobile) -->
    <button
      class="md:hidden absolute left-4 top-1/2 -translate-y-1/2 p-2 hover:bg-white/10 rounded-lg transition-colors z-30 focus:outline-none focus:ring-2 focus:ring-netflix-red"
      on:click={handleMobileMenuToggle}
      aria-label="Toggle navigation menu"
      aria-expanded={mobileMenuOpen}
    >
      ☰
    </button>

    <!-- Content area (flex-1, scrollable) -->
    <main class="flex-1 overflow-y-auto overflow-x-hidden">
      <slot />
    </main>
  </div>
</div>

<style>
  :global(.netflix-black) {
    background-color: #141414;
  }
</style>
