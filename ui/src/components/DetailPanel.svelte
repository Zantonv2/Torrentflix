<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { UiSearchResult } from "../types";
    import Button from "./core/Button.svelte";
    import Modal from "./core/Modal.svelte";

    interface Props {
        movie: UiSearchResult;
        isOpen?: boolean;
    }

    const { movie, isOpen = false }: Props = $props();
    const dispatch = createEventDispatcher<{
        close: void;
        download: UiSearchResult;
        bookmark: UiSearchResult;
    }>();

    function handleClose() {
        dispatch("close");
    }

    function handleDownload() {
        dispatch("download", movie);
    }

    function handleBookmark() {
        dispatch("bookmark", movie);
    }
</script>

{#if isOpen}
    <Modal
        title={movie.title}
        ariaLabel={`Details for ${movie.title}`}
        on:close={handleClose}
    >
        <div class="space-y-4">
            {#if movie.poster_path}
                <img
                    src={movie.poster_path}
                    alt={movie.title}
                    class="w-full h-auto rounded"
                />
            {/if}

            <div>
                <h3 class="text-lg font-semibold text-white mb-2">Overview</h3>
                <p class="text-netflix-light text-sm">
                    {movie.description || "No description available"}
                </p>
            </div>

            {#if movie.rating}
                <div>
                    <p class="text-sm text-netflix-light">
                        Rating: <span class="text-netflix-red font-semibold"
                            >{movie.rating}</span
                        >
                    </p>
                </div>
            {/if}

            {#if movie.year}
                <div>
                    <p class="text-sm text-netflix-light">
                        Year: <span class="text-white font-semibold"
                            >{movie.year}</span
                        >
                    </p>
                </div>
            {/if}
        </div>

        <svelte:fragment slot="footer">
            <Button variant="secondary" on:click={handleClose}>Close</Button>
            <Button variant="primary" on:click={handleBookmark}>
                Bookmark
            </Button>
            <Button variant="primary" on:click={handleDownload}>
                Download
            </Button>
        </svelte:fragment>
    </Modal>
{/if}

<style>
    :global(.netflix-light) {
        color: #757575;
    }

    :global(.netflix-red) {
        color: #e50914;
    }
</style>
