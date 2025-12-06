<script lang="ts">
    import Button from "./Button.svelte";

    interface Props {
        icon?: string;
        title: string;
        description?: string;
        actionLabel?: string;
        actionIcon?: string;
        onAction?: () => void;
        variant?: "default" | "error" | "warning";
    }

    let {
        icon = "📭",
        title,
        description,
        actionLabel,
        actionIcon,
        onAction,
        variant = "default",
    }: Props = $props();

    const variantClasses = {
        default: "text-white/70",
        error: "text-red-400",
        warning: "text-yellow-400",
    };
</script>

<div class="flex items-center justify-center h-full">
    <div class="text-center py-12 px-6">
        <!-- Icon -->
        <div class="text-6xl mb-4">{icon}</div>

        <!-- Title -->
        <h2 class="text-2xl font-bold text-white mb-2">{title}</h2>

        <!-- Description -->
        {#if description}
            <p class={`text-lg mb-6 ${variantClasses[variant]}`}>
                {description}
            </p>
        {/if}

        <!-- Action button -->
        {#if actionLabel && onAction}
            <Button variant="primary" on:click={onAction}>
                {#if actionIcon}{actionIcon}
                {/if}
                {actionLabel}
            </Button>
        {/if}
    </div>
</div>
