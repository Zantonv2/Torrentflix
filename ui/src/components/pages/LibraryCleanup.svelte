<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import Loading from '../core/Loading.svelte';
  import Modal from '../core/Modal.svelte';
  import Button from '../core/Button.svelte';

  interface CleanupCandidate {
    id: number;
    file_path: string;
    file_size: number;
    reason: string;
    media_title?: string;
  }

  let candidates: CleanupCandidate[] = [];
  let selectedCandidates: Set<number> = new Set();
  let isLoading = false;
  let isExecuting = false;
  let error: string | null = null;
  let cleanupResult: any = null;
  let totalSpaceSavings = 0;
  let showConfirmationModal = false;

  onMount(async () => {
    await loadCandidates();
  });

  async function loadCandidates() {
    isLoading = true;
    error = null;

    try {
      const result = await invoke<any[]>('get_cleanup_candidates');
      candidates = result || [];
      calculateSpaceSavings();
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      error = `Ошибка загрузки кандидатов на очистку: ${errorMsg}`;
      console.error('Error:', err);
    } finally {
      isLoading = false;
    }
  }

  function calculateSpaceSavings() {
    totalSpaceSavings = candidates
      .filter((c) => selectedCandidates.has(c.id))
      .reduce((sum, c) => sum + c.file_size, 0);
  }

  function toggleCandidate(id: number) {
    if (selectedCandidates.has(id)) {
      selectedCandidates.delete(id);
    } else {
      selectedCandidates.add(id);
    }
    selectedCandidates = selectedCandidates;
    calculateSpaceSavings();
  }

  function toggleAll() {
    if (selectedCandidates.size === candidates.length) {
      selectedCandidates.clear();
    } else {
      candidates.forEach((c) => selectedCandidates.add(c.id));
    }
    selectedCandidates = selectedCandidates;
    calculateSpaceSavings();
  }

  function openConfirmationModal() {
    if (selectedCandidates.size === 0) {
      error = 'Пожалуйста, выберите хотя бы один элемент для очистки';
      return;
    }
    showConfirmationModal = true;
  }

  function closeConfirmationModal() {
    showConfirmationModal = false;
  }

  async function executeCleanup() {
    isExecuting = true;
    error = null;
    cleanupResult = null;
    showConfirmationModal = false;

    try {
      const candidateIds = Array.from(selectedCandidates);
      const result = await invoke<any>('execute_cleanup', {
        candidate_ids: candidateIds,
      });

      cleanupResult = result;
      selectedCandidates.clear();
      selectedCandidates = selectedCandidates;
      await loadCandidates();
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      error = `Ошибка очистки: ${errorMsg}`;
      console.error('Error:', err);
    } finally {
      isExecuting = false;
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  }

  function getReasonColor(reason: string): string {
    if (reason.includes('duplicate')) return 'bg-red-900 text-red-200';
    if (reason.includes('low-quality')) return 'bg-orange-900 text-orange-200';
    if (reason.includes('trashed')) return 'bg-yellow-900 text-yellow-200';
    if (reason.includes('missing')) return 'bg-gray-700 text-gray-200';
    return 'bg-gray-600 text-gray-200';
  }
</script>

<div class="library-cleanup w-full h-full text-white overflow-y-auto">
  <div class="p-6 space-y-6">
    {#if error}
      <div class="bg-red-900 border border-red-700 text-red-100 px-4 py-3 rounded">
        {error}
      </div>
    {/if}

    {#if cleanupResult}
      <div class="bg-green-900 border border-green-700 text-green-100 px-4 py-3 rounded">
        <h3 class="font-semibold mb-2">✅ Очистка завершена</h3>
        <div class="space-y-1 text-sm">
          <p>Удалено: {cleanupResult.deleted_count || 0} элементов</p>
          <p>Освобождено: {formatBytes(cleanupResult.freed_space || 0)}</p>
          {#if cleanupResult.errors && cleanupResult.errors.length > 0}
            <p class="text-yellow-200">Ошибок: {cleanupResult.errors.length}</p>
          {/if}
        </div>
        <button
          on:click={() => (cleanupResult = null)}
          class="mt-3 px-4 py-2 bg-green-700 hover:bg-green-600 rounded transition"
          aria-label="Закрыть результат"
        >
          Закрыть
        </button>
      </div>
    {/if}

    {#if isLoading}
      <div class="flex justify-center items-center py-12">
        <Loading type="spinner" />
      </div>
    {:else if candidates.length === 0}
      <div class="text-center py-12 text-gray-400">
        <p class="text-lg mb-2">Кандидатов на очистку не найдено</p>
        <p class="text-sm">Ваша библиотека чистая!</p>
      </div>
    {:else}
      <div class="space-y-4">
        <!-- Summary -->
        <div class="bg-gray-800 p-4 rounded">
          <h3 class="font-semibold mb-3">Сводка очистки</h3>
          <div class="grid grid-cols-2 sm:grid-cols-4 gap-4 text-sm">
            <div>
              <p class="text-gray-400">Всего кандидатов</p>
              <p class="text-lg font-semibold">{candidates.length}</p>
            </div>
            <div>
              <p class="text-gray-400">Выбрано</p>
              <p class="text-lg font-semibold text-blue-400">{selectedCandidates.size}</p>
            </div>
            <div>
              <p class="text-gray-400">Место для освобождения</p>
              <p class="text-lg font-semibold text-green-400">{formatBytes(totalSpaceSavings)}</p>
            </div>
            <div>
              <p class="text-gray-400">Общий размер</p>
              <p class="text-lg font-semibold">
                {formatBytes(candidates.reduce((sum, c) => sum + c.file_size, 0))}
              </p>
            </div>
          </div>
        </div>

        <!-- Selection Controls -->
        <div class="flex gap-2 flex-wrap">
          <button
            on:click={toggleAll}
            class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition"
            aria-label={selectedCandidates.size === candidates.length ? 'Отменить выбор всех' : 'Выбрать все'}
          >
            {selectedCandidates.size === candidates.length ? 'Отменить выбор всех' : 'Выбрать все'}
          </button>
          <button
            on:click={() => {
              selectedCandidates.clear();
              selectedCandidates = selectedCandidates;
              calculateSpaceSavings();
            }}
            class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition"
            aria-label="Очистить выбор"
          >
            Очистить выбор
          </button>
          <button
            on:click={openConfirmationModal}
            disabled={selectedCandidates.size === 0 || isExecuting}
            class="ml-auto px-6 py-2 bg-netflix-red hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed rounded transition font-semibold"
            aria-label="Выполнить очистку"
          >
            {isExecuting ? 'Очистка...' : 'Выполнить очистку'}
          </button>
        </div>

        <!-- Candidates List -->
        <div class="space-y-2 max-h-96 overflow-y-auto">
          {#each candidates as candidate (candidate.id)}
            <div
              class={`p-3 rounded border-2 cursor-pointer transition ${
                selectedCandidates.has(candidate.id)
                  ? 'border-blue-500 bg-gray-700'
                  : 'border-gray-600 bg-gray-800 hover:border-gray-500'
              }`}
              on:click={() => toggleCandidate(candidate.id)}
              role="button"
              tabindex="0"
              on:keydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  toggleCandidate(candidate.id);
                }
              }}
            >
              <div class="flex items-start gap-3">
                <!-- Checkbox -->
                <input
                  type="checkbox"
                  checked={selectedCandidates.has(candidate.id)}
                  on:change={() => toggleCandidate(candidate.id)}
                  class="mt-1 w-4 h-4 cursor-pointer"
                  aria-label="Выбрать {candidate.media_title || candidate.file_path}"
                />

                <!-- Content -->
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-1">
                    {#if candidate.media_title}
                      <p class="font-semibold truncate">{candidate.media_title}</p>
                    {/if}
                    <span
                      class={`px-2 py-0.5 rounded text-xs font-semibold whitespace-nowrap ${getReasonColor(candidate.reason)}`}
                    >
                      {candidate.reason}
                    </span>
                  </div>
                  <p class="text-sm text-gray-400 truncate">{candidate.file_path}</p>
                  <p class="text-sm text-gray-500">{formatBytes(candidate.file_size)}</p>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- Confirmation Modal -->
<Modal
  open={showConfirmationModal}
  title="Подтверждение очистки"
  onClose={closeConfirmationModal}
>
  <div class="space-y-4">
    <p class="text-white">
      Вы уверены? Это удалит <span class="font-bold">{selectedCandidates.size}</span> файлов и освободит
      <span class="font-bold text-green-400">{formatBytes(totalSpaceSavings)}</span> места.
    </p>
    <p class="text-sm text-gray-400">Это действие нельзя отменить.</p>
  </div>

  <div slot="footer" class="flex gap-3">
    <button
      on:click={closeConfirmationModal}
      class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition"
      aria-label="Отмена"
    >
      Отмена
    </button>
    <button
      on:click={executeCleanup}
      disabled={isExecuting}
      class="px-4 py-2 bg-netflix-red hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed rounded transition font-semibold"
      aria-label="Удалить"
    >
      {isExecuting ? 'Удаление...' : 'Удалить'}
    </button>
  </div>
</Modal>

<style>
  .library-cleanup {
    display: flex;
    flex-direction: column;
  }

  :global(.netflix-red) {
    color: #e50914;
  }
</style>
