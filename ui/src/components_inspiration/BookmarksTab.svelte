<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  let bookmarks = [];
  let loading = true;
  let error = null;
  
  // Global bookmark settings
  let showSettings = false;
  let bookmarkSettings = {
    min_quality: "1080p",
    preferred_source: "Web",
    preferred_audio: "DTS"
  };
  let savingSettings = false;
  
  // Individual bookmark editing
  let showEditModal = false;
  let editingBookmark = null;
  let individualSettings = {
    min_quality: "1080p",
    preferred_source: "Web",
    preferred_audio: "DTS"
  };
  let savingIndividual = false;

  onMount(async () => {
    await loadBookmarks();
    await loadBookmarkSettings();
  });

  async function loadBookmarks() {
    try {
      loading = true;
      error = null;
      bookmarks = await invoke('list_bookmarks');
    } catch (err) {
      error = err;
      console.error('Failed to load bookmarks:', err);
    } finally {
      loading = false;
    }
  }
  
  async function loadBookmarkSettings() {
    try {
      const settings = await invoke('get_bookmark_settings');
      if (settings) {
        bookmarkSettings = { ...bookmarkSettings, ...settings };
      }
    } catch (err) {
      // No saved settings found, using defaults
    }
  }
  
  async function saveBookmarkSettings() {
    try {
      savingSettings = true;
      await invoke('save_bookmark_settings', { settings: bookmarkSettings });
    } catch (err) {
      // Handle save error
    } finally {
      savingSettings = false;
    }
  }

  async function removeBookmark(bookmarkId) {
    try {
      await invoke('remove_bookmark', { bookmarkId });
      await loadBookmarks(); // Refresh list
    } catch (err) {
      console.error('Failed to remove bookmark:', err);
    }
  }
  
  function openEditModal(bookmark) {
    editingBookmark = bookmark;
    individualSettings = {
      min_quality: bookmark.min_quality || '1080p',
      preferred_source: bookmark.preferred_source || 'Web',
      preferred_audio: bookmark.preferred_audio || 'DTS'
    };
    showEditModal = true;
  }
  
  async function saveIndividualSettings() {
    if (!editingBookmark) return;
    
    try {
      savingIndividual = true;
      const preferences = {
        min_quality: individualSettings.min_quality,
        preferred_source: individualSettings.preferred_source,
        preferred_audio: individualSettings.preferred_audio
      };
      
      console.log('🔧 DEBUG: Saving bookmark preferences:', preferences);
      console.log('🔧 DEBUG: Bookmark ID:', editingBookmark.id);
      
      await invoke('update_bookmark_preferences', {
        bookmarkId: editingBookmark.id,
        preferences
      });
      
      console.log('✅ DEBUG: Bookmark preferences saved successfully');
      
      await loadBookmarks(); // Refresh list
      showEditModal = false;
      editingBookmark = null;
    } catch (err) {
      console.error('❌ DEBUG: Failed to save bookmark preferences:', err);
      alert(`Ошибка при сохранении настроек: ${err}`);
    } finally {
      savingIndividual = false;
    }
  }

  function formatDate(dateString) {
    return new Date(dateString).toLocaleDateString();
  }
</script>

<div class="p-6">
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-2xl font-bold text-white">Закладки</h2>
    <button 
      on:click={() => showSettings = !showSettings}
      class="px-4 bg-white/10 hover:bg-white/20 text-white font-bold py-2 rounded transition-colors flex items-center gap-2"
    >
      ⚙️ Настройки
    </button>
  </div>
  
  <!-- Bookmark Settings Panel -->
  {#if showSettings}
  <div class="bg-[#1a1a1a] rounded-lg p-6 mb-6 border border-white/10">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-white text-lg font-bold">Настройки закладок по умолчанию</h3>
      <button 
        on:click={() => showSettings = false}
        class="text-white/60 hover:text-white text-xl"
      >
        ✕
      </button>
    </div>
    
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <!-- Quality -->
      <div>
        <label for="settings-quality" class="text-white text-sm font-medium mb-2 block">Качество по умолчанию</label>
        <select id="settings-quality" bind:value={bookmarkSettings.min_quality} class="w-full bg-[#2a2a2a] border border-white/10 rounded px-3 py-2 text-white">
          <option value="4K">4K</option>
          <option value="1080p">1080p</option>
          <option value="720p">720p</option>
          <option value="480p">480p</option>
        </select>
      </div>
      
      <!-- Source -->
      <div>
        <label for="settings-source" class="text-white text-sm font-medium mb-2 block">Предпочитаемый источник</label>
        <select id="settings-source" bind:value={bookmarkSettings.preferred_source} class="w-full bg-[#2a2a2a] border border-white/10 rounded px-3 py-2 text-white">
          <option value="Web">Web</option>
          <option value="BluRay">BluRay</option>
          <option value="BDRip">BDRip</option>
          <option value="WebRip">WebRip</option>
          <option value="Any">Любой</option>
        </select>
      </div>
      
      <!-- Preferred Audio -->
      <div>
        <label for="settings-audio" class="text-white text-sm font-medium mb-2 block">Предпочитаемая аудио дорожка</label>
        <select id="settings-audio" bind:value={bookmarkSettings.preferred_audio} class="w-full bg-[#2a2a2a] border border-white/10 rounded px-3 py-2 text-white">
          <option value="DTS-HD MA">DTS-HD MA</option>
          <option value="DTS-X">DTS-X</option>
          <option value="Dolby Atmos">Dolby Atmos</option>
          <option value="TrueHD">TrueHD</option>
          <option value="DTS">DTS</option>
          <option value="AC3">AC3</option>
          <option value="AAC">AAC</option>
          <option value="Any">Любая</option>
        </select>
      </div>
    </div>
    
    <div class="flex justify-end mt-4">
      <button 
        on:click={saveBookmarkSettings}
        disabled={savingSettings}
        class="px-6 bg-netflix-red hover:bg-red-600 disabled:bg-gray-600 text-white font-bold py-2 rounded transition-colors"
      >
        {savingSettings ? 'Сохранение...' : 'Сохранить настройки'}
      </button>
    </div>
  </div>
  {/if}

  {#if loading}
    <div class="text-white/60 text-center py-8">Загрузка закладок...</div>
  {:else if error}
    <div class="text-red-400 text-center py-8">Ошибка: {error}</div>
  {:else if bookmarks.length === 0}
    <div class="text-white/60 text-center py-8">
      <div class="text-4xl mb-4">📚</div>
      <div>Пока нет закладок</div>
      <div class="text-sm mt-2">Нажмите ⭐ на любом фильме, чтобы добавить в закладки</div>
    </div>
  {:else}
    <div class="space-y-6">
      {#each bookmarks as bookmark}
        <div class="group relative bg-gradient-to-br from-[#1a1a1a] via-[#252525] to-[#1a1a1a] rounded-xl p-6 border border-white/10 hover:border-white/20 transition-all duration-300 hover:shadow-2xl hover:shadow-netflix-red/10 hover:scale-[1.02] transform">
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-start gap-4 mb-4">
                {#if bookmark.poster_url}
                  <img 
                    src={bookmark.poster_url} 
                    alt={bookmark.title}
                    class="w-16 h-24 object-cover rounded-lg shadow-lg border-2 border-white/10 transition-transform duration-300 group-hover:scale-105"
                  />
                {/if}
                <div class="flex-1">
                  <h3 class="text-white font-bold text-xl mb-1 leading-tight group-hover:text-netflix-red transition-colors duration-200">{bookmark.title}</h3>
                  {#if bookmark.year}
                    <span class="text-white/70 text-sm font-medium">{bookmark.year}</span>
                  {/if}
                </div>
              </div>
              
              <div class="flex flex-wrap gap-3 text-xs">
                <span class="bg-gradient-to-r from-blue-500/20 to-blue-600/20 text-blue-300 px-3 py-1.5 rounded-full border border-blue-500/30 flex items-center gap-1.5 font-medium">
                  🎬 {bookmark.min_quality || 'Любое'}
                </span>
                <span class="bg-gradient-to-r from-purple-500/20 to-purple-600/20 text-purple-300 px-3 py-1.5 rounded-full border border-purple-500/30 flex items-center gap-1.5 font-medium">
                  📀 {bookmark.preferred_source || 'Любой'}
                </span>
                <span class="bg-gradient-to-r from-green-500/20 to-green-600/20 text-green-300 px-3 py-1.5 rounded-full border border-green-500/30 flex items-center gap-1.5 font-medium">
                  🔊 {bookmark.preferred_audio || 'Любое'}
                </span>
                {#if bookmark.created_at}
                  <span class="bg-white/10 text-white/60 px-3 py-1.5 rounded-full flex items-center gap-1.5">
                    📅 {formatDate(bookmark.created_at)}
                  </span>
                {/if}
              </div>
            </div>
            
            <div class="flex gap-2 ml-4">
              <button 
                on:click={() => openEditModal(bookmark)}
                class="text-blue-400 hover:text-blue-300 transition-colors"
                title="Редактировать закладку"
              >
                ✏️
              </button>
              <button 
                on:click={() => removeBookmark(bookmark.id)}
                class="text-red-400 hover:text-red-300 transition-colors"
                title="Удалить закладку"
              >
                🗑️
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Individual Bookmark Edit Modal -->
{#if showEditModal && editingBookmark}
<div 
  class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" 
  role="dialog" 
  aria-modal="true" 
  aria-labelledby="edit-modal-title"
  on:click={() => showEditModal = false}
  on:keydown={(e) => e.key === 'Escape' && (showEditModal = false)}
  tabindex="-1"
>
  <!-- eslint-disable-next-line svelte/no-noninteractive-element-interactions -->
  <!-- eslint-disable-next-line svelte/no-unnecessary-event-listener -->
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div 
    class="bg-[#1a1a1a] rounded-lg p-6 max-w-md w-full mx-4 border border-white/10" 
    role="document"
    on:click|stopPropagation
  >
    <div class="flex items-center justify-between mb-4">
      <h3 id="edit-modal-title" class="text-white text-lg font-bold">Редактировать закладку</h3>
      <button 
        on:click={() => showEditModal = false}
        class="text-white/60 hover:text-white text-xl"
        aria-label="Закрыть модальное окно"
      >
        ✕
      </button>
    </div>
    
    <div class="mb-4">
      <div class="text-white font-medium mb-2">{editingBookmark.title}</div>
      {#if editingBookmark.year}
        <div class="text-white/60 text-sm">{editingBookmark.year}</div>
      {/if}
    </div>
    
    <div class="space-y-4">
      <!-- Quality -->
      <div>
        <label for="edit-quality" class="text-white text-sm font-medium mb-2 block">Качество</label>
        <select id="edit-quality" bind:value={individualSettings.min_quality} class="w-full bg-[#2a2a2a] border border-white/10 rounded px-3 py-2 text-white">
          <option value="4K">4K</option>
          <option value="1080p">1080p</option>
          <option value="720p">720p</option>
          <option value="480p">480p</option>
        </select>
      </div>
      
      <!-- Source -->
      <div>
        <label for="edit-source" class="text-white text-sm font-medium mb-2 block">Источник</label>
        <select id="edit-source" bind:value={individualSettings.preferred_source} class="w-full bg-[#2a2a2a] border border-white/10 rounded px-3 py-2 text-white">
          <option value="Web">Web</option>
          <option value="BluRay">BluRay</option>
          <option value="BDRip">BDRip</option>
          <option value="WebRip">WebRip</option>
          <option value="Any">Любой</option>
        </select>
      </div>
      
      <!-- Audio -->
      <div>
        <label for="edit-audio" class="text-white text-sm font-medium mb-2 block">Аудио</label>
        <select id="edit-audio" bind:value={individualSettings.preferred_audio} class="w-full bg-[#2a2a2a] border border-white/10 rounded px-3 py-2 text-white">
          <option value="DTS-HD MA">DTS-HD MA</option>
          <option value="DTS-X">DTS-X</option>
          <option value="Dolby Atmos">Dolby Atmos</option>
          <option value="TrueHD">TrueHD</option>
          <option value="DTS">DTS</option>
          <option value="AC3">AC3</option>
          <option value="AAC">AAC</option>
          <option value="Any">Любая</option>
        </select>
      </div>
    </div>
    
    <div class="flex justify-end gap-3 mt-6">
      <button 
        on:click={() => showEditModal = false}
        class="px-4 bg-white/10 hover:bg-white/20 text-white font-bold py-2 rounded transition-colors"
      >
        Отмена
      </button>
      <button 
        on:click={saveIndividualSettings}
        disabled={savingIndividual}
        class="px-4 bg-netflix-red hover:bg-red-600 disabled:bg-gray-600 text-white font-bold py-2 rounded transition-colors"
      >
        {savingIndividual ? 'Сохранение...' : 'Сохранить'}
      </button>
    </div>
  </div>
</div>
{/if}

<style>
  /* Force dark styling for all select elements */
  select {
    background-color: #2a2a2a !important;
    color: white !important;
    color-scheme: dark !important;
    -webkit-appearance: none !important;
    -moz-appearance: none !important;
    appearance: none !important;
  }
  
  select option {
    background-color: #2a2a2a !important;
    color: white !important;
    color-scheme: dark !important;
  }
</style>
