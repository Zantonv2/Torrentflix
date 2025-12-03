<script lang="ts">
  export let title: string;
  export let description: string = '';
</script>

<div class="bg-gray-800 rounded-lg p-6 border border-gray-700">
  <div class="mb-4">
    <h2 class="text-lg font-semibold text-white mb-1">{title}</h2>
    {#if description}
      <p class="text-sm text-gray-400">{description}</p>
    {/if}
  </div>
  <slot />
</div>

<style>
  :global(.bg-gray-800) {
    background-color: #2a2a2a;
  }

  :global(.border-gray-700) {
    border-color: #404040;
  }
</style>
