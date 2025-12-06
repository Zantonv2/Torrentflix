<script lang="ts">
    import {
        notificationStore,
        type Notification,
    } from "../../stores/notificationStore";
    import Button from "./Button.svelte";

    interface Props {
        open: boolean;
        onClose: () => void;
    }

    let { open, onClose }: Props = $props();
    let notifications = $state<Notification[]>([]);
    let unreadCount = $state(0);

    notificationStore.subscribe((state) => {
        notifications = state.notifications;
        unreadCount = state.unreadCount;
    });

    function handleMarkAsRead(id: string) {
        notificationStore.markAsRead(id);
    }

    function handleRemove(id: string) {
        notificationStore.removeNotification(id);
    }

    function handleMarkAllAsRead() {
        notificationStore.markAllAsRead();
    }

    function handleClearAll() {
        notificationStore.clearAll();
    }

    function getTypeIcon(type: string): string {
        switch (type) {
            case "success":
                return "✅";
            case "error":
                return "❌";
            case "warning":
                return "⚠️";
            case "info":
            default:
                return "ℹ️";
        }
    }

    function formatTime(date: Date): string {
        const now = new Date();
        const diff = now.getTime() - date.getTime();
        const minutes = Math.floor(diff / 60000);
        const hours = Math.floor(diff / 3600000);
        const days = Math.floor(diff / 86400000);

        if (minutes < 1) return "just now";
        if (minutes < 60) return `${minutes}m ago`;
        if (hours < 24) return `${hours}h ago`;
        if (days < 7) return `${days}d ago`;

        return date.toLocaleDateString();
    }
</script>

{#if open}
    <button
        type="button"
        class="fixed inset-0 bg-black/50 z-40 cursor-default"
        onclick={onClose}
        aria-label="Close notifications"
    ></button>

    <div
        class="fixed right-0 top-[60px] w-96 max-w-full h-[calc(100vh-60px)] bg-netflix-dark border-l border-white/10 shadow-xl z-50 flex flex-col"
    >
        <!-- Header -->
        <div
            class="border-b border-white/10 p-4 flex items-center justify-between"
        >
            <h2 class="text-lg font-bold text-white">Notifications</h2>
            <button
                type="button"
                onclick={onClose}
                class="text-white/50 hover:text-white transition-colors"
                aria-label="Close"
            >
                ✕
            </button>
        </div>

        <!-- Notifications list -->
        <div class="flex-1 overflow-y-auto">
            {#if notifications.length === 0}
                <div class="flex items-center justify-center h-full">
                    <div class="text-center py-12 px-4">
                        <div class="text-4xl mb-2">🔔</div>
                        <p class="text-white/70">No notifications yet</p>
                    </div>
                </div>
            {:else}
                <div class="space-y-1">
                    {#each notifications as notification (notification.id)}
                        <div
                            class="px-4 py-3 border-b border-white/5 hover:bg-white/5 transition-colors cursor-pointer {!notification.read
                                ? 'bg-white/10'
                                : ''}"
                            role="button"
                            tabindex="0"
                            onclick={() => handleMarkAsRead(notification.id)}
                            onkeydown={(e) => {
                                if (e.key === "Enter" || e.key === " ") {
                                    e.preventDefault();
                                    handleMarkAsRead(notification.id);
                                }
                            }}
                        >
                            <div class="flex items-start gap-3">
                                <!-- Icon -->
                                <span class="text-lg flex-shrink-0 mt-0.5">
                                    {getTypeIcon(notification.type)}
                                </span>

                                <!-- Content -->
                                <div class="flex-1 min-w-0">
                                    <p class="text-white text-sm break-words">
                                        {notification.message}
                                    </p>
                                    <p class="text-white/50 text-xs mt-1">
                                        {formatTime(notification.timestamp)}
                                    </p>
                                </div>

                                <!-- Close button -->
                                <button
                                    type="button"
                                    onclick={(e) => {
                                        e.stopPropagation();
                                        handleRemove(notification.id);
                                    }}
                                    class="text-white/30 hover:text-white/70 transition-colors flex-shrink-0"
                                    aria-label="Remove"
                                >
                                    ✕
                                </button>
                            </div>
                        </div>
                    {/each}
                </div>
            {/if}
        </div>

        <!-- Footer -->
        {#if notifications.length > 0}
            <div class="border-t border-white/10 p-4 space-y-2">
                {#if unreadCount > 0}
                    <Button
                        variant="secondary"
                        size="sm"
                        on:click={handleMarkAllAsRead}
                        class="w-full"
                    >
                        Mark all as read
                    </Button>
                {/if}
                <Button
                    variant="ghost"
                    size="sm"
                    on:click={handleClearAll}
                    class="w-full"
                >
                    Clear all
                </Button>
            </div>
        {/if}
    </div>
{/if}

<style>
    :global(.netflix-dark) {
        background-color: #1a1a1a;
    }
</style>
