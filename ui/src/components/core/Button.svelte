<script lang="ts">
  import type { HTMLButtonAttributes } from "svelte/elements";

  interface $Props extends HTMLButtonAttributes {
    variant?: "primary" | "secondary" | "ghost" | "danger";
    size?: "sm" | "md" | "lg";
    loading?: boolean;
    ariaLabel?: string;
    ariaDescribedby?: string;
  }

  let {
    variant = "primary",
    size = "md",
    loading = false,
    disabled = false,
    ariaLabel,
    ariaDescribedby,
    class: className = "",
    ...rest
  }: $Props = $props();

  // Variant classes with proper color contrast (4.5:1 minimum for WCAG AA)
  // Primary: Netflix Red (#E50914) on dark background
  // Secondary: Gray (#4b5563) on dark background
  // Ghost: Transparent with border
  // Danger: Red (#DC2626) on dark background
  const variantClasses = {
    primary: "bg-netflix-red hover:bg-red-700 active:bg-red-800 text-white",
    secondary: "bg-gray-600 hover:bg-gray-700 active:bg-gray-800 text-white",
    ghost:
      "bg-transparent hover:bg-netflix-gray active:bg-gray-700 text-white border border-netflix-gray",
    danger: "bg-red-600 hover:bg-red-700 active:bg-red-800 text-white",
  };

  const sizeClasses = {
    sm: "px-2 py-1 text-sm",
    md: "px-3 py-2 text-base",
    lg: "px-4 py-3 text-lg",
  };

  const baseClasses =
    "font-netflix font-semibold rounded transition-all duration-150 disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-2 focus-visible:outline-netflix-red focus-visible:outline-offset-2";
  const computedClass = `${baseClasses} ${variantClasses[variant]} ${sizeClasses[size]} ${className}`;
</script>

<button
  {disabled}
  class={computedClass}
  aria-label={ariaLabel}
  aria-describedby={ariaDescribedby}
  aria-busy={loading}
  {...rest}
  on:click
>
  {#if loading}
    <span class="inline-block animate-spin" aria-hidden="true">⏳</span>
  {/if}
  <slot />
</button>
