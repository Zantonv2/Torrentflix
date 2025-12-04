<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import LoadingSpinner from './LoadingSpinner.svelte';

  interface Job {
    id: number;
    job_type: string;
    status: 'scheduled' | 'running' | 'completed' | 'failed' | 'cancelled';
    progress_percent: number;
    created_at: string;
    started_at?: string;
    completed_at?: string;
    error_message?: string;
    result_summary?: string;
  }

  let jobs: Job[] = [];
  let isLoading = false;
  let error: string | null = null;
  let selectedTab: 'running' | 'completed' | 'all' = 'running';
  let autoRefresh = true;
  let refreshInterval: number | null = null;

  onMount(async () => {
    await loadJobs();
    if (autoRefresh) {
      refreshInterval = window.setInterval(loadJobs, 2000);
    }
  });

  onDestroy(() => {
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
  });

  async function loadJobs() {
    // This would be implemented in the engine
    // For now, we'll show a placeholder
    isLoading = false;
    jobs = [];
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'running':
        return 'bg-blue-900 text-blue-200';
      case 'completed':
        return 'bg-green-900 text-green-200';
      case 'failed':
        return 'bg-red-900 text-red-200';
      case 'cancelled':
        return 'bg-gray-700 text-gray-200';
      case 'scheduled':
        return 'bg-yellow-900 text-yellow-200';
      default:
        return 'bg-gray-700 text-gray-200';
    }
  }

  function getStatusIcon(status: string): string {
    switch (status) {
      case 'running':
        return '⏳';
      case 'completed':
        return '✓';
      case 'failed':
        return '✕';
      case 'cancelled':
        return '⊘';
      case 'scheduled':
        return '⏱';
      default:
        return '?';
    }
  }

  function getJobTypeLabel(type: string): string {
    return type
      .split('_')
      .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ');
  }

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleString();
  }

  function getElapsedTime(startedAt: string, completedAt?: string): string {
    const start = new Date(startedAt).getTime();
    const end = completedAt ? new Date(completedAt).getTime() : Date.now();
    const elapsed = Math.floor((end - start) / 1000);

    if (elapsed < 60) return `${elapsed}s`;
    if (elapsed < 3600) return `${Math.floor(elapsed / 60)}m`;
    return `${Math.floor(elapsed / 3600)}h`;
  }

  function filteredJobs(): Job[] {
    switch (selectedTab) {
      case 'running':
        return jobs.filter((j) => j.status === 'running' || j.status === 'scheduled');
      case 'completed':
        return jobs.filter((j) => j.status === 'completed' || j.status === 'failed' || j.status === 'cancelled');
      default:
        return jobs;
    }
  }

  function toggleAutoRefresh() {
    autoRefresh = !autoRefresh;
    if (autoRefresh) {
      refreshInterval = window.setInterval(loadJobs, 2000);
    } else if (refreshInterval) {
      clearInterval(refreshInterval);
      refreshInterval = null;
    }
  }
</script>

<div class="jobs-dashboard w-full text-white">
  {#if error}
    <div class="bg-red-900 border border-red-700 text-red-100 px-4 py-3 rounded mb-4">
      {error}
    </div>
  {/if}

  <!-- Header -->
  <div class="bg-gray-800 p-4 rounded mb-4 flex justify-between items-center">
    <h2 class="text-2xl font-semibold">Background Jobs</h2>
    <div class="flex gap-2">
      <button
        on:click={toggleAutoRefresh}
        class={`px-4 py-2 rounded transition ${
          autoRefresh ? 'bg-green-600 hover:bg-green-700' : 'bg-gray-700 hover:bg-gray-600'
        }`}
      >
        {autoRefresh ? '🔄 Auto-refresh' : '⏸ Paused'}
      </button>
      <button
        on:click={loadJobs}
        class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition"
      >
        🔄 Refresh
      </button>
    </div>
  </div>

  <!-- Tabs -->
  <div class="flex gap-2 border-b border-gray-700 mb-4">
    <button
      on:click={() => (selectedTab = 'running')}
      class={`px-4 py-2 font-semibold transition ${
        selectedTab === 'running'
          ? 'border-b-2 border-blue-500 text-blue-400'
          : 'text-gray-400 hover:text-white'
      }`}
    >
      Running
      {#if jobs.filter((j) => j.status === 'running' || j.status === 'scheduled').length > 0}
        <span class="ml-2 bg-blue-600 px-2 py-0.5 rounded text-xs">
          {jobs.filter((j) => j.status === 'running' || j.status === 'scheduled').length}
        </span>
      {/if}
    </button>
    <button
      on:click={() => (selectedTab = 'completed')}
      class={`px-4 py-2 font-semibold transition ${
        selectedTab === 'completed'
          ? 'border-b-2 border-blue-500 text-blue-400'
          : 'text-gray-400 hover:text-white'
      }`}
    >
      Completed
      {#if jobs.filter((j) => j.status === 'completed' || j.status === 'failed' || j.status === 'cancelled').length > 0}
        <span class="ml-2 bg-blue-600 px-2 py-0.5 rounded text-xs">
          {jobs.filter((j) => j.status === 'completed' || j.status === 'failed' || j.status === 'cancelled').length}
        </span>
      {/if}
    </button>
    <button
      on:click={() => (selectedTab = 'all')}
      class={`px-4 py-2 font-semibold transition ${
        selectedTab === 'all'
          ? 'border-b-2 border-blue-500 text-blue-400'
          : 'text-gray-400 hover:text-white'
      }`}
    >
      All
      {#if jobs.length > 0}
        <span class="ml-2 bg-blue-600 px-2 py-0.5 rounded text-xs">{jobs.length}</span>
      {/if}
    </button>
  </div>

  <!-- Content -->
  {#if isLoading}
    <div class="flex justify-center items-center py-12">
      <LoadingSpinner />
    </div>
  {:else if filteredJobs().length === 0}
    <div class="text-center py-12 text-gray-400">
      <p class="text-lg mb-2">No jobs</p>
      <p class="text-sm">
        {selectedTab === 'running'
          ? 'No running jobs at the moment'
          : selectedTab === 'completed'
            ? 'No completed jobs yet'
            : 'No jobs found'}
      </p>
    </div>
  {:else}
    <div class="space-y-3">
      {#each filteredJobs() as job (job.id)}
        <div class="bg-gray-800 p-4 rounded border border-gray-700 hover:border-gray-600 transition">
          <!-- Header -->
          <div class="flex items-start justify-between mb-3">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-1">
                <span class="text-xl">{getStatusIcon(job.status)}</span>
                <h3 class="font-semibold text-lg">{getJobTypeLabel(job.job_type)}</h3>
                <span class={`px-2 py-0.5 rounded text-xs font-semibold ${getStatusColor(job.status)}`}>
                  {job.status}
                </span>
              </div>
              <p class="text-sm text-gray-400">
                Created: {formatDate(job.created_at)}
                {#if job.started_at}
                  • Started: {formatDate(job.started_at)}
                {/if}
              </p>
            </div>

            <!-- Progress -->
            {#if job.status === 'running' || job.status === 'scheduled'}
              <div class="text-right">
                <p class="text-2xl font-bold text-blue-400">{job.progress_percent}%</p>
              </div>
            {/if}
          </div>

          <!-- Progress Bar -->
          {#if job.status === 'running' || job.status === 'scheduled'}
            <div class="mb-3">
              <div class="w-full bg-gray-700 rounded-full h-2">
                <div
                  class="bg-blue-600 h-2 rounded-full transition-all"
                  style="width: {job.progress_percent}%"
                ></div>
              </div>
            </div>
          {/if}

          <!-- Details -->
          <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 text-sm mb-3">
            {#if job.started_at}
              <div>
                <p class="text-gray-400">Elapsed</p>
                <p class="font-semibold">{getElapsedTime(job.started_at, job.completed_at)}</p>
              </div>
            {/if}

            {#if job.completed_at}
              <div>
                <p class="text-gray-400">Completed</p>
                <p class="font-semibold">{formatDate(job.completed_at)}</p>
              </div>
            {/if}

            {#if job.result_summary}
              <div class="col-span-2 sm:col-span-2">
                <p class="text-gray-400">Result</p>
                <p class="font-semibold truncate">{job.result_summary}</p>
              </div>
            {/if}
          </div>

          <!-- Error Message -->
          {#if job.error_message}
            <div class="bg-red-900 bg-opacity-30 border border-red-700 p-2 rounded text-sm text-red-200 mb-3">
              <p class="font-semibold mb-1">Error:</p>
              <p class="truncate">{job.error_message}</p>
            </div>
          {/if}

          <!-- Actions -->
          {#if job.status === 'running'}
            <div class="flex gap-2">
              <button
                class="px-4 py-2 bg-red-600 hover:bg-red-700 rounded text-sm transition"
              >
                Cancel
              </button>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  <!-- Stats Footer -->
  {#if jobs.length > 0}
    <div class="mt-6 pt-4 border-t border-gray-700 grid grid-cols-2 sm:grid-cols-4 gap-4 text-sm">
      <div>
        <p class="text-gray-400">Total Jobs</p>
        <p class="text-lg font-semibold">{jobs.length}</p>
      </div>
      <div>
        <p class="text-gray-400">Running</p>
        <p class="text-lg font-semibold text-blue-400">
          {jobs.filter((j) => j.status === 'running').length}
        </p>
      </div>
      <div>
        <p class="text-gray-400">Completed</p>
        <p class="text-lg font-semibold text-green-400">
          {jobs.filter((j) => j.status === 'completed').length}
        </p>
      </div>
      <div>
        <p class="text-gray-400">Failed</p>
        <p class="text-lg font-semibold text-red-400">
          {jobs.filter((j) => j.status === 'failed').length}
        </p>
      </div>
    </div>
  {/if}
</div>

<style>
  .jobs-dashboard {
    display: flex;
    flex-direction: column;
  }
</style>
