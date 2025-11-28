<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  let downloads = [];
  let statistics = null;
  let loading = true;
  let error = null;
  let showStats = false;

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    try {
      loading = true;
      error = null;
      
      // Load individually to catch errors separately
      let historyData = '[]';
      let statsData = '{"total_downloads": 0, "by_quality": {}, "by_year": {}, "by_category": {}, "total_size_bytes": 0, "total_size_gb": 0}';
      
      try {
        historyData = await invoke('get_download_history');
        console.log('🔥 DEBUG: History data loaded successfully');
      } catch (historyErr) {
        console.error('❌ DEBUG: Failed to load download history:', historyErr);
        historyData = '[]'; // Fallback to empty array
      }
      
      try {
        statsData = await invoke('get_statistics');
        console.log('🔥 DEBUG: Statistics data loaded successfully');
      } catch (statsErr) {
        console.error('❌ DEBUG: Failed to load statistics:', statsErr);
        statsData = {"total_downloads": 0, "by_quality": {}, "by_year": {}, "by_category": {}, "total_size_bytes": 0, "total_size_gb": 0}; // Fallback to empty stats
      }
      
      // Use data directly (Tauru invoke already parses JSON)
      downloads = historyData || [];
      console.log('🔥 DEBUG: Downloads loaded:', downloads.length);
      
      statistics = statsData || {"total_downloads": 0, "by_quality": {}, "by_year": {}, "by_category": {}, "total_size_bytes": 0, "total_size_gb": 0};
      console.log('🔥 DEBUG: Statistics loaded:', statistics);
      
    } catch (err) {
      error = err;
      console.error('Failed to load download history:', err);
      // Set fallback values
      downloads = [];
      statistics = {
        total_downloads: 0,
        by_quality: {},
        by_year: {},
        by_category: {},
        total_size_bytes: 0,
        total_size_gb: 0
      };
    } finally {
      loading = false;
    }
  }

  function formatDate(dateString) {
    return new Date(dateString).toLocaleDateString('ru-RU', {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function formatSize(bytes) {
    if (!bytes) return 'N/A';
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
  }

  async function clearHistory() {
    if (confirm('Вы уверены, что хотите очистить всю историю скачиваний и статистику?')) {
      try {
        const result = await invoke('clear_download_history');
        
        if (result.success) {
          console.log('✅ DEBUG: History cleared successfully');
          alert(`История очищена: ${result.message}`);
          // Reload all data using the correct function name
          await loadData();
        } else {
          console.error('❌ DEBUG: Failed to clear history:', result.error);
          alert(`Ошибка при очистке истории: ${result.error}`);
        }
      } catch (error) {
        console.error('❌ DEBUG: Error clearing history:', error);
        alert(`Ошибка: ${error}`);
      }
    }
  }

  function getQualityColor(quality) {
    const colors = {
      '4K': 'text-purple-300 bg-purple-500/20 border-purple-500/30',
      '1080p': 'text-blue-300 bg-blue-500/20 border-blue-500/30',
      '720p': 'text-green-300 bg-green-500/20 border-green-500/30',
      '480p': 'text-yellow-300 bg-yellow-500/20 border-yellow-500/30'
    };
    return colors[quality] || 'text-gray-300 bg-gray-500/20 border-gray-500/30';
  }

  function getTotalSize() {
    if (!statistics?.total_downloads) return '0 GB';
    // This would need actual size calculation from statistics
    return formatSize(statistics.total_size_bytes || 0);
  }
</script>

<div class="p-6">
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-2xl font-bold text-white">История скачиваний</h2>
    <div class="flex items-center gap-2">
      <button 
        on:click={clearHistory}
        class="px-4 bg-red-500/20 hover:bg-red-500/30 text-red-300 font-bold py-2 rounded transition-colors flex items-center gap-2 border border-red-500/30"
      >
        🗑️ Очистить историю
      </button>
      <button 
        on:click={() => showStats = !showStats}
        class="px-4 bg-white/10 hover:bg-white/20 text-white font-bold py-2 rounded transition-colors flex items-center gap-2"
      >
        📊 {showStats ? 'Скрыть' : 'Показать'} статистику
      </button>
    </div>
  </div>

  <!-- Statistics Panel -->
  {#if showStats && statistics}
  <div class="bg-[#1a1a1a] rounded-lg p-6 mb-6 border border-white/10">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-white text-lg font-bold">📊 Статистика скачиваний</h3>
      <button 
        on:click={() => showStats = false}
        class="text-white/60 hover:text-white text-xl"
      >
        ✕
      </button>
    </div>
    
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
      <!-- Total Downloads -->
      <div class="bg-gradient-to-br from-netflix-red/20 to-red-600/20 rounded-lg p-4 border border-red-500/30">
        <div class="text-red-300 text-2xl mb-1">📥</div>
        <div class="text-white text-2xl font-bold">{statistics.total_downloads || 0}</div>
        <div class="text-white/60 text-sm">Всего скачиваний</div>
      </div>
      
      <!-- Total Size -->
      <div class="bg-gradient-to-br from-blue-500/20 to-blue-600/20 rounded-lg p-4 border border-blue-500/30">
        <div class="text-blue-300 text-2xl mb-1">💾</div>
        <div class="text-white text-2xl font-bold">{getTotalSize()}</div>
        <div class="text-white/60 text-sm">Общий размер</div>
      </div>
      
      <!-- Most Common Quality -->
      <div class="bg-gradient-to-br from-purple-500/20 to-purple-600/20 rounded-lg p-4 border border-purple-500/30">
        <div class="text-purple-300 text-2xl mb-1">🎬</div>
        <div class="text-white text-2xl font-bold">
          {Object.keys(statistics.by_quality || {}).length > 0 
            ? Object.entries(statistics.by_quality).sort((a, b) => b[1].count - a[1].count)[0][0] 
            : 'N/A'}
        </div>
        <div class="text-white/60 text-sm">Популярное качество</div>
      </div>
      
      <!-- Most Common Year -->
      <div class="bg-gradient-to-br from-green-500/20 to-green-600/20 rounded-lg p-4 border border-green-500/30">
        <div class="text-green-300 text-2xl mb-1">📅</div>
        <div class="text-white text-2xl font-bold">
          {Object.keys(statistics.by_year || {}).length > 0 
            ? Object.entries(statistics.by_year).sort((a, b) => b[1].count - a[1].count)[0][0] 
            : 'N/A'}
        </div>
        <div class="text-white/60 text-sm">Популярный год</div>
      </div>
    </div>
    
    <!-- Detailed Stats -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
      <!-- By Quality -->
      <div>
        <h4 class="text-white font-medium mb-3">По качеству</h4>
        <div class="space-y-2">
          {#each Object.entries(statistics.by_quality || {}).sort((a, b) => b[1].count - a[1].count) as [quality, data]}
            <div class="flex items-center justify-between">
              <span class="text-white/70 text-sm">{quality}</span>
              <span class="text-white text-sm font-medium">{data.count}</span>
            </div>
          {/each}
        </div>
      </div>
      
      <!-- By Year -->
      <div>
        <h4 class="text-white font-medium mb-3">По годам</h4>
        <div class="space-y-2 max-h-40 overflow-y-auto">
          {#each Object.entries(statistics.by_year || {}).sort((a, b) => b[1].count - a[1].count) as [year, data]}
            <div class="flex items-center justify-between">
              <span class="text-white/70 text-sm">{year}</span>
              <span class="text-white text-sm font-medium">{data.count}</span>
            </div>
          {/each}
        </div>
      </div>
      
      <!-- By Category -->
      <div>
        <h4 class="text-white font-medium mb-3">По категориям</h4>
        <div class="space-y-2 max-h-40 overflow-y-auto">
          {#each Object.entries(statistics.by_category || {}).sort((a, b) => b[1].count - a[1].count) as [category, data]}
            <div class="flex items-center justify-between">
              <span class="text-white/70 text-sm">{category}</span>
              <span class="text-white text-sm font-medium">{data.count}</span>
            </div>
          {/each}
        </div>
      </div>
    </div>
  </div>
  {/if}

  {#if loading}
    <div class="text-white/60 text-center py-8">Загрузка истории скачиваний...</div>
  {:else if error}
    <div class="text-red-400 text-center py-8">Ошибка: {error}</div>
  {:else if downloads.length === 0}
    <div class="text-white/60 text-center py-8">
      <div class="text-4xl mb-4">📥</div>
      <div>Пока нет скачиваний</div>
      <div class="text-sm mt-2">Скачайте фильмы и они появятся здесь</div>
    </div>
  {:else}
    <div class="space-y-4">
      {#each downloads as download}
        <div class="group relative bg-gradient-to-br from-[#1a1a1a] via-[#252525] to-[#1a1a1a] rounded-lg p-4 border border-white/10 hover:border-white/20 transition-all duration-300 hover:shadow-lg hover:shadow-netflix-red/10">
          <div class="flex items-center justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-4 mb-2">
                <h3 class="text-white font-bold text-lg group-hover:text-netflix-red transition-colors duration-200">
                  {download.title}
                </h3>
                {#if download.year}
                  <span class="text-white/70 text-sm font-medium">({download.year})</span>
                {/if}
                <span class="px-2 py-1 text-xs font-medium rounded-full border {getQualityColor(download.quality)}">
                  {download.quality}
                </span>
              </div>
              
              <div class="flex flex-wrap gap-3 text-xs">
                <span class="bg-white/10 text-white/60 px-3 py-1.5 rounded-full flex items-center gap-1.5">
                  📅 {formatDate(download.downloaded_at)}
                </span>
                {#if download.size_bytes}
                  <span class="bg-white/10 text-white/60 px-3 py-1.5 rounded-full flex items-center gap-1.5">
                    💾 {formatSize(download.size_bytes)}
                  </span>
                {/if}
                <span class="bg-white/10 text-white/60 px-3 py-1.5 rounded-full flex items-center gap-1.5">
                  🌐 {download.source}
                </span>
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
