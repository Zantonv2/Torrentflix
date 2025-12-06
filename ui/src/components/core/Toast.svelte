<script lang="ts">
  import { onMount } from 'svelte';

  interface $Props {
    type?: 'success' | 'error' | 'info' | 'warning';
    message: string;
    autoDismiss?: boolean;
    duration?: number;
    onDismiss?: () => void;
  }

  let {
    type = 'info',
    message,
    autoDismiss = true,
    duration = type === 'error' ? 5000 : 3000,
    onDismiss,
  }: $Props = $props();

  let visible = $state(true);

  const typeClasses = {
    success: 'bg-green-600',
    error: 'bg-red-600',
    info: 'bg-blue-600',
    warning: 'bg-yellow-600',
  };

  const typeIcons = {
    success: '✓',
    error: '✕',
    info: 'ℹ',
    warning: '⚠',
  };

  const typeAriaLabels = {
    success: 'Success',
    error: 'Error',
    info: 'Information',
    warning: 'Warning',
  };

  const dismiss = () => {
    visible = false;
    onDismiss?.();
  };

  onMount(() => {
    if (autoDismiss) {
      const timer = setTimeout(dismiss, duration);
      return () => clearTimeout(timer);
    }
  });
</script>

{#if visible}
  <div
    class={`fixed bottom-4 right-4 ${typeClasses[type]} text-white px-4 py-3 rounded-lg shadow-lg flex items-center gap-3 animate-slide-up z-50`}
    role="alert"
    aria-live="polite"
    aria-label={`${typeAriaLabels[type]}: ${message}`}
  >
    <span class="text-lg" aria-hidden="true">{typeIcons[type]}</span>
    <span class="font-netflix">{message}</span>
    <button
      type="button"
      class="ml-2 hover:opacity-80 transition-opacity focus:outline-none focus:ring-2 focus:ring-white rounded"
      aria-label="Dismiss notification"
      on:click={dismiss}
    >
      ✕
    </button>
  </div>
{/if}
