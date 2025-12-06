<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { initializeI18n } from "./i18n/config";
  import MainLayout from "./components/layout/MainLayout.svelte";
  import { uiStore, type Page, setCurrentPage } from "./stores/uiStore";
  import { themeStore } from "./stores/themeStore";
  import { languageStore } from "./stores/languageStore";
  import Loading from "./components/core/Loading.svelte";
  import SetupWizard from "./components/core/SetupWizard.svelte";

  // Page components
  import DiscoverPage from "./components/pages/Discover.svelte";
  import LibraryPage from "./components/pages/Library.svelte";
  import DownloadsPage from "./components/pages/Downloads.svelte";
  import SettingsPage from "./components/pages/Settings.svelte";

  let isInitialLoading = true;
  let currentPage: Page = "discover";
  let showSetupWizard = false;
  let unsubscribe: (() => void) | null = null;
  let unsubscribeTheme: (() => void) | null = null;
  let unsubscribeLanguage: (() => void) | null = null;

  onMount(async () => {
    console.log("🚀 App.svelte onMount() called");

    // Initialize i18n first
    initializeI18n();

    // Subscribe to current page from uiStore
    unsubscribe = uiStore.subscribe((state) => {
      currentPage = state.currentPage;
    });

    // Subscribe to theme changes
    unsubscribeTheme = themeStore.subscribe(() => {
      // Theme is automatically applied via the store
    });

    // Subscribe to language changes
    unsubscribeLanguage = languageStore.subscribe(() => {
      // Language is automatically applied via the store
    });

    // Failsafe: if loading takes more than 10 seconds, show error
    const failsafeTimeout = setTimeout(() => {
      if (isInitialLoading) {
        console.error("⏰ Failsafe triggered: Loading took too long");
        isInitialLoading = false;
      }
    }, 10000);

    // Add a small delay to ensure Tauri is fully ready
    console.log("⏳ Waiting 100ms for Tauri to be ready...");
    await new Promise((resolve) => setTimeout(resolve, 100));

    isInitialLoading = false;
    clearTimeout(failsafeTimeout);

    // Check if settings are incomplete and show setup wizard
    // This would be triggered on first app launch
    const hasSeenSetupWizard = localStorage.getItem("hasSeenSetupWizard");
    if (!hasSeenSetupWizard) {
      showSetupWizard = true;
      localStorage.setItem("hasSeenSetupWizard", "true");
    }

    console.log("✅ onMount() complete");
  });

  function handleNavigateToSettings() {
    setCurrentPage("settings");
  }

  onDestroy(() => {
    // Clean up store subscriptions
    if (unsubscribe) {
      unsubscribe();
    }
    if (unsubscribeTheme) {
      unsubscribeTheme();
    }
    if (unsubscribeLanguage) {
      unsubscribeLanguage();
    }
  });
</script>

<div class="flex flex-col h-screen bg-netflix-black overflow-hidden">
  {#if isInitialLoading}
    <div class="flex items-center justify-center h-screen bg-netflix-black">
      <Loading type="spinner" />
    </div>
  {:else}
    <MainLayout>
      {#if currentPage === "discover"}
        <DiscoverPage />
      {:else if currentPage === "library"}
        <LibraryPage />
      {:else if currentPage === "downloads"}
        <DownloadsPage />
      {:else if currentPage === "settings"}
        <SettingsPage />
      {:else}
        <div class="flex items-center justify-center h-full">
          <div class="text-center py-12">
            <div class="text-6xl mb-4">🚧</div>
            <p class="text-xl text-gray-400 mb-2">В разработке</p>
            <p class="text-sm text-gray-500">Эта страница скоро будет готова</p>
          </div>
        </div>
      {/if}
    </MainLayout>

    <!-- Setup Wizard Modal -->
    <SetupWizard
      open={showSetupWizard}
      onClose={() => (showSetupWizard = false)}
      onNavigateToSettings={handleNavigateToSettings}
    />
  {/if}
</div>

<style>
  :global(.netflix-black) {
    background-color: #0f0f0f;
  }
</style>
