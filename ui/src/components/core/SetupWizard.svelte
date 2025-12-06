<script lang="ts">
    import Modal from "./Modal.svelte";
    import Button from "./Button.svelte";

    interface Props {
        open: boolean;
        onClose: () => void;
        onNavigateToSettings: () => void;
    }

    let { open, onClose, onNavigateToSettings }: Props = $props();

    function handleNavigate() {
        onNavigateToSettings();
        onClose();
    }
</script>

<Modal {open} {onClose} title="Setup Required">
    <div class="space-y-4">
        <p class="text-white/80">
            Your settings are incomplete. Please configure the following:
        </p>

        <ul class="space-y-2 text-white/70 text-sm">
            <li class="flex items-center gap-2">
                <span class="text-yellow-400">⚠️</span>
                <span>TMDB API Key (for movie metadata)</span>
            </li>
            <li class="flex items-center gap-2">
                <span class="text-yellow-400">⚠️</span>
                <span>qBittorrent connection (for downloads)</span>
            </li>
            <li class="flex items-center gap-2">
                <span class="text-yellow-400">⚠️</span>
                <span>Library path (for organizing downloads)</span>
            </li>
        </ul>

        <p class="text-white/60 text-sm">
            You can configure these settings now to get started.
        </p>
    </div>

    <svelte:fragment slot="footer">
        <Button variant="secondary" on:click={onClose}>Skip for now</Button>
        <Button variant="primary" on:click={handleNavigate}>
            Go to Settings
        </Button>
    </svelte:fragment>
</Modal>
