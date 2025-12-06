<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { UiSearchResult, QualityVersion } from "../types";
    import Button from "./core/Button.svelte";
    import Badge from "./core/Badge.svelte";
    import QualitySelector from "./QualitySelector.svelte";

    interface Props {
        movie: UiSearchResult;
        isOpen?: boolean;
    }

    const { movie, isOpen = false }: Props = $props();
    const dispatch = createEventDispatcher<{
        close: void;
        download: { movie: UiSearchResult; quality?: QualityVersion };
        bookmark: UiSearchResult;
    }>();

    let modalElement = $state<HTMLDivElement | undefined>();
    let previousFocus: HTMLElement | null = null;
    let selectedQuality = $state<QualityVersion | null>(null);

    function handleClose() {
        dispatch("close");
    }

    function handleDownload() {
        dispatch("download", { movie, quality: selectedQuality || undefined });
    }

    function handleQualitySelect(event: CustomEvent<QualityVersion>) {
        selectedQuality = event.detail;
    }

    function handleBookmark() {
        dispatch("bookmark", movie);
    }

    function handleEscape(e: KeyboardEvent) {
        if (e.key === "Escape" && isOpen) {
            handleClose();
        }
    }

    function handleBackdropClick(e: MouseEvent) {
        if (e.target === e.currentTarget) {
            handleClose();
        }
    }

    function formatDuration(minutes?: number): string {
        if (!minutes) return "N/A";
        const hours = Math.floor(minutes / 60);
        const mins = minutes % 60;
        return `${hours}h ${mins}m`;
    }

    function formatFileSize(sizeGb?: number): string {
        if (!sizeGb) return "N/A";
        return `${sizeGb.toFixed(2)} GB`;
    }

    $effect(() => {
        if (isOpen) {
            previousFocus = document.activeElement as HTMLElement;
            document.addEventListener("keydown", handleEscape);
            const firstFocusable = modalElement?.querySelector(
                'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
            ) as HTMLElement;
            firstFocusable?.focus();
        } else {
            document.removeEventListener("keydown", handleEscape);
            if (previousFocus) {
                previousFocus.focus();
            }
        }

        return () => {
            document.removeEventListener("keydown", handleEscape);
        };
    });
</script>

{#if isOpen}
    <div
        class="fixed inset-0 bg-black bg-opacity-50 backdrop-blur-sm flex items-center justify-center z-50 animate-fade-in"
        onclick={handleBackdropClick}
        role="presentation"
    >
        <div
            bind:this={modalElement}
            class="bg-netflix-dark rounded-lg shadow-2xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto animate-slide-up"
            role="dialog"
            aria-modal="true"
            aria-labelledby="detail-panel-title"
        >
            <!-- Backdrop Image -->
            {#if movie.backdrop_url}
                <div class="relative w-full h-48 overflow-hidden rounded-t-lg">
                    <img
                        src={movie.backdrop_url}
                        alt={movie.title}
                        class="w-full h-full object-cover"
                        loading="lazy"
                    />
                    <div
                        class="absolute inset-0 bg-gradient-to-b from-transparent to-netflix-dark"
                    ></div>
                </div>
            {:else if movie.poster_url}
                <div class="relative w-full h-48 overflow-hidden rounded-t-lg">
                    <img
                        src={movie.poster_url}
                        alt={movie.title}
                        class="w-full h-full object-cover"
                        loading="lazy"
                    />
                    <div
                        class="absolute inset-0 bg-gradient-to-b from-transparent to-netflix-dark"
                    ></div>
                </div>
            {/if}

            <!-- Content -->
            <div class="p-6 space-y-4">
                <!-- Title and Close Button -->
                <div class="flex items-start justify-between gap-4">
                    <div class="flex-1">
                        <h2
                            id="detail-panel-title"
                            class="text-2xl font-bold text-white mb-2"
                        >
                            {movie.title}
                        </h2>
                        {#if movie.year}
                            <p class="text-sm text-gray-400">
                                {movie.year}
                                {#if movie.runtime_minutes}
                                    • {formatDuration(movie.runtime_minutes)}
                                {/if}
                            </p>
                        {/if}
                    </div>
                    <button
                        type="button"
                        class="text-gray-400 hover:text-white transition-colors focus:outline-none focus:ring-2 focus:ring-red-600 rounded p-1"
                        aria-label="Close detail panel"
                        onclick={handleClose}
                    >
                        ✕
                    </button>
                </div>

                <!-- Description -->
                {#if movie.description}
                    <div>
                        <h3 class="text-sm font-semibold text-gray-300 mb-2">
                            Overview
                        </h3>
                        <p class="text-sm text-gray-400 leading-relaxed">
                            {movie.description}
                        </p>
                    </div>
                {/if}

                <!-- Genres -->
                {#if movie.genres && movie.genres.length > 0}
                    <div>
                        <h3 class="text-sm font-semibold text-gray-300 mb-2">
                            Genres
                        </h3>
                        <div class="flex flex-wrap gap-2">
                            {#each movie.genres as genre}
                                <Badge variant="info" size="sm">
                                    {genre}
                                </Badge>
                            {/each}
                        </div>
                    </div>
                {/if}

                <!-- Cast -->
                {#if movie.cast && movie.cast.length > 0}
                    <div>
                        <h3 class="text-sm font-semibold text-gray-300 mb-2">
                            Cast
                        </h3>
                        <div class="flex flex-wrap gap-2">
                            {#each movie.cast.slice(0, 5) as castMember}
                                <span
                                    class="text-xs text-gray-400 bg-gray-800 px-2 py-1 rounded"
                                >
                                    {castMember}
                                </span>
                            {/each}
                        </div>
                    </div>
                {/if}

                <!-- Ratings -->
                <div>
                    <h3 class="text-sm font-semibold text-gray-300 mb-2">
                        Ratings
                    </h3>
                    <div class="flex gap-4 flex-wrap">
                        {#if movie.rating_kinopoisk}
                            <div class="text-sm">
                                <span class="text-gray-400">KP:</span>
                                <span class="text-white font-semibold ml-1">
                                    {movie.rating_kinopoisk.toFixed(1)}
                                </span>
                            </div>
                        {/if}
                        {#if movie.rating_imdb}
                            <div class="text-sm">
                                <span class="text-gray-400">IMDb:</span>
                                <span class="text-white font-semibold ml-1">
                                    {movie.rating_imdb.toFixed(1)}
                                </span>
                            </div>
                        {/if}
                        {#if movie.rating_tmdb}
                            <div class="text-sm">
                                <span class="text-gray-400">TMDB:</span>
                                <span class="text-white font-semibold ml-1">
                                    {movie.rating_tmdb.toFixed(1)}
                                </span>
                            </div>
                        {/if}
                    </div>
                </div>

                <!-- Torrent Info -->
                <div>
                    <h3 class="text-sm font-semibold text-gray-300 mb-2">
                        Torrent Info
                    </h3>
                    <div class="grid grid-cols-3 gap-4 text-sm">
                        <div>
                            <span class="text-gray-400">Seeders:</span>
                            <p class="text-white font-semibold">
                                {movie.torrent_info.seeders}
                            </p>
                        </div>
                        <div>
                            <span class="text-gray-400">Leechers:</span>
                            <p class="text-white font-semibold">
                                {movie.torrent_info.seeders -
                                    movie.torrent_info.seeders}
                            </p>
                        </div>
                        <div>
                            <span class="text-gray-400">Size:</span>
                            <p class="text-white font-semibold">
                                {formatFileSize(movie.torrent_info.size_gb)}
                            </p>
                        </div>
                    </div>
                </div>

                <!-- Series Info -->
                {#if movie.number_of_seasons}
                    <div>
                        <h3 class="text-sm font-semibold text-gray-300 mb-2">
                            Series Info
                        </h3>
                        <p class="text-sm text-gray-400">
                            {movie.number_of_seasons} season{movie.number_of_seasons !==
                            1
                                ? "s"
                                : ""}
                            {#if movie.number_of_episodes}
                                • {movie.number_of_episodes} episode{movie.number_of_episodes !==
                                1
                                    ? "s"
                                    : ""}
                            {/if}
                        </p>
                    </div>
                {/if}

                <!-- Quality Selection -->
                {#if movie.torrent_info?.qualities && movie.torrent_info.qualities.length > 0}
                    <div>
                        <QualitySelector
                            qualities={movie.torrent_info.qualities}
                            onSelect={handleQualitySelect}
                        />
                    </div>
                {/if}
            </div>

            <!-- Footer with Actions -->
            <div
                class="flex gap-3 p-6 border-t border-gray-700 justify-end bg-gray-900 rounded-b-lg"
            >
                <Button variant="secondary" on:click={handleClose}>
                    Close
                </Button>
                <Button variant="ghost" on:click={handleBookmark}>
                    ♡ Bookmark
                </Button>
                <Button variant="primary" on:click={handleDownload}>
                    Download
                </Button>
            </div>
        </div>
    </div>
{/if}

<style>
    :global(.animate-fade-in) {
        animation: fadeIn 0.3s ease-in-out;
    }

    :global(.animate-slide-up) {
        animation: slideUp 0.3s ease-out;
    }

    @keyframes fadeIn {
        from {
            opacity: 0;
        }
        to {
            opacity: 1;
        }
    }

    @keyframes slideUp {
        from {
            transform: translateY(20px);
            opacity: 0;
        }
        to {
            transform: translateY(0);
            opacity: 1;
        }
    }

    @media (prefers-reduced-motion: reduce) {
        :global(.animate-fade-in),
        :global(.animate-slide-up) {
            animation: none;
        }
    }
</style>
