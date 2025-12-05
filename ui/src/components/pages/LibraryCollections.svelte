<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import Button from "../core/Button.svelte";
  import Input from "../core/Input.svelte";
  import Modal from "../core/Modal.svelte";
  import Loading from "../core/Loading.svelte";
  import {
    libraryStore,
    setCollections,
    type Collection,
    type MediaItemBase,
  } from "../../stores/libraryStore";

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
  let collectionItems: MediaItemBase[] = [];
  let isLoading = false;
  let isCreating = false;
  let error: string | null = null;
  let showCreateModal = false;
  let showSmartModal = false;
  let showAddItemsModal = false;

  // Form state
  let newCollectionName = "";
  let newCollectionDesc = "";
  let smartCriteria: FilterCriteria = {};
  let selectedItemIds: Set<string | number> = new Set();
  let availableItems: MediaItemBase[] = [];

  onMount(async () => {
    await loadCollections();
  });

  async function loadCollections() {
    isLoading = true;
    error = null;

    try {
      // Fetch collections from backend
      const result =
        await invoke<
          Array<{
            id: string | number;
            name: string;
            description?: string;
            item_count: number;
            is_smart: boolean;
          }>
        >("get_collections");

      collections = result.map((c) => ({
        id: String(c.id),
        name: c.name,
        description: c.description,
        itemCount: c.item_count,
        isSmart: c.is_smart,
      }));

      setCollections(collections);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      error = `Ошибка загрузки коллекций: ${errorMsg}`;
      console.error("Error:", err);
    } finally {
      isLoading = false;
    }
  }

  async function selectCollection(collection: Collection) {
    selectedCollection = collection;
    isLoading = true;
    error = null;

    try {
      const items = await invoke<MediaItemBase[]>("get_collection_items", {
        collection_id: collection.id,
      });
      collectionItems = items;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      error = `Ошибка загрузки элементов коллекции: ${errorMsg}`;
      console.error("Error:", err);
    } finally {
      isLoading = false;
    }
  }

  async function createCollection() {
    if (!newCollectionName.trim()) {
      error = "Имя коллекции обязательно";
      return;
    }

    isCreating = true;
    error = null;

    try {
      const collectionId = await invoke<string | number>("create_collection", {
        name: newCollectionName,
        description: newCollectionDesc || null,
      });

      const newCollection: Collection = {
        id: String(collectionId),
        name: newCollectionName,
        description: newCollectionDesc,
        itemCount: 0,
        isSmart: false,
      };

      collections = [...collections, newCollection];
      setCollections(collections);
      resetForms();
      showCreateModal = false;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      error = `Ошибка создания коллекции: ${errorMsg}`;
      console.error("Error:", err);
    } finally {
      isCreating = false;
    }
  }

  async function createSmartCollection() {
    if (!newCollectionName.trim()) {
      error = "Имя коллекции обязательно";
      return;
    }

    isCreating = true;
    error = null;

    try {
      const collectionId = await invoke<string | number>(
        "create_smart_collection",
        {
          name: newCollectionName,
          criteria: smartCriteria,
        },
      );

      const newCollection: Collection = {
        id: String(collectionId),
        name: newCollectionName,
        description: "Умная коллекция",
        itemCount: 0,
        isSmart: true,
      };

      collections = [...collections, newCollection];
      setCollections(collections);
      resetForms();
      showSmartModal = false;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      error = `Ошибка создания умной коллекции: ${errorMsg}`;
      console.error("Error:", err);
    } finally {
      isCreating = false;
    }
  }

  async function addItemsToCollection() {
    if (!selectedCollection || selectedItemIds.size === 0) {
      error = "Выберите элементы для добавления";
      return;
    }

    isCreating = true;
    error = null;

    try {
      await invoke("add_to_collection", {
        collection_id: selectedCollection.id,
        media_ids: Array.from(selectedItemIds),
      });

      // Reload collection items
      await selectCollection(selectedCollection);
      selectedItemIds.clear();
      showAddItemsModal = false;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      error = `Ошибка добавления элементов: ${errorMsg}`;
      console.error("Error:", err);
    } finally {
      isCreating = false;
    }
  }

  async function loadAvailableItems() {
    try {
      const items = await invoke<MediaItemBase[]>("query_library", {
        filters: {},
      });
      availableItems = items;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      error = `Ошибка загрузки элементов библиотеки: ${errorMsg}`;
      console.error("Error:", err);
    }
  }

  function toggleItemSelection(itemId: string | number) {
    if (selectedItemIds.has(itemId)) {
      selectedItemIds.delete(itemId);
    } else {
      selectedItemIds.add(itemId);
    }
    selectedItemIds = selectedItemIds; // Trigger reactivity
  }

  function resetForms() {
    newCollectionName = "";
    newCollectionDesc = "";
    smartCriteria = {};
    selectedItemIds.clear();
    error = null;
  }

  function openAddItemsModal() {
    if (!selectedCollection) return;
    selectedItemIds.clear();
    loadAvailableItems();
    showAddItemsModal = true;
  }
</script>

<div class="flex h-full overflow-hidden bg-netflix-dark text-white">
  <!-- Error notification -->
  {#if error}
    <div
      class="fixed top-4 right-4 bg-red-600 text-white px-4 py-3 rounded shadow-lg z-40 max-w-sm"
    >
      {error}
      <button
        class="ml-4 text-white hover:text-gray-200"
        on:click={() => (error = null)}
        aria-label="Close error"
      >
        ✕
      </button>
    </div>
  {/if}

  <!-- Collections List Panel (300px) -->
  <div class="w-80 border-r border-netflix-gray bg-netflix-dark flex flex-col">
    <div class="p-6 border-b border-netflix-gray">
      <h2 class="text-lg font-netflix font-bold mb-4">Коллекции</h2>
      <div class="flex gap-2">
        <Button
          variant="primary"
          size="sm"
          on:click={() => {
            resetForms();
            showCreateModal = true;
          }}
          aria-label="Create new collection"
        >
          + Новая
        </Button>
        <Button
          variant="secondary"
          size="sm"
          on:click={() => {
            resetForms();
            showSmartModal = true;
          }}
          aria-label="Create smart collection"
        >
          🧠 Умная
        </Button>
      </div>
    </div>

    <!-- Collections List -->
    <div class="flex-1 overflow-y-auto">
      {#if isLoading}
        <div class="flex justify-center py-8">
          <Loading type="spinner" />
        </div>
      {:else if collections.length === 0}
        <div class="text-center py-8 text-netflix-light">
          <p>Нет коллекций</p>
        </div>
      {:else}
        <div class="space-y-1 p-4">
          {#each collections as collection (collection.id)}
            <button
              on:click={() => selectCollection(collection)}
              class={`w-full text-left p-3 rounded transition-colors ${
                selectedCollection?.id === collection.id
                  ? "bg-netflix-red text-white"
                  : "bg-netflix-gray hover:bg-netflix-gray/80 text-white"
              }`}
              aria-current={selectedCollection?.id === collection.id
                ? "true"
                : "false"}
            >
              <div class="flex items-center gap-2">
                {#if collection.isSmart}
                  <span class="text-lg">🧠</span>
                {/if}
                <div class="flex-1 min-w-0">
                  <p class="font-netflix font-semibold truncate">
                    {collection.name}
                  </p>
                  <p class="text-xs text-netflix-light truncate">
                    {collection.itemCount} элементов
                  </p>
                </div>
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <!-- Collection Items Panel -->
  <div class="flex-1 flex flex-col overflow-hidden">
    {#if selectedCollection}
      <div class="p-6 border-b border-netflix-gray">
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-2xl font-netflix font-bold">
              {selectedCollection.name}
            </h2>
            {#if selectedCollection.description}
              <p class="text-sm text-netflix-light mt-1">
                {selectedCollection.description}
              </p>
            {/if}
          </div>
          <Button
            variant="primary"
            size="md"
            on:click={openAddItemsModal}
            aria-label="Add items to collection"
          >
            + Добавить
          </Button>
        </div>
      </div>

      <!-- Items List -->
      <div class="flex-1 overflow-y-auto p-6">
        {#if isLoading}
          <div class="flex justify-center py-12">
            <Loading type="spinner" />
          </div>
        {:else if collectionItems.length === 0}
          <div class="text-center py-12 text-netflix-light">
            <p>Нет элементов в этой коллекции</p>
          </div>
        {:else}
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each collectionItems as item (item.id)}
              <div
                class="bg-netflix-gray rounded-lg p-4 hover:bg-netflix-gray/80 transition-colors"
              >
                <p class="font-netflix font-semibold text-white">
                  {item.title}
                  {#if item.year}
                    <span class="text-netflix-light text-sm">({item.year})</span
                    >
                  {/if}
                </p>
                {#if item.user_rating}
                  <p class="text-sm text-netflix-light mt-2">
                    ⭐ {item.user_rating}/10
                  </p>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {:else}
      <div class="flex items-center justify-center h-full text-netflix-light">
        <p>Выберите коллекцию для просмотра элементов</p>
      </div>
    {/if}
  </div>

  <!-- Create Collection Modal -->
  <Modal
    open={showCreateModal}
    title="Создать коллекцию"
    onClose={() => {
      showCreateModal = false;
      resetForms();
    }}
  >
    <div class="space-y-4">
      <Input
        label="Имя коллекции"
        placeholder="например, Избранное, К просмотру"
        bind:value={newCollectionName}
        error={!newCollectionName.trim() && error
          ? "Имя обязательно"
          : undefined}
      />

      <div class="flex flex-col gap-2">
        <label for="desc" class="text-sm font-netflix font-semibold text-white">
          Описание (опционально)
        </label>
        <textarea
          id="desc"
          bind:value={newCollectionDesc}
          placeholder="Добавьте описание..."
          class="w-full px-3 py-2 bg-netflix-gray text-white rounded border border-netflix-gray focus:border-netflix-red focus:outline-none transition-colors"
          rows="3"
        ></textarea>
      </div>
    </div>

    <svelte:fragment slot="footer">
      <Button
        variant="ghost"
        size="md"
        on:click={() => {
          showCreateModal = false;
          resetForms();
        }}
      >
        Отмена
      </Button>
      <Button
        variant="primary"
        size="md"
        loading={isCreating}
        disabled={isCreating || !newCollectionName.trim()}
        on:click={createCollection}
      >
        Создать
      </Button>
    </svelte:fragment>
  </Modal>

  <!-- Create Smart Collection Modal -->
  <Modal
    open={showSmartModal}
    title="Создать умную коллекцию"
    onClose={() => {
      showSmartModal = false;
      resetForms();
    }}
  >
    <div class="space-y-4 max-h-96 overflow-y-auto">
      <Input
        label="Имя коллекции"
        placeholder="например, 4K фильмы, Непросмотренные"
        bind:value={newCollectionName}
        error={!newCollectionName.trim() && error
          ? "Имя обязательно"
          : undefined}
      />

      <Input
        label="Шаблон названия (опционально)"
        placeholder="например, Marvel"
        bind:value={smartCriteria.title_pattern}
      />

      <Input
        label="Минимальный рейтинг (опционально)"
        type="number"
        min="0"
        max="10"
        step="0.5"
        placeholder="0"
        bind:value={smartCriteria.min_rating}
      />

      <div class="flex flex-col gap-2">
        <label
          for="status"
          class="text-sm font-netflix font-semibold text-white"
        >
          Статус просмотра (опционально)
        </label>
        <select
          id="status"
          bind:value={smartCriteria.watch_status}
          class="w-full px-3 py-2 bg-netflix-gray text-white rounded border border-netflix-gray focus:border-netflix-red focus:outline-none transition-colors"
        >
          <option value="">Любой</option>
          <option value="not_watched">Непросмотренные</option>
          <option value="watching">Просматриваю</option>
          <option value="watched">Просмотренные</option>
        </select>
      </div>
    </div>

    <svelte:fragment slot="footer">
      <Button
        variant="ghost"
        size="md"
        on:click={() => {
          showSmartModal = false;
          resetForms();
        }}
      >
        Отмена
      </Button>
      <Button
        variant="primary"
        size="md"
        loading={isCreating}
        disabled={isCreating || !newCollectionName.trim()}
        on:click={createSmartCollection}
      >
        Создать
      </Button>
    </svelte:fragment>
  </Modal>

  <!-- Add Items Modal -->
  <Modal
    open={showAddItemsModal}
    title="Добавить элементы в коллекцию"
    onClose={() => {
      showAddItemsModal = false;
      selectedItemIds.clear();
    }}
  >
    <div class="space-y-4 max-h-96 overflow-y-auto">
      {#if availableItems.length === 0}
        <p class="text-netflix-light">Нет доступных элементов</p>
      {:else}
        <div class="space-y-2">
          {#each availableItems as item (item.id)}
            <label
              class="flex items-center gap-3 p-2 hover:bg-netflix-gray rounded cursor-pointer"
            >
              <input
                type="checkbox"
                checked={selectedItemIds.has(item.id)}
                on:change={() => toggleItemSelection(item.id)}
                class="w-4 h-4 rounded border-netflix-gray bg-netflix-gray text-netflix-red focus:ring-netflix-red"
              />
              <span class="text-white">
                {item.title}
                {#if item.year}
                  <span class="text-netflix-light text-sm">({item.year})</span>
                {/if}
              </span>
            </label>
          {/each}
        </div>
      {/if}
    </div>

    <svelte:fragment slot="footer">
      <Button
        variant="ghost"
        size="md"
        on:click={() => {
          showAddItemsModal = false;
          selectedItemIds.clear();
        }}
      >
        Отмена
      </Button>
      <Button
        variant="primary"
        size="md"
        loading={isCreating}
        disabled={isCreating || selectedItemIds.size === 0}
        on:click={addItemsToCollection}
      >
        Добавить ({selectedItemIds.size})
      </Button>
    </svelte:fragment>
  </Modal>
</div>

<style>
  :global(.netflix-dark) {
    background-color: #0f0f0f;
  }

  :global(.netflix-gray) {
    background-color: #1a1a1a;
  }

  :global(.netflix-light) {
    color: #999999;
  }

  :global(.netflix-red) {
    background-color: #e50914;
  }
</style>
