<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { settingsStore, setSettings } from "../../stores/settingsStore";
    import type { Settings } from "../../stores/settingsStore";

    let loading = false;
    let error: string | null = null;

    async function loadSettings() {
        loading = true;
        try {
            const response = await invoke("get_settings");
            setSettings(response as Settings);
        } catch (err) {
            const errorMsg = err instanceof Error ? err.message : String(err);
            error = errorMsg;
            console.error("Failed to load settings:", err);
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        loadSettings();
    });
</script>

<div class="flex flex-col h-full bg-netflix-dark-bg overflow-hidden p-6">
    {#if loading}
        <div class="flex items-center justify-center flex-1">
            <div class="text-center">
                <div
                    class="animate-spin rounded-full h-12 w-12 border-b-2 border-netflix-red mx-auto mb-4"
                ></div>
                <p class="text-white/70">Loading settings...</p>
            </div>
        </div>
    {:else if error}
        <div class="flex items-center justify-center flex-1">
            <div class="text-center">
                <p class="text-netflix-red mb-4">Error loading settings</p>
                <p class="text-white/70">{error}</p>
            </div>
        </div>
    {:else}
        <div class="max-w-4xl mx-auto">
            <h1 class="text-3xl font-bold text-white mb-8">⚙️ Settings</h1>
            <div
                class="bg-netflix-card-bg border border-netflix-border rounded-lg p-6"
            >
                <p class="text-white/70">
                    Settings page is under development. Check back soon!
                </p>
            </div>
        </div>
    {/if}
</div>

<style>
    :global(.netflix-dark-bg) {
        background-color: #0f0f0f;
    }

    :global(.netflix-card-bg) {
        background-color: #1a1a1a;
    }

    :global(.netflix-border) {
        border-color: rgba(255, 255, 255, 0.1);
    }

    :global(.netflix-red) {
        color: #e50914;
    }
</style>
