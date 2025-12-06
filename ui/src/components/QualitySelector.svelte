<script lang="ts">
    import type { QualityVersion } from "../types";
    import Badge from "./core/Badge.svelte";

    interface Props {
        qualities: QualityVersion[];
        onSelect: (quality: QualityVersion) => void;
    }

    const { qualities, onSelect }: Props = $props();

    // Sort qualities by resolution (highest first)
    const sortedQualities = $derived.by(() => {
        const resolutionOrder: Record<string, number> = {
            "4K": 4,
            "1080p": 3,
            "720p": 2,
            SD: 1,
        };

        return [...qualities].sort((a, b) => {
            const orderA = resolutionOrder[a.resolution] || 0;
            const orderB = resolutionOrder[b.resolution] || 0;
            return orderB - orderA;
        });
    });

    function formatFileSize(sizeGb: number): string {
        return `${sizeGb.toFixed(2)} GB`;
    }

    function getResolutionBadgeVariant(
        resolution: string,
    ): "success" | "info" | "warning" | "error" {
        switch (resolution) {
            case "4K":
                return "success";
            case "1080p":
                return "info";
            case "720p":
                return "warning";
            default:
                return "error";
        }
    }
</script>

<div class="space-y-2">
    <h3 class="text-sm font-semibold text-gray-300 mb-3">
        Available Qualities
    </h3>

    {#if sortedQualities.length === 0}
        <p class="text-sm text-gray-500">No quality versions available</p>
    {:else if sortedQualities.length <= 5}
        <!-- Display all qualities if 5 or fewer -->
        <div class="space-y-2">
            {#each sortedQualities as quality (quality.magnet_link)}
                <button
                    type="button"
                    class="w-full p-3 rounded border border-gray-700 hover:border-gray-500 hover:bg-gray-800 transition-colors text-left focus:outline-none focus:ring-2 focus:ring-red-600"
                    onclick={() => onSelect(quality)}
                    aria-label={`Select ${quality.resolution} quality`}
                >
                    <div class="flex items-center justify-between gap-3">
                        <div class="flex-1">
                            <div class="flex items-center gap-2 mb-1">
                                <Badge
                                    variant={getResolutionBadgeVariant(
                                        quality.resolution,
                                    )}
                                    size="sm"
                                >
                                    {quality.resolution}
                                </Badge>
                                {#if quality.is_recommended}
                                    <span
                                        class="text-xs font-semibold text-green-400"
                                    >
                                        ⭐ Recommended
                                    </span>
                                {/if}
                            </div>
                            <div class="text-xs text-gray-400 space-y-1">
                                <div>
                                    Size: {formatFileSize(quality.file_size_gb)}
                                </div>
                                <div>
                                    Seeders: {quality.seeders} | Leechers:
                                    {quality.leechers}
                                </div>
                            </div>
                        </div>
                        <div class="text-gray-500">→</div>
                    </div>
                </button>
            {/each}
        </div>
    {:else}
        <!-- Scrollable list if more than 5 options -->
        <div
            class="max-h-64 overflow-y-auto space-y-2 border border-gray-700 rounded p-2"
        >
            {#each sortedQualities as quality (quality.magnet_link)}
                <button
                    type="button"
                    class="w-full p-2 rounded border border-gray-700 hover:border-gray-500 hover:bg-gray-800 transition-colors text-left text-sm focus:outline-none focus:ring-2 focus:ring-red-600"
                    onclick={() => onSelect(quality)}
                    aria-label={`Select ${quality.resolution} quality`}
                >
                    <div class="flex items-center justify-between gap-2">
                        <div class="flex-1">
                            <div class="flex items-center gap-2 mb-1">
                                <Badge
                                    variant={getResolutionBadgeVariant(
                                        quality.resolution,
                                    )}
                                    size="sm"
                                >
                                    {quality.resolution}
                                </Badge>
                                {#if quality.is_recommended}
                                    <span class="text-xs text-green-400">
                                        ⭐
                                    </span>
                                {/if}
                            </div>
                            <div class="text-xs text-gray-500">
                                {formatFileSize(quality.file_size_gb)} • S:{quality.seeders}
                                L:{quality.leechers}
                            </div>
                        </div>
                        <div class="text-gray-500">→</div>
                    </div>
                </button>
            {/each}
        </div>
    {/if}
</div>

<style>
    ::-webkit-scrollbar {
        width: 6px;
    }

    ::-webkit-scrollbar-track {
        background: transparent;
    }

    ::-webkit-scrollbar-thumb {
        background: #4b5563;
        border-radius: 3px;
    }

    ::-webkit-scrollbar-thumb:hover {
        background: #6b7280;
    }
</style>
