<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import LoadingSpinner from './LoadingSpinner.svelte';

  interface StorageStats {
    total_size: number;
    media_item_count: number;
    file_version_count: number;
    average_versions_per_media: number;
    breakdown_by_resolution?: Record<string, number>;
    breakdown_by_quality?: Record<string, number>;
    breakdown_by_status?: Record<string, number>;
    duplicate_space: number;
    largest_items?: Array<{ title: string; size: number }>;
  }

  let stats: StorageStats | null = null;
  let isLoading = false;
  let error: string | null = null;
  let selectedTab: 'overview' | 'breakdown' | 'largest' = 'overview';

  onMount(async () => {
    await loadAnalytics();
  });

  async function loadAnalytics() {
    isLoading = true;
    error = null;

    try {
      const result = await invoke<StorageStats>('get_storage_analytics');
      stats = result;
    } catch (err) {
      error = `Failed to load storage analytics: ${err}`;
      console.error('Error:', err);
    } finally {
      isLoading = false;
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  }

  function getPercentage(value: number, total: number): number {
    return Math.round((value / total) * 100);
  }

  function getBreakdownColor(index: number): string {
    const colors = [
      'bg-blue-600',
      'bg-purple-600',
      'bg-pink-600',
      'bg-red-600',
      'bg-orange-600',
      'bg-yellow-600',
      'bg-green-600',
      'bg-teal-600',
    ];
    return colors[index % colors.length];
  }
</script>

<div class="storage-analytics w-full text-white">
  {#if error}
    <div class="bg-red-900 border border-red-700 text-red-100 px-4 py-3 rounded mb-4">
      {error}
    </div>
  {/if}

  {#if isLoading}
    <div class="flex justify-center items-center py-12">
      <LoadingSpinner />
    </div>
  {:else if stats}
    <div class="space-y-6">
      <!-- Header Stats -->
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
        <div class="bg-gray-800 p-4 rounded">
          <p class="text-gray-400 text-sm mb-1">Total Size</p>
          <p class="text-2xl font-bold">{formatBytes(stats.total_size)}</p>
        </div>
        <div class="bg-gray-800 p-4 rounded">
          <p class="text-gray-400 text-sm mb-1">Media Items</p>
          <p class="text-2xl font-bold">{stats.media_item_count}</p>
        </div>
        <div class="bg-gray-800 p-4 rounded">
          <p class="text-gray-400 text-sm mb-1">File Versions</p>
          <p class="text-2xl font-bold">{stats.file_version_count}</p>
        </div>
        <div class="bg-gray-800 p-4 rounded">
          <p class="text-gray-400 text-sm mb-1">Avg Versions</p>
          <p class="text-2xl font-bold">{stats.average_versions_per_media.toFixed(1)}</p>
        </div>
      </div>

      <!-- Duplicate Space Warning -->
      {#if stats.duplicate_space > 0}
        <div class="bg-yellow-900 border border-yellow-700 p-4 rounded">
          <p class="font-semibold mb-2">⚠️ Duplicate Space Detected</p>
          <p class="text-sm mb-2">
            You have <span class="font-bold">{formatBytes(stats.duplicate_space)}</span> of duplicate or low-quality files
            that could be cleaned up.
          </p>
          <button
            class="px-4 py-2 bg-yellow-700 hover:bg-yellow-600 rounded text-sm transition"
          >
            Run Cleanup
          </button>
        </div>
      {/if}

      <!-- Tabs -->
      <div class="flex gap-2 border-b border-gray-700">
        <button
          on:click={() => (selectedTab = 'overview')}
          class={`px-4 py-2 font-semibold transition ${
            selectedTab === 'overview'
              ? 'border-b-2 border-blue-500 text-blue-400'
              : 'text-gray-400 hover:text-white'
          }`}
        >
          Overview
        </button>
        <button
          on:click={() => (selectedTab = 'breakdown')}
          class={`px-4 py-2 font-semibold transition ${
            selectedTab === 'breakdown'
              ? 'border-b-2 border-blue-500 text-blue-400'
              : 'text-gray-400 hover:text-white'
          }`}
        >
          Breakdown
        </button>
        <button
          on:click={() => (selectedTab = 'largest')}
          class={`px-4 py-2 font-semibold transition ${
            selectedTab === 'largest'
              ? 'border-b-2 border-blue-500 text-blue-400'
              : 'text-gray-400 hover:text-white'
          }`}
        >
          Largest Items
        </button>
      </div>

      <!-- Tab Content -->
      {#if selectedTab === 'overview'}
        <div class="bg-gray-800 p-6 rounded space-y-4">
          <h3 class="text-lg font-semibold mb-4">Storage Overview</h3>

          <div class="space-y-3">
            <div>
              <div class="flex justify-between mb-2">
                <span>Total Library Size</span>
                <span class="font-semibold">{formatBytes(stats.total_size)}</span>
              </div>
              <div class="w-full bg-gray-700 rounded-full h-2">
                <div class="bg-blue-600 h-2 rounded-full" style="width: 100%"></div>
              </div>
            </div>

            {#if stats.duplicate_space > 0}
              <div>
                <div class="flex justify-between mb-2">
                  <span>Duplicate/Low-Quality Space</span>
                  <span class="font-semibold text-yellow-400">
                    {formatBytes(stats.duplicate_space)}
                    ({getPercentage(stats.duplicate_space, stats.total_size)}%)
                  </span>
                </div>
                <div class="w-full bg-gray-700 rounded-full h-2">
                  <div
                    class="bg-yellow-600 h-2 rounded-full"
                    style="width: {getPercentage(stats.duplicate_space, stats.total_size)}%"
                  ></div>
                </div>
              </div>
            {/if}
          </div>

          <div class="grid grid-cols-2 gap-4 mt-6 pt-6 border-t border-gray-700">
            <div>
              <p class="text-gray-400 text-sm mb-1">Average File Size</p>
              <p class="text-xl font-semibold">
                {formatBytes(stats.total_size / Math.max(stats.file_version_count, 1))}
              </p>
            </div>
            <div>
              <p class="text-gray-400 text-sm mb-1">Wasted Space</p>
              <p class="text-xl font-semibold text-yellow-400">
                {formatBytes(stats.duplicate_space)}
              </p>
            </div>
          </div>
        </div>
      {:else if selectedTab === 'breakdown'}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <!-- Resolution Breakdown -->
          {#if stats.breakdown_by_resolution}
            <div class="bg-gray-800 p-6 rounded">
              <h3 class="text-lg font-semibold mb-4">By Resolution</h3>
              <div class="space-y-3">
                {#each Object.entries(stats.breakdown_by_resolution) as [resolution, size], index}
                  <div>
                    <div class="flex justify-between mb-1">
                      <span>{resolution}</span>
                      <span class="text-sm text-gray-400">{formatBytes(size)}</span>
                    </div>
                    <div class="w-full bg-gray-700 rounded-full h-2">
                      <div
                        class={`${getBreakdownColor(index)} h-2 rounded-full`}
                        style="width: {getPercentage(size, stats.total_size)}%"
                      ></div>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Quality Breakdown -->
          {#if stats.breakdown_by_quality}
            <div class="bg-gray-800 p-6 rounded">
              <h3 class="text-lg font-semibold mb-4">By Quality</h3>
              <div class="space-y-3">
                {#each Object.entries(stats.breakdown_by_quality) as [quality, size], index}
                  <div>
                    <div class="flex justify-between mb-1">
                      <span>{quality}</span>
                      <span class="text-sm text-gray-400">{formatBytes(size)}</span>
                    </div>
                    <div class="w-full bg-gray-700 rounded-full h-2">
                      <div
                        class={`${getBreakdownColor(index)} h-2 rounded-full`}
                        style="width: {getPercentage(size, stats.total_size)}%"
                      ></div>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Status Breakdown -->
          {#if stats.breakdown_by_status}
            <div class="bg-gray-800 p-6 rounded">
              <h3 class="text-lg font-semibold mb-4">By Status</h3>
              <div class="space-y-3">
                {#each Object.entries(stats.breakdown_by_status) as [status, size], index}
                  <div>
                    <div class="flex justify-between mb-1">
                      <span class="capitalize">{status}</span>
                      <span class="text-sm text-gray-400">{formatBytes(size)}</span>
                    </div>
                    <div class="w-full bg-gray-700 rounded-full h-2">
                      <div
                        class={`${getBreakdownColor(index)} h-2 rounded-full`}
                        style="width: {getPercentage(size, stats.total_size)}%"
                      ></div>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      {:else if selectedTab === 'largest'}
        <div class="bg-gray-800 p-6 rounded">
          <h3 class="text-lg font-semibold mb-4">Largest Items</h3>
          {#if stats.largest_items && stats.largest_items.length > 0}
            <div class="space-y-3">
              {#each stats.largest_items as item, index}
                <div class="flex items-center gap-4">
                  <div class="text-2xl font-bold text-gray-600 w-8">#{index + 1}</div>
                  <div class="flex-1 min-w-0">
                    <p class="font-semibold truncate">{item.title}</p>
                    <div class="w-full bg-gray-700 rounded-full h-2 mt-1">
                      <div
                        class={`${getBreakdownColor(index)} h-2 rounded-full`}
                        style="width: {getPercentage(item.size, stats.total_size)}%"
                      ></div>
                    </div>
                  </div>
                  <div class="text-right whitespace-nowrap">
                    <p class="font-semibold">{formatBytes(item.size)}</p>
                    <p class="text-sm text-gray-400">{getPercentage(item.size, stats.total_size)}%</p>
                  </div>
                </div>
              {/each}
            </div>
          {:else}
            <p class="text-gray-400">No data available</p>
          {/if}
        </div>
      {/if}

      <!-- Refresh Button -->
      <div class="flex justify-end">
        <button
          on:click={loadAnalytics}
          class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition"
        >
          🔄 Refresh
        </button>
      </div>
    </div>
  {:else}
    <div class="text-center py-12 text-gray-400">
      <p>No analytics data available</p>
    </div>
  {/if}
</div>

<style>
  .storage-analytics {
    display: flex;
    flex-direction: column;
  }
</style>
