<script lang="ts">
  import type { HTMLButtonAttributes } from 'svelte/elements';

  interface $Props extends HTMLButtonAttributes {
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
    size?: 'sm' | 'md' | 'lg';
    loading?: boolean;
    ariaLabel?: string;
    ariaDescribedby?: string;
  }

  let {
    variant = 'primary',
    size = 'md',
    loading = false,
    disabled = false,
    ariaLabel,
    ariaDescribedby,
    class: className = '',
    ...rest
  }: $Props = $props();

  const variantClasses = {
    primary: 'bg-netflix-red hover:bg-red-700 text-white',
    secondary: 'bg-netflix-gray hover:bg-gray-600 text-white',
    ghost: 'bg-transparent hover:bg-netflix-gray text-white border border-netflix-gray',
    danger: 'bg-red-600 hover:bg-red-700 text-white',
  };

  const sizeClasses = {
    sm: 'px-2 py-1 text-sm',
    md: 'px-3 py-2 text-base',
    lg: 'px-4 py-3 text-lg',
  };

  const baseClasses = 'font-netflix font-semibold rounded transition-colors duration-200 disabled:opacity-50 disabled:cursor-not-allowed';
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

<style>
  :global(button:focus-visible) {
    outline: 2px solid #E50914;
    outline-offset: 2px;
  }
</style>
