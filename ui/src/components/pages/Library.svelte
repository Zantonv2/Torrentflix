<script lang="ts">
  import { onMount } from "svelte";
  import LibraryBrowser from "../LibraryBrowser.svelte";
  import LibraryCollections from "./LibraryCollections.svelte";
  import LibraryAnalytics from "./LibraryAnalytics.svelte";
  import LibraryCleanup from "./LibraryCleanup.svelte";

  let currentTab = "browser";

  const tabs = [
    { id: "browser", label: "Браузер" },
    { id: "collections", label: "Коллекции" },
    { id: "analytics", label: "Аналитика" },
    { id: "cleanup", label: "Очистка" },
  ];

  onMount(() => {
    // Initialize library page
  });
</script>

<div class="flex flex-col h-full overflow-hidden page-transition">
  <!-- Tab navigation -->
  <div class="border-b border-white/10 px-6 py-4">
    <div class="flex gap-4">
      {#each tabs as tab (tab.id)}
        <button
          on:click={() => (currentTab = tab.id)}
          class="px-4 py-2 text-sm font-medium transition-colors {currentTab ===
          tab.id
            ? 'text-netflix-red border-b-2 border-netflix-red'
            : 'text-white/70 hover:text-white'}"
          aria-current={currentTab === tab.id ? "page" : undefined}
        >
          {tab.label}
        </button>
      {/each}
    </div>
  </div>

  <!-- Tab content -->
  <div class="flex-1 overflow-hidden">
    {#if currentTab === "browser"}
      <LibraryBrowser />
    {:else if currentTab === "collections"}
      <LibraryCollections />
    {:else if currentTab === "analytics"}
      <LibraryAnalytics />
    {:else if currentTab === "cleanup"}
      <LibraryCleanup />
    {/if}
  </div>
</div>

<style>
  :global(.netflix-red) {
    color: #e50914;
  }
</style>
