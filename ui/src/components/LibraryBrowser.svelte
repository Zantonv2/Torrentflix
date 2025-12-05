<script lang="ts">
    import Loading from "./core/Loading.svelte";

    let isLoading = $state(false);
</script>

<div class="w-full h-full overflow-y-auto p-6">
    {#if isLoading}
        <div class="flex items-center justify-center h-full">
            <Loading type="spinner" />
        </div>
    {:else}
        <div class="text-center py-12">
            <p class="text-netflix-light text-lg">Library browser</p>
            <p class="text-sm text-gray-500 mt-2">
                Your library will appear here
            </p>
        </div>
    {/if}
</div>

<style>
    :global(.netflix-light) {
        color: #757575;
    }
</style>
