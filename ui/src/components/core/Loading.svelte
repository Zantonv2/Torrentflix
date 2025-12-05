<script lang="ts">
  interface $Props {
    type?: "spinner" | "skeleton" | "dots";
    ariaLabel?: string;
    class?: string;
  }

  let {
    type = "spinner",
    ariaLabel = "Loading",
    class: className = "",
  }: $Props = $props();
</script>

{#if type === "spinner"}
  <div
    class={`flex justify-center items-center ${className}`}
    aria-busy="true"
    aria-label={ariaLabel}
  >
    <div
      class="w-8 h-8 border-4 border-netflix-gray border-t-netflix-red rounded-full animate-spin"
      aria-hidden="true"
    ></div>
  </div>
{:else if type === "skeleton"}
  <div class={`space-y-4 ${className}`} aria-busy="true" aria-label={ariaLabel}>
    <div
      class="h-4 bg-netflix-gray rounded animate-pulse"
      aria-hidden="true"
    ></div>
    <div
      class="h-4 bg-netflix-gray rounded animate-pulse w-5/6"
      aria-hidden="true"
    ></div>
    <div
      class="h-4 bg-netflix-gray rounded animate-pulse w-4/6"
      aria-hidden="true"
    ></div>
  </div>
{:else if type === "dots"}
  <div
    class={`flex justify-center items-center gap-2 ${className}`}
    aria-busy="true"
    aria-label={ariaLabel}
  >
    <div
      class="w-2 h-2 bg-netflix-red rounded-full animate-bounce"
      style="animation-delay: 0s"
      aria-hidden="true"
    ></div>
    <div
      class="w-2 h-2 bg-netflix-red rounded-full animate-bounce"
      style="animation-delay: 0.2s"
      aria-hidden="true"
    ></div>
    <div
      class="w-2 h-2 bg-netflix-red rounded-full animate-bounce"
      style="animation-delay: 0.4s"
      aria-hidden="true"
    ></div>
  </div>
{/if}
