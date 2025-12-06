<script lang="ts">
    import type { UiSearchResult } from "../types";
    import Card from "./core/Card.svelte";
    import ContextMenu from "./core/ContextMenu.svelte";
    import type { MenuItem } from "./core/ContextMenu.svelte";

    interface $Props {
        movie: UiSearchResult;
        onClick?: () => void;
        onContextMenu?: (event: MouseEvent) => void;
        ariaLabel?: string;
    }

    let { movie, onClick, onContextMenu, ariaLabel }: $Props = $props();
    let showContextMenu = $state(false);
    let contextMenuX = $state(0);
    let contextMenuY = $state(0);

    // Format ratings display
    const formatRating = (rating: number | undefined): string => {
        if (rating === undefined || rating === null) return "";
        return rating.toFixed(1);
    };

    // Build ratings string
    const ratingsDisplay = (() => {
        const ratings = [];
        if (movie.rating_kinopoisk !== undefined) {
            ratings.push(`★ KP ${formatRating(movie.rating_kinopoisk)}`);
        }
        if (movie.rating_imdb !== undefined) {
            ratings.push(`IMDB ${formatRating(movie.rating_imdb)}`);
        }
        if (movie.rating_tmdb !== undefined) {
            ratings.push(`TMDB ${formatRating(movie.rating_tmdb)}`);
        }
        return ratings.join(" | ");
    })();

    // Format series info
    const seriesInfo = (() => {
        if (
            movie.number_of_seasons !== undefined &&
            movie.number_of_episodes !== undefined
        ) {
            return `Season ${movie.number_of_seasons} • ${movie.number_of_episodes} episodes`;
        }
        return "";
    })();

    // Determine if this is series content
    const isSeriesContent = movie.number_of_seasons !== undefined;

    const handleKeyDown = (e: KeyboardEvent) => {
        if ((e.key === "Enter" || e.key === " ") && onClick) {
            e.preventDefault();
            onClick();
        }
    };

    const handleContextMenu = (event: MouseEvent) => {
        event.preventDefault();
        contextMenuX = event.clientX;
        contextMenuY = event.clientY;
        showContextMenu = true;
        if (onContextMenu) {
            onContextMenu(event);
        }
    };

    const contextMenuItems: MenuItem[] = [
        {
            id: "view-details",
            label: "View Details",
            icon: "👁️",
            action: () => {
                if (onClick) onClick();
            },
        },
        {
            id: "download",
            label: "Download",
            icon: "📥",
            action: () => {
                // Dispatch download event
                const event = new CustomEvent("download", { detail: movie });
                window.dispatchEvent(event);
            },
        },
        {
            id: "add-collection",
            label: "Add to Collection",
            icon: "📚",
            action: () => {
                // Dispatch add-to-collection event
                const event = new CustomEvent("add-to-collection", {
                    detail: movie,
                });
                window.dispatchEvent(event);
            },
        },
        {
            id: "copy-title",
            label: "Copy Title",
            icon: "📋",
            action: () => {
                navigator.clipboard.writeText(movie.title);
            },
        },
    ];
</script>

<Card
    hover={true}
    clickable={true}
    on:click={onClick}
    on:keydown={handleKeyDown}
    on:contextmenu={handleContextMenu}
    ariaLabel={ariaLabel ||
        `${movie.title}${isSeriesContent ? ` - ${seriesInfo}` : ""}`}
    class="flex flex-col h-full hover-transition"
>
    <!-- Poster Container: 2:3 aspect ratio (portrait) -->
    <div
        class="relative w-full aspect-video overflow-hidden"
        style="aspect-ratio: 2/3;"
    >
        {#if movie.poster_url}
            <!-- Poster with object-fit cover -->
            <img
                src={movie.poster_url}
                alt={`${movie.title} poster`}
                class="w-full h-full object-cover rounded-lg"
                loading="lazy"
            />
        {:else}
            <!-- Placeholder for missing poster -->
            <div
                class="w-full h-full bg-netflix-gray rounded-lg flex items-center justify-center"
            >
                <div class="text-center px-4">
                    <p
                        class="text-netflix-light text-sm font-netflix line-clamp-3"
                    >
                        {movie.title}
                    </p>
                </div>
            </div>
        {/if}

        <!-- Gradient overlay on hover (Property 23.5) -->
        <div
            class="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 hover:opacity-100 transition-opacity duration-150 rounded-lg pointer-events-none"
        ></div>
    </div>

    <!-- Title Section -->
    <div class="mt-3 flex-1 flex flex-col">
        <h3
            class="text-sm font-netflix font-semibold text-white line-clamp-2 leading-tight"
        >
            {movie.title}
        </h3>

        <!-- Ratings Display -->
        {#if ratingsDisplay}
            <p class="text-xs text-netflix-light mt-2 font-netflix">
                {ratingsDisplay}
            </p>
        {/if}

        <!-- Series Info Display -->
        {#if seriesInfo}
            <p class="text-xs text-netflix-light mt-1 font-netflix">
                {seriesInfo}
            </p>
        {/if}
    </div>
</Card>

{#if showContextMenu}
    <ContextMenu
        items={contextMenuItems}
        x={contextMenuX}
        y={contextMenuY}
        onClose={() => (showContextMenu = false)}
    />
{/if}

<style>
    :global(.line-clamp-2) {
        display: -webkit-box;
        -webkit-line-clamp: 2;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }

    :global(.line-clamp-3) {
        display: -webkit-box;
        -webkit-line-clamp: 3;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }
</style>
