<script lang="ts">
    import { onMount } from "svelte";

    export interface MenuItem {
        id: string;
        label: string;
        icon?: string;
        action: () => void;
        disabled?: boolean;
    }

    interface Props {
        items: MenuItem[];
        x: number;
        y: number;
        onClose: () => void;
    }

    let { items, x, y, onClose }: Props = $props();
    let menuElement: HTMLDivElement | null = null;
    let selectedIndex = $state(0);

    function handleItemClick(item: MenuItem) {
        if (!item.disabled) {
            item.action();
            onClose();
        }
    }

    function handleKeyDown(event: KeyboardEvent) {
        if (event.key === "Escape") {
            onClose();
        } else if (event.key === "ArrowDown") {
            event.preventDefault();
            selectedIndex = (selectedIndex + 1) % items.length;
        } else if (event.key === "ArrowUp") {
            event.preventDefault();
            selectedIndex = (selectedIndex - 1 + items.length) % items.length;
        } else if (event.key === "Enter") {
            event.preventDefault();
            handleItemClick(items[selectedIndex]);
        }
    }

    function handleClickOutside(event: MouseEvent) {
        if (menuElement && !menuElement.contains(event.target as Node)) {
            onClose();
        }
    }

    onMount(() => {
        // Focus the menu for keyboard navigation
        menuElement?.focus();

        // Add click outside listener
        document.addEventListener("click", handleClickOutside);
        document.addEventListener("keydown", handleKeyDown);

        return () => {
            document.removeEventListener("click", handleClickOutside);
            document.removeEventListener("keydown", handleKeyDown);
        };
    });
</script>

<div
    bind:this={menuElement}
    class="fixed bg-netflix-dark border border-white/10 rounded-lg shadow-xl z-50 min-w-48 py-1"
    style="left: {x}px; top: {y}px;"
    role="menu"
    tabindex="0"
>
    {#each items as item, index (item.id)}
        <button
            type="button"
            onclick={() => handleItemClick(item)}
            class="w-full text-left px-4 py-2 text-white/80 hover:bg-white/10 hover:text-white transition-colors flex items-center gap-2 {selectedIndex ===
            index
                ? 'bg-white/10 text-white'
                : ''} {item.disabled
                ? 'opacity-50 cursor-not-allowed'
                : 'cursor-pointer'}"
            role="menuitem"
            aria-disabled={item.disabled}
            disabled={item.disabled}
        >
            {#if item.icon}
                <span class="text-lg">{item.icon}</span>
            {/if}
            <span>{item.label}</span>
        </button>
    {/each}
</div>

<style>
    :global(.netflix-dark) {
        background-color: #1a1a1a;
    }
</style>
