<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { t } from "svelte-i18n";
  import { downloadStore } from "../../stores/downloadStore";
  import Modal from "../core/Modal.svelte";
  import Button from "../core/Button.svelte";
  import EmptyState from "../core/EmptyState.svelte";
  import ContextMenu from "../core/ContextMenu.svelte";
  import type { MenuItem } from "../core/ContextMenu.svelte";

  let downloads: any[] = [];
  let isLoading = false;
  let error: string | null = null;
  let pollInterval: ReturnType<typeof setInterval> | null = null;
  let deleteModalOpen = false;
  let deleteTargetHash: string | null = null;
  let isDeleting = false;
  let unsubscribe: (() => void) | null = null;
  let showContextMenu = false;
  let contextMenuX = 0;
  let contextMenuY = 0;
  let contextMenuHash: string | null = null;

  async function loadDownloads() {
    try {
      const response = await invoke("get_active_downloads");
      const downloadsList = (response as any).downloads || [];
      downloadStore.update((state) => ({
        ...state,
        active: downloadsList,
        loading: false,
        error: null,
      }));
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      downloadStore.update((state) => ({
        ...state,
        error: errorMsg,
        loading: false,
      }));
      console.error("Failed to load downloads:", err);
    }
  }

  async function handlePause(hash: string) {
    try {
      await invoke("pause_download", { hash });
      await loadDownloads();
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      downloadStore.update((state) => ({ ...state, error: errorMsg }));
      console.error("Failed to pause download:", err);
    }
  }

  async function handleResume(hash: string) {
    try {
      await invoke("resume_download", { hash });
      await loadDownloads();
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      downloadStore.update((state) => ({ ...state, error: errorMsg }));
      console.error("Failed to resume download:", err);
    }
  }

  function openDeleteModal(hash: string) {
    deleteTargetHash = hash;
    deleteModalOpen = true;
  }

  function closeDeleteModal() {
    deleteModalOpen = false;
    deleteTargetHash = null;
    isDeleting = false;
  }

  async function confirmDelete() {
    if (!deleteTargetHash) return;

    isDeleting = true;
    try {
      await invoke("delete_download", {
        hash: deleteTargetHash,
        delete_files: false,
      });
      await loadDownloads();
      closeDeleteModal();
    } catch (err) {
      console.error("Failed to delete download:", err);
      isDeleting = false;
    }
  }

  function handleContextMenu(event: MouseEvent, hash: string) {
    event.preventDefault();
    contextMenuX = event.clientX;
    contextMenuY = event.clientY;
    contextMenuHash = hash;
    showContextMenu = true;
  }

  function getContextMenuItems(hash: string): MenuItem[] {
    const download = downloads.find((d) => d.hash === hash);
    if (!download) return [];

    return [
      {
        id: "pause-resume",
        label: download.is_paused ? "Resume" : "Pause",
        icon: download.is_paused ? "▶️" : "⏸️",
        action: () => {
          if (download.is_paused) {
            handleResume(hash);
          } else {
            handlePause(hash);
          }
        },
      },
      {
        id: "cancel",
        label: "Cancel",
        icon: "❌",
        action: () => {
          openDeleteModal(hash);
        },
      },
      {
        id: "open-folder",
        label: "Open Folder",
        icon: "📁",
        action: async () => {
          try {
            await invoke("open_download_folder", { hash });
          } catch (err) {
            console.error("Failed to open folder:", err);
          }
        },
      },
    ];
  }

  onMount(() => {
    // Subscribe to download store
    unsubscribe = downloadStore.subscribe((state) => {
      downloads = state.active;
      isLoading = state.loading;
      error = state.error;
    });

    loadDownloads();

    // Start polling every 5 seconds
    pollInterval = setInterval(loadDownloads, 5000);

    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
      if (pollInterval) {
        clearInterval(pollInterval);
      }
    };
  });

  onDestroy(() => {
    if (pollInterval) {
      clearInterval(pollInterval);
    }
    if (unsubscribe) {
      unsubscribe();
    }
  });
</script>

<div class="flex flex-col h-full overflow-hidden p-6 page-transition">
  <h1 class="text-2xl font-bold text-white mb-6">
    {$t("downloads.active_downloads")}
  </h1>

  {#if isLoading && downloads.length === 0}
    <div class="flex items-center justify-center flex-1">
      <div class="text-center">
        <div
          class="animate-spin rounded-full h-12 w-12 border-b-2 border-netflix-red mx-auto mb-4"
        ></div>
        <p class="text-white/70">{$t("common.loading")}</p>
      </div>
    </div>
  {:else if error}
    <div class="flex items-center justify-center flex-1">
      <div class="text-center">
        <p class="text-netflix-red mb-4">{$t("common.error")}: {error}</p>
        <button
          onclick={loadDownloads}
          class="px-4 py-2 bg-netflix-red text-white rounded hover:bg-red-700 transition-colors"
          aria-label={$t("common.retry")}
        >
          {$t("common.retry")}
        </button>
      </div>
    </div>
  {:else if downloads.length === 0}
    <div class="flex-1">
      <EmptyState
        icon="📥"
        title={$t("downloads.empty_state")}
        description="Start downloading movies from the Discover page"
        actionLabel="Browse Movies"
        actionIcon="🎬"
        onAction={() => (window.location.hash = "#/discover")}
      />
    </div>
  {:else}
    <div class="flex-1 overflow-y-auto space-y-4">
      {#each downloads as download (download.hash)}
        <div
          class="bg-white/5 border border-white/10 rounded-lg p-4 cursor-context-menu"
          oncontextmenu={(e) => handleContextMenu(e, download.hash)}
        >
          <div class="flex justify-between items-start mb-2">
            <h3 class="text-white font-medium">{download.name}</h3>
            <span class="text-sm text-white/50"
              >{Math.round(download.progress || 0)}%</span
            >
          </div>

          <!-- Progress bar -->
          <div class="w-full bg-white/10 rounded-full h-2 mb-3">
            <div
              class="bg-netflix-red h-2 rounded-full transition-all duration-300"
              style="width: {Math.round(download.progress || 0)}%"
            ></div>
          </div>

          <!-- Stats -->
          <div class="text-sm text-white/60 mb-4">
            {download.downloaded_size || 0} / {download.total_size || 0} GB |
            {download.speed || 0} MB/s
          </div>

          <!-- Buttons -->
          <div class="flex gap-2">
            {#if download.is_paused}
              <button
                onclick={() => handleResume(download.hash)}
                class="px-3 py-1 bg-netflix-red hover:bg-red-700 text-white text-sm rounded transition-colors"
                aria-label={$t("downloads.resume")}
              >
                {$t("downloads.resume")}
              </button>
            {:else}
              <button
                onclick={() => handlePause(download.hash)}
                class="px-3 py-1 bg-white/10 hover:bg-white/20 text-white text-sm rounded transition-colors"
                aria-label={$t("downloads.pause")}
              >
                {$t("downloads.pause")}
              </button>
            {/if}

            <button
              onclick={() => openDeleteModal(download.hash)}
              class="px-3 py-1 bg-red-900/30 hover:bg-red-900/50 text-red-400 text-sm rounded transition-colors"
              aria-label={$t("downloads.delete")}
            >
              {$t("downloads.delete")}
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if deleteModalOpen}
  <Modal title={$t("downloads.confirmation")} onClose={closeDeleteModal}>
    <p class="text-white/80 mb-4">
      {$t("downloads.confirmation")}
    </p>
    <svelte:fragment slot="footer">
      <Button
        variant="secondary"
        onclick={closeDeleteModal}
        disabled={isDeleting}
      >
        {$t("common.cancel")}
      </Button>
      <Button variant="danger" onclick={confirmDelete} loading={isDeleting}>
        {$t("downloads.confirm_delete")}
      </Button>
    </svelte:fragment>
  </Modal>
{/if}

{#if showContextMenu && contextMenuHash}
  <ContextMenu
    items={getContextMenuItems(contextMenuHash)}
    x={contextMenuX}
    y={contextMenuY}
    onClose={() => (showContextMenu = false)}
  />
{/if}

<style>
  :global(.netflix-red) {
    background-color: #e50914;
    color: #e50914;
  }
</style>
