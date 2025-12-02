<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  
  interface Download {
    hash: string;
    title: string;
    progress: number;
    speed: number;
    eta: number;
    seeders: number;
    leechers: number;
    state: string;
    size: number;
  }
  
  let downloads: Download[] = [];
  let loading = true;
  let error: string | null = null;
  let pollInterval: number | null = null;
  
  async function loadDownloads() {
    try {
      const result = await invoke('get_active_downloads');
      downloads = result as Download[];
      error = null;
    } catch (e) {
      error = e as string;
      console.error('Failed to load downloads:', e);
    } finally {
      loading = false;
    }
  }
  
  async function pauseDownload(hash: string) {
    try {
      await invoke('pause_download', { hash });
      await loadDownloads();
    } catch (e) {
      console.error('Failed to pause:', e);
    }
  }
  
  async function resumeDownload(hash: string) {
    try {
      await invoke('resume_download', { hash });
      await loadDownloads();
    } catch (e) {
      console.error('Failed to resume:', e);
    }
  }
  
  async function deleteDownload(hash: string) {
    if (!confirm('Удалить этот торрент?')) return;
    
    try {
      await invoke('delete_download', { hash, deleteFiles: false });
      await loadDownloads();
    } catch (e) {
      console.error('Failed to delete:', e);
    }
  }
  
  function formatSpeed(bytesPerSec: number): string {
    const mbps = bytesPerSec / (1024 * 1024);
    return `${mbps.toFixed(1)} МБ/с`;
  }
  
  function formatEta(seconds: number): string {
    if (seconds < 0 || seconds === 8640000) return '∞';
    const hours = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    if (hours > 0) return `${hours}ч ${mins}м`;
    return `${mins}м`;
  }
  
  function formatSize(bytes: number): string {
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(2)} ГБ`;
  }
  
  function getStateLabel(state: string): string {
    const states: Record<string, string> = {
      'downloading': 'Загрузка',
      'uploading': 'Раздача',
      'pausedDL': 'Пауза',
      'pausedUP': 'Пауза',
      'queuedDL': 'В очереди',
      'queuedUP': 'В очереди',
      'stalledDL': 'Остановлено',
      'stalledUP': 'Остановлено',
      'checkingDL': 'Проверка',
      'checkingUP': 'Проверка',
      'metaDL': 'Метаданные',
      'allocating': 'Выделение',
      'error': 'Ошибка',
      'missingFiles': 'Файлы отсутствуют',
    };
    return states[state] || state;
  }
  
  function getStateColor(state: string): string {
    if (state.includes('downloading') || state.includes('metaDL')) return 'text-green-400';
    if (state.includes('paused')) return 'text-yellow-400';
    if (state.includes('error') || state.includes('missing')) return 'text-red-400';
    if (state.includes('uploading')) return 'text-blue-400';
    return 'text-gray-400';
  }
  
  onMount(() => {
    loadDownloads();
    pollInterval = window.setInterval(loadDownloads, 2000);
  });
  
  onDestroy(() => {
    if (pollInterval) clearInterval(pollInterval);
  });
</script>

<div class="flex-1 overflow-y-auto p-6">
  <div class="max-w-6xl mx-auto">
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-2xl font-bold text-white">Загрузки</h2>
      <button 
        on:click={loadDownloads}
        class="px-4 py-2 bg-netflix-gray rounded hover:bg-gray-700 transition-colors text-sm"
      >
        🔄 Обновить
      </button>
    </div>
    
    {#if loading && downloads.length === 0}
      <div class="flex items-center justify-center py-20">
        <div class="text-center">
          <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-netflix-red mx-auto mb-4"></div>
          <p class="text-gray-400">Загрузка...</p>
        </div>
      </div>
    {:else if error}
      <div class="bg-red-900/20 border border-red-500/50 rounded-lg p-6 text-center">
        <p class="text-red-400 mb-2">❌ Ошибка загрузки</p>
        <p class="text-sm text-gray-400">{error}</p>
        <button 
          on:click={loadDownloads}
          class="mt-4 px-4 py-2 bg-netflix-red rounded hover:bg-red-700 transition-colors"
        >
          Повторить
        </button>
      </div>
    {:else if downloads.length === 0}
      <div class="text-center py-20">
        <div class="text-6xl mb-4">📥</div>
        <p class="text-xl text-gray-400 mb-2">Нет активных загрузок</p>
        <p class="text-sm text-gray-500">Загрузки появятся здесь автоматически</p>
      </div>
    {:else}
      <div class="space-y-4">
        {#each downloads as download (download.hash)}
          <div class="bg-netflix-gray rounded-lg p-5 hover:bg-gray-800 transition-colors">
            <!-- Title and Actions -->
            <div class="flex justify-between items-start mb-3">
              <div class="flex-1 mr-4">
                <h3 class="font-semibold text-white text-lg mb-1">{download.title}</h3>
                <div class="flex items-center gap-3 text-sm">
                  <span class={getStateColor(download.state)}>
                    {getStateLabel(download.state)}
                  </span>
                  <span class="text-gray-500">•</span>
                  <span class="text-gray-400">{formatSize(download.size)}</span>
                </div>
              </div>
              
              <div class="flex gap-2">
                {#if download.state.includes('paused')}
                  <button 
                    on:click={() => resumeDownload(download.hash)}
                    class="px-3 py-1.5 bg-green-600 rounded hover:bg-green-700 transition-colors text-sm"
                    title="Возобновить"
                  >
                    ▶️
                  </button>
                {:else if download.state.includes('downloading') || download.state.includes('metaDL')}
                  <button 
                    on:click={() => pauseDownload(download.hash)}
                    class="px-3 py-1.5 bg-yellow-600 rounded hover:bg-yellow-700 transition-colors text-sm"
                    title="Пауза"
                  >
                    ⏸️
                  </button>
                {/if}
                <button 
                  on:click={() => deleteDownload(download.hash)}
                  class="px-3 py-1.5 bg-red-600 rounded hover:bg-red-700 transition-colors text-sm"
                  title="Удалить"
                >
                  🗑️
                </button>
              </div>
            </div>
            
            <!-- Progress Bar -->
            <div class="mb-3">
              <div class="flex justify-between text-sm mb-1.5">
                <span class="text-gray-300">{(download.progress * 100).toFixed(1)}%</span>
                <div class="flex items-center gap-3 text-gray-400">
                  {#if download.speed > 0}
                    <span>↓ {formatSpeed(download.speed)}</span>
                  {/if}
                  {#if download.eta > 0 && download.eta < 8640000}
                    <span>⏱️ {formatEta(download.eta)}</span>
                  {/if}
                </div>
              </div>
              <div class="w-full bg-gray-700 rounded-full h-2.5 overflow-hidden">
                <div 
                  class="bg-gradient-to-r from-netflix-red to-red-600 h-2.5 rounded-full transition-all duration-300"
                  style="width: {download.progress * 100}%"
                ></div>
              </div>
            </div>
            
            <!-- Stats -->
            <div class="flex gap-6 text-sm text-gray-400">
              <span title="Сиды">🌱 {download.seeders}</span>
              <span title="Личи">🔻 {download.leechers}</span>
              <span class="text-gray-600">•</span>
              <span class="text-xs text-gray-500 font-mono">{download.hash.substring(0, 16)}...</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  :global(.bg-netflix-gray) {
    background-color: #2F2F2F;
  }
  
  :global(.bg-netflix-red) {
    background-color: #E50914;
  }
</style>
