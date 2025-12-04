<script lang="ts">
  import CleanupManager from './CleanupManager.svelte';
  import JobsDashboard from './JobsDashboard.svelte';

  let selectedView: 'cleanup' | 'jobs' = 'cleanup';
</script>

<div class="w-full h-full flex flex-col bg-gray-900 text-white">
  <!-- Header -->
  <div class="bg-gray-800 border-b border-gray-700 p-4">
    <h1 class="text-2xl font-bold mb-4">Library Management</h1>

    <!-- View Selector -->
    <div class="flex gap-2">
      <button
        on:click={() => (selectedView = 'cleanup')}
        class={`px-4 py-2 rounded transition ${
          selectedView === 'cleanup'
            ? 'bg-blue-600 text-white'
            : 'bg-gray-700 hover:bg-gray-600'
        }`}
      >
        🧹 Cleanup
      </button>
      <button
        on:click={() => (selectedView = 'jobs')}
        class={`px-4 py-2 rounded transition ${
          selectedView === 'jobs'
            ? 'bg-blue-600 text-white'
            : 'bg-gray-700 hover:bg-gray-600'
        }`}
      >
        ⏳ Jobs
      </button>
    </div>
  </div>

  <!-- Content -->
  <div class="flex-1 overflow-auto p-4">
    {#if selectedView === 'cleanup'}
      <CleanupManager />
    {:else if selectedView === 'jobs'}
      <JobsDashboard />
    {/if}
  </div>
</div>
