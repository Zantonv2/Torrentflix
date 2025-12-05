<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { UiSearchResult } from "../types";
    import Card from "./core/Card.svelte";
    import Loading from "./core/Loading.svelte";

    interface Props {
        movies: UiSearchResult[];
        loading?: boolean;
    }

    const { movies = [], loading = false }: Props = $props();
    const dispatch = createEventDispatcher<{ movieSelect: UiSearchResult }>();

    function handleMovieClick(movie: UiSearchResult) {
        dispatch("movieSelect", movie);
    }
</script>

<div class="w-full h-full overflow-y-auto p-6">
    {#if loading && movies.length === 0}
        <div class="flex items-center justify-center h-full">
            <Loading type="spinner" />
        </div>
    {:else if movies.length === 0}
        <div class="flex items-center justify-center h-full">
            <div class="text-center">
                <p class="text-netflix-light text-lg">No movies found</p>
            </div>
        </div>
    {:else}
        <div
            class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4"
        >
            {#each movies as movie (movie.id)}
                <button
                    class="group cursor-pointer focus:outline-none focus:ring-2 focus:ring-netflix-red rounded overflow-hidden"
                    on:click={() => handleMovieClick(movie)}
                    aria-label={`View details for ${movie.title}`}
                >
                    <Card hover={true} clickable={true} class="h-full">
                        <div
                            class="aspect-video bg-netflix-gray flex items-center justify-center"
                        >
                            {#if movie.poster_path}
                                <img
                                    src={movie.poster_path}
                                    alt={movie.title}
                                    class="w-full h-full object-cover"
                                />
                            {:else}
                                <div class="text-netflix-light text-center p-2">
                                    <p class="text-sm font-semibold">
                                        {movie.title}
                                    </p>
                                </div>
                            {/if}
                        </div>
                        <div class="p-3">
                            <p
                                class="text-white text-sm font-semibold truncate"
                            >
                                {movie.title}
                            </p>
                            {#if movie.rating}
                                <p class="text-netflix-light text-xs mt-1">
                                    ⭐ {movie.rating}
                                </p>
                            {/if}
                        </div>
                    </Card>
                </button>
            {/each}
        </div>
    {/if}
</div>

<style>
    :global(.netflix-light) {
        color: #757575;
    }

    :global(.netflix-gray) {
        background-color: #333333;
    }

    :global(.netflix-red) {
        color: #e50914;
    }
</style>
