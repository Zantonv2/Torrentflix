<script lang="ts">
  import { onMount } from "svelte";

  interface $Props {
    open?: boolean;
    title?: string;
    onClose?: () => void;
    ariaLabel?: string;
    class?: string;
  }

  let {
    open = false,
    title,
    onClose,
    ariaLabel,
    class: className = "",
  }: $Props = $props();

  let modalElement = $state<HTMLDivElement | undefined>();
  let previousFocus: HTMLElement | null = null;
  let isOpen = $state(open);
  const modalId = `modal-${Math.random().toString(36).substr(2, 9)}`;
  const titleId = `${modalId}-title`;

  // Sync isOpen with open prop
  $effect(() => {
    isOpen = open;
  });

  const handleEscape = (e: KeyboardEvent) => {
    if (e.key === "Escape" && isOpen) {
      handleClose();
    }
  };

  const handleClose = () => {
    isOpen = false;
    onClose?.();
    if (previousFocus) {
      previousFocus.focus();
    }
  };

  const handleBackdropClick = (e: MouseEvent) => {
    if (e.target === e.currentTarget) {
      handleClose();
    }
  };

  const focusTrap = (e: KeyboardEvent) => {
    if (e.key !== "Tab" || !modalElement) return;

    const focusableElements = modalElement.querySelectorAll(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
    );
    const firstElement = focusableElements[0] as HTMLElement;
    const lastElement = focusableElements[
      focusableElements.length - 1
    ] as HTMLElement;

    if (e.shiftKey) {
      if (document.activeElement === firstElement) {
        e.preventDefault();
        lastElement?.focus();
      }
    } else {
      if (document.activeElement === lastElement) {
        e.preventDefault();
        firstElement?.focus();
      }
    }
  };

  onMount(() => {
    if (open) {
      previousFocus = document.activeElement as HTMLElement;
      document.addEventListener("keydown", handleEscape);
      document.addEventListener("keydown", focusTrap);
      const firstFocusable = modalElement?.querySelector(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
      ) as HTMLElement;
      firstFocusable?.focus();
    }

    return () => {
      document.removeEventListener("keydown", handleEscape);
      document.removeEventListener("keydown", focusTrap);
    };
  });

  $effect(() => {
    if (open) {
      previousFocus = document.activeElement as HTMLElement;
      document.addEventListener("keydown", handleEscape);
      document.addEventListener("keydown", focusTrap);
      const firstFocusable = modalElement?.querySelector(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
      ) as HTMLElement;
      firstFocusable?.focus();
    } else {
      document.removeEventListener("keydown", handleEscape);
      document.removeEventListener("keydown", focusTrap);
    }
  });
</script>

{#if isOpen}
  <div
    class="fixed inset-0 bg-black bg-opacity-50 backdrop-blur-sm flex items-center justify-center z-50 animate-fade-in"
    on:click={handleBackdropClick}
    role="presentation"
  >
    <div
      bind:this={modalElement}
      class={`bg-netflix-dark rounded-lg shadow-2xl max-w-md w-full mx-4 animate-slide-up ${className}`}
      role="dialog"
      aria-modal="true"
      aria-labelledby={title ? titleId : undefined}
      aria-label={ariaLabel}
    >
      <div
        class="flex items-center justify-between p-6 border-b border-netflix-gray"
      >
        {#if title}
          <h2 id={titleId} class="text-xl font-netflix font-bold text-white">
            {title}
          </h2>
        {/if}
        <button
          type="button"
          class="text-netflix-light hover:text-white transition-colors focus:outline-none focus:ring-2 focus:ring-netflix-red rounded"
          aria-label="Close modal"
          on:click={handleClose}
        >
          ✕
        </button>
      </div>

      <div class="p-6 max-h-96 overflow-y-auto">
        <slot />
      </div>

      <div class="flex gap-3 p-6 border-t border-netflix-gray justify-end">
        <slot name="footer" />
      </div>
    </div>
  </div>
{/if}
