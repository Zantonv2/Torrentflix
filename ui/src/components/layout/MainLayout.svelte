<script lang="ts">
  import { uiStore, setCurrentPage, type Page } from "../../stores/uiStore";
  import { searchStore, setLoading, setQuery } from "../../stores/searchStore";
  import { invoke } from "@tauri-apps/api/core";
  import Header from "./Header.svelte";
  import Sidebar from "./Sidebar.svelte";
  import { onDestroy, onMount } from "svelte";

  let sidebarCollapsed = false;
  let mobileMenuOpen = false;
  let previousFocus: HTMLElement | null = null;
  let unsubscribe: (() => void) | null = null;
  let headerComponent: any = null;
  let openModals: Set<string> = new Set();

  function handleMobileMenuToggle() {
    if (!mobileMenuOpen) {
      previousFocus = document.activeElement as HTMLElement;
    }
    mobileMenuOpen = !mobileMenuOpen;
    if (!mobileMenuOpen && previousFocus) {
      previousFocus.focus();
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    // Ctrl+K or Cmd+K: Focus search input (Property 68)
    if ((e.ctrlKey || e.metaKey) && e.key === "k") {
      e.preventDefault();
      if (headerComponent && headerComponent.focusSearch) {
        headerComponent.focusSearch();
      }
    }

    // Escape: Close mobile menu or modals (Property 69)
    if (e.key === "Escape") {
      if (mobileMenuOpen) {
        mobileMenuOpen = false;
        if (previousFocus) {
          previousFocus.focus();
        }
      }
      // Dispatch event for modals to listen to
      window.dispatchEvent(new CustomEvent("mainlayout:escape"));
    }

    // Number keys 1-4: Switch pages (Property 70)
    // 1 = Discover, 2 = Library, 3 = Downloads, 4 = Settings
    if (!e.ctrlKey && !e.metaKey && !e.altKey && !e.shiftKey) {
      const pageMap: Record<string, Page> = {
        "1": "discover",
        "2": "library",
        "3": "downloads",
        "4": "settings",
      };

      if (e.key in pageMap) {
        e.preventDefault();
        setCurrentPage(pageMap[e.key]);
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

  onMount(() => {
    // Subscribe to UI store for sidebar state
    unsubscribe = uiStore.subscribe((state) => {
      sidebarCollapsed = state.sidebarCollapsed;
    });

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  });

  onDestroy(() => {
    if (unsubscribe) {
      unsubscribe();
    }
    document.removeEventListener("keydown", handleKeyDown);
  });
</script>

<div class="flex flex-col h-screen bg-netflix-black overflow-hidden">
  <!-- Header (60px fixed) -->
  <Header bind:this={headerComponent} on:search={handleSearch} />

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
