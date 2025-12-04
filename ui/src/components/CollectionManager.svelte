<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import LoadingSpinner from './LoadingSpinner.svelte';

  interface Collection {
    id: number;
    name: string;
    description?: string;
    is_smart: boolean;
    created_at: string;
  }

  interface MediaItem {
    id: number;
    title: string;
    year?: number;
  }

  interface FilterCriteria {
    title_pattern?: string;
    year_range?: [number, number];
    quality_labels?: string[];
    tags?: string[];
    min_rating?: number;
    watch_status?: string;
  }

  let collections: Collection[] = [];
  let selectedCollection: Collection | null = null;
  let collectionItems: MediaItem[] = [];
  let isLoading = false;
  let isCreating = false;
  let error: string | null = null;
  let showCreateForm = false;
  let showSmartForm = false;

  // Form state
  let newCollectionName = '';
  let newCollectionDesc = '';
  let smartCriteria: FilterCriteria = {};

  onMount(async () => {
    await loadCollections();
  });

  async function loadCollections() {
    isLoading = true;
    error = null;

    try {
      // This would need to be implemented in the engine
      // For now, we'll show a placeholder
      collections = [];
    } catch (err) {
      error = `Failed to load collections: ${err}`;
      console.error('Error:', err);
    } finally {
      isLoading = false;
    }
  }

  async function selectCollection(collection: Collection) {
    selectedCollection = collection;
    isLoading = true;
    error = null;

    try {
      const items = await invoke<MediaItem[]>('get_collection_items', {
        collection_id: collection.id,
      });
      collectionItems = items;
    } catch (err) {
      error = `Failed to load collection items: ${err}`;
      console.error('Error:', err);
    } finally {
      isLoading = false;
    }
  }

  async function createCollection() {
    if (!newCollectionName.trim()) {
      error = 'Collection name is required';
      return;
    }

    isCreating = true;
    error = null;

    try {
      const collectionId = await invoke<number>('create_collection', {
        name: newCollectionName,
        description: newCollectionDesc || null,
      });

      const newCollection: Collection = {
        id: collectionId,
        name: newCollectionName,
        description: newCollectionDesc,
        is_smart: false,
        created_at: new Date().toISOString(),
      };

      collections = [...collections, newCollection];
      newCollectionName = '';
      newCollectionDesc = '';
      showCreateForm = false;
    } catch (err) {
      error = `Failed to create collection: ${err}`;
      console.error('Error:', err);
    } finally {
      isCreating = false;
    }
  }

  async function createSmartCollection() {
    if (!newCollectionName.trim()) {
      error = 'Collection name is required';
      return;
    }

    isCreating = true;
    error = null;

    try {
      const collectionId = await invoke<number>('create_smart_collection', {
        name: newCollectionName,
        criteria: smartCriteria,
      });

      const newCollection: Collection = {
        id: collectionId,
        name: newCollectionName,
        description: 'Smart collection',
        is_smart: true,
        created_at: new Date().toISOString(),
      };

      collections = [...collections, newCollection];
      newCollectionName = '';
      smartCriteria = {};
      showSmartForm = false;
    } catch (err) {
      error = `Failed to create smart collection: ${err}`;
      console.error('Error:', err);
    } finally {
      isCreating = false;
    }
  }

  function resetForms() {
    newCollectionName = '';
    newCollectionDesc = '';
    smartCriteria = {};
    showCreateForm = false;
    showSmartForm = false;
    error = null;
  }
</script>

<div class="collection-manager w-full text-white">
  {#if error}
    <div class="bg-red-900 border border-red-700 text-red-100 px-4 py-3 rounded mb-4">
      {error}
    </div>
  {/if}

  <div class="grid grid-cols-1 md:grid-cols-3 gap-4 h-full">
    <!-- Collections List -->
    <div class="md:col-span-1 bg-gray-800 rounded p-4 flex flex-col">
      <div class="flex justify-between items-center mb-4">
        <h2 class="text-xl font-semibold">Collections</h2>
        <div class="flex gap-2">
          <button
            on:click={() => {
              resetForms();
              showCreateForm = true;
            }}
            class="px-3 py-1 bg-blue-600 hover:bg-blue-700 rounded text-sm transition"
          >
            + New
          </button>
          <button
            on:click={() => {
              resetForms();
              showSmartForm = true;
            }}
            class="px-3 py-1 bg-purple-600 hover:bg-purple-700 rounded text-sm transition"
          >
            🧠 Smart
          </button>
        </div>
      </div>

      {#if isLoading}
        <div class="flex justify-center py-8">
          <LoadingSpinner />
        </div>
      {:else if collections.length === 0}
        <div class="text-center py-8 text-gray-400">
          <p>No collections yet</p>
        </div>
      {:else}
        <div class="space-y-2 flex-1 overflow-y-auto">
          {#each collections as collection (collection.id)}
            <button
              on:click={() => selectCollection(collection)}
              class={`w-full text-left p-3 rounded transition ${
                selectedCollection?.id === collection.id
                  ? 'bg-blue-600'
                  : 'bg-gray-700 hover:bg-gray-600'
              }`}
            >
              <div class="flex items-center gap-2">
                {#if collection.is_smart}
                  <span class="text-purple-400">🧠</span>
                {/if}
                <div class="flex-1 min-w-0">
                  <p class="font-semibold truncate">{collection.name}</p>
                  {#if collection.description}
                    <p class="text-xs text-gray-400 truncate">{collection.description}</p>
                  {/if}
                </div>
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Collection Items -->
    <div class="md:col-span-2 bg-gray-800 rounded p-4 flex flex-col">
      {#if selectedCollection}
        <div class="mb-4">
          <h2 class="text-xl font-semibold mb-2">{selectedCollection.name}</h2>
          {#if selectedCollection.description}
            <p class="text-sm text-gray-400">{selectedCollection.description}</p>
          {/if}
        </div>

        {#if isLoading}
          <div class="flex justify-center py-12">
            <LoadingSpinner />
          </div>
        {:else if collectionItems.length === 0}
          <div class="text-center py-12 text-gray-400">
            <p>No items in this collection</p>
          </div>
        {:else}
          <div class="space-y-2 flex-1 overflow-y-auto">
            {#each collectionItems as item (item.id)}
              <div class="bg-gray-700 p-3 rounded hover:bg-gray-600 transition cursor-pointer">
                <p class="font-semibold">
                  {item.title}
                  {#if item.year}
                    <span class="text-gray-400">({item.year})</span>
                  {/if}
                </p>
              </div>
            {/each}
          </div>
        {/if}
      {:else}
        <div class="flex items-center justify-center h-full text-gray-400">
          <p>Select a collection to view items</p>
        </div>
      {/if}
    </div>
  </div>

  <!-- Create Collection Modal -->
  {#if showCreateForm}
    <div class="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50 p-4">
      <div class="bg-gray-800 rounded-lg p-6 max-w-md w-full">
        <h3 class="text-xl font-semibold mb-4">Create Collection</h3>

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-semibold mb-2">Collection Name</label>
            <input
              type="text"
              bind:value={newCollectionName}
              placeholder="e.g., Favorites, To Watch"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
            />
          </div>

          <div>
            <label class="block text-sm font-semibold mb-2">Description (optional)</label>
            <textarea
              bind:value={newCollectionDesc}
              placeholder="Add a description..."
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
              rows="3"
            />
          </div>

          <div class="flex gap-2 justify-end">
            <button
              on:click={resetForms}
              class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition"
            >
              Cancel
            </button>
            <button
              on:click={createCollection}
              disabled={isCreating}
              class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed rounded transition font-semibold"
            >
              {isCreating ? 'Creating...' : 'Create'}
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- Create Smart Collection Modal -->
  {#if showSmartForm}
    <div class="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50 p-4">
      <div class="bg-gray-800 rounded-lg p-6 max-w-md w-full max-h-[90vh] overflow-y-auto">
        <h3 class="text-xl font-semibold mb-4">Create Smart Collection</h3>

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-semibold mb-2">Collection Name</label>
            <input
              type="text"
              bind:value={newCollectionName}
              placeholder="e.g., 4K Movies, Unwatched"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
            />
          </div>

          <div>
            <label class="block text-sm font-semibold mb-2">Title Pattern (optional)</label>
            <input
              type="text"
              bind:value={smartCriteria.title_pattern}
              placeholder="e.g., Marvel"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
            />
          </div>

          <div>
            <label class="block text-sm font-semibold mb-2">Minimum Rating</label>
            <input
              type="number"
              bind:value={smartCriteria.min_rating}
              min="0"
              max="10"
              step="0.5"
              placeholder="0"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
            />
          </div>

          <div>
            <label class="block text-sm font-semibold mb-2">Watch Status</label>
            <select
              bind:value={smartCriteria.watch_status}
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-blue-500"
            >
              <option value="">Any</option>
              <option value="unwatched">Unwatched</option>
              <option value="in_progress">In Progress</option>
              <option value="watched">Watched</option>
            </select>
          </div>

          <div class="flex gap-2 justify-end">
            <button
              on:click={resetForms}
              class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition"
            >
              Cancel
            </button>
            <button
              on:click={createSmartCollection}
              disabled={isCreating}
              class="px-4 py-2 bg-purple-600 hover:bg-purple-700 disabled:opacity-50 disabled:cursor-not-allowed rounded transition font-semibold"
            >
              {isCreating ? 'Creating...' : 'Create'}
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .collection-manager {
    display: flex;
    flex-direction: column;
  }
</style>
