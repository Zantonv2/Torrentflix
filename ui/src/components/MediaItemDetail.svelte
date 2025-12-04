<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import LoadingSpinner from './LoadingSpinner.svelte';

  interface MediaItem {
    id: number;
    title: string;
    original_title?: string;
    year?: number;
    media_type: 'movie' | 'series';
    overview?: string;
    genres?: string[];
    cast?: string[];
    director?: string;
    runtime?: number;
    poster_url?: string;
    poster_path?: string;
    user_rating?: number;
    watch_status: 'unwatched' | 'in_progress' | 'watched';
    custom_notes?: string;
    created_at: string;
    updated_at: string;
  }

  interface FileVersion {
    id: number;
    media_item_id: number;
    file_size: number;
    status: string;
    is_preferred: boolean;
    resolution_width?: number;
    resolution_height?: number;
    video_codec?: string;
    audio_codec?: string;
    quality_label?: string;
    added_at: string;
  }

  interface PlaybackHistory {
    id: number;
    file_version_id: number;
    started_at: string;
    stopped_at?: string;
    playback_position: number;
    duration: number;
    completed: boolean;
  }

  export let mediaId: number;
  export let onClose: () => void;

  let mediaItem: MediaItem | null = null;
  let fileVersions: FileVersion[] = [];
  let playbackHistory: PlaybackHistory[] = [];
  let isLoading = true;
  let error: string | null = null;
  let editingNotes = false;
  let newNotes = '';
  let newRating = 0;
  let newWatchStatus = 'unwatched';

  onMount(async () => {
    await loadMediaItem();
  });

  async function loadMediaItem() {
    isLoading = true;
    error = null;

    try {
      const item = await invoke<MediaItem | null>('get_media_item', { id: mediaId });
      if (!item) {
        error = 'Media item not found';
        return;
      }

      mediaItem = item;
      newNotes = item.custom_notes || '';
      newRating = item.user_rating || 0;
      newWatchStatus = item.watch_status;

      const versions = await invoke<FileVersion[]>('get_file_versions', {
        media_id: mediaId,
      });
      fileVersions = versions;
    } catch (err) {
      error = `Failed to load media item: ${err}`;
      console.error('Load error:', err);
    } finally {
      isLoading = false;
    }
  }

  async function saveRating() {
    if (!mediaItem) return;

    try {
      await invoke('set_user_rating', {
        media_id: mediaId,
        rating: newRating,
      });
      if (mediaItem) {
        mediaItem.user_rating = newRating;
      }
    } catch (err) {
      error = `Failed to save rating: ${err}`;
    }
  }

  async function saveWatchStatus() {
    if (!mediaItem) return;

    try {
      await invoke('set_watch_status', {
        media_id: mediaId,
        status: newWatchStatus,
      });
      if (mediaItem) {
        mediaItem.watch_status = newWatchStatus;
      }
    } catch (err) {
      error = `Failed to save watch status: ${err}`;
    }
  }

  async function saveNotes() {
    if (!mediaItem) return;

    try {
      await invoke('set_custom_notes', {
        media_id: mediaId,
        notes: newNotes,
      });
      if (mediaItem) {
        mediaItem.custom_notes = newNotes;
      }
      editingNotes = false;
    } catch (err) {
      error = `Failed to save notes: ${err}`;
    }
  }

  function getResolutionLabel(width?: number, height?: number): string {
    if (!height) return 'Unknown';
    if (height >= 2160) return '4K';
    if (height >= 1080) return '1080p';
    if (height >= 720) return '720p';
    return 'SD';
  }
</script>

<div class="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50 p-4">
  <div class="bg-gray-800 rounded-lg max-w-4xl w-full max-h-[90vh] overflow-y-auto">
    {#if isLoading}
      <div class="flex justify-center items-center h-96">
        <LoadingSpinner />
      </div>
    {:else if error}
      <div class="p-6">
        <div class="bg-red-900 border border-red-700 text-red-100 px-4 py-3 rounded mb-4">
          {error}
        </div>
        <button
          on:click={onClose}
          class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition"
        >
          Close
        </button>
      </div>
    {:else if mediaItem}
      <div class="text-white">
        <!-- Header -->
        <div class="bg-gray-900 p-6 border-b border-gray-700 flex justify-between items-start">
          <div>
            <h1 class="text-3xl font-bold mb-2">{mediaItem.title}</h1>
            {#if mediaItem.original_title && mediaItem.original_title !== mediaItem.title}
              <p class="text-gray-400 mb-2">{mediaItem.original_title}</p>
            {/if}
            <div class="flex gap-4 text-sm text-gray-400">
              {#if mediaItem.year}
                <span>{mediaItem.year}</span>
              {/if}
              {#if mediaItem.runtime}
                <span>{Math.floor(mediaItem.runtime / 60)}h {mediaItem.runtime % 60}m</span>
              {/if}
              <span class="capitalize">{mediaItem.media_type}</span>
            </div>
          </div>
          <button
            on:click={onClose}
            class="text-2xl text-gray-400 hover:text-white transition"
          >
            ✕
          </button>
        </div>

        <!-- Content -->
        <div class="p-6 space-y-6">
          <!-- Overview -->
          {#if mediaItem.overview}
            <div>
              <h2 class="text-xl font-semibold mb-2">Overview</h2>
              <p class="text-gray-300">{mediaItem.overview}</p>
            </div>
          {/if}

          <!-- Metadata -->
          <div class="grid grid-cols-2 gap-6">
            {#if mediaItem.genres && mediaItem.genres.length > 0}
              <div>
                <h3 class="font-semibold mb-2">Genres</h3>
                <div class="flex flex-wrap gap-2">
                  {#each mediaItem.genres as genre}
                    <span class="bg-gray-700 px-3 py-1 rounded text-sm">{genre}</span>
                  {/each}
                </div>
              </div>
            {/if}

            {#if mediaItem.cast && mediaItem.cast.length > 0}
              <div>
                <h3 class="font-semibold mb-2">Cast</h3>
                <p class="text-gray-300 text-sm">{mediaItem.cast.slice(0, 3).join(', ')}</p>
              </div>
            {/if}

            {#if mediaItem.director}
              <div>
                <h3 class="font-semibold mb-2">Director</h3>
                <p class="text-gray-300">{mediaItem.director}</p>
              </div>
            {/if}
          </div>

          <!-- User Metadata -->
          <div class="bg-gray-700 p-4 rounded space-y-4">
            <h2 class="text-xl font-semibold">Your Library Info</h2>

            <!-- Rating -->
            <div>
              <label class="block text-sm font-semibold mb-2">Your Rating</label>
              <div class="flex items-center gap-4">
                <input
                  type="range"
                  min="0"
                  max="10"
                  step="0.5"
                  bind:value={newRating}
                  class="flex-1"
                />
                <span class="text-lg font-semibold w-12">{newRating.toFixed(1)}</span>
                <button
                  on:click={saveRating}
                  class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded transition"
                >
                  Save
                </button>
              </div>
            </div>

            <!-- Watch Status -->
            <div>
              <label class="block text-sm font-semibold mb-2">Watch Status</label>
              <div class="flex gap-2">
                {#each ['unwatched', 'in_progress', 'watched'] as status}
                  <button
                    on:click={() => {
                      newWatchStatus = status;
                      saveWatchStatus();
                    }}
                    class={`px-4 py-2 rounded transition ${
                      newWatchStatus === status
                        ? 'bg-blue-600 text-white'
                        : 'bg-gray-600 hover:bg-gray-500'
                    }`}
                  >
                    {status.replace(/_/g, ' ')}
                  </button>
                {/each}
              </div>
            </div>

            <!-- Notes -->
            <div>
              <label class="block text-sm font-semibold mb-2">Notes</label>
              {#if editingNotes}
                <div class="space-y-2">
                  <textarea
                    bind:value={newNotes}
                    class="w-full px-3 py-2 bg-gray-600 border border-gray-500 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
                    rows="4"
                    placeholder="Add your notes..."
                  />
                  <div class="flex gap-2">
                    <button
                      on:click={saveNotes}
                      class="px-4 py-2 bg-green-600 hover:bg-green-700 rounded transition"
                    >
                      Save
                    </button>
                    <button
                      on:click={() => {
                        editingNotes = false;
                        newNotes = mediaItem?.custom_notes || '';
                      }}
                      class="px-4 py-2 bg-gray-600 hover:bg-gray-500 rounded transition"
                    >
                      Cancel
                    </button>
                  </div>
                </div>
              {:else}
                <div class="bg-gray-600 p-3 rounded">
                  {#if newNotes}
                    <p class="text-gray-200">{newNotes}</p>
                  {:else}
                    <p class="text-gray-400 italic">No notes yet</p>
                  {/if}
                </div>
                <button
                  on:click={() => (editingNotes = true)}
                  class="mt-2 px-4 py-2 bg-gray-600 hover:bg-gray-500 rounded transition"
                >
                  Edit Notes
                </button>
              {/if}
            </div>
          </div>

          <!-- File Versions -->
          {#if fileVersions.length > 0}
            <div>
              <h2 class="text-xl font-semibold mb-4">File Versions ({fileVersions.length})</h2>
              <div class="space-y-2">
                {#each fileVersions as version (version.id)}
                  <div class="bg-gray-700 p-4 rounded">
                    <div class="flex justify-between items-start mb-2">
                      <div>
                        <p class="font-semibold">
                          {getResolutionLabel(version.resolution_width, version.resolution_height)}
                          {#if version.quality_label}
                            - {version.quality_label}
                          {/if}
                        </p>
                        <p class="text-sm text-gray-400">
                          {(version.file_size / (1024 * 1024 * 1024)).toFixed(2)} GB
                        </p>
                      </div>
                      <div class="text-right">
                        <p class="text-sm">
                          <span class={`px-2 py-1 rounded text-xs font-semibold ${
                            version.status === 'present'
                              ? 'bg-green-600'
                              : version.status === 'missing'
                                ? 'bg-red-600'
                                : 'bg-gray-600'
                          }`}>
                            {version.status}
                          </span>
                        </p>
                        {#if version.is_preferred}
                          <p class="text-xs text-yellow-400 mt-1">⭐ Preferred</p>
                        {/if}
                      </div>
                    </div>
                    {#if version.video_codec || version.audio_codec}
                      <p class="text-xs text-gray-400">
                        {#if version.video_codec}
                          Video: {version.video_codec}
                        {/if}
                        {#if version.audio_codec}
                          | Audio: {version.audio_codec}
                        {/if}
                      </p>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  :global(body) {
    overflow: hidden;
  }
</style>
