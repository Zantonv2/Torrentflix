<script lang="ts">
  interface $Props {
    padding?: 'sm' | 'md' | 'lg';
    hover?: boolean;
    clickable?: boolean;
    ariaLabel?: string;
    ariaDescribedby?: string;
    class?: string;
  }

  let {
    padding = 'md',
    hover = false,
    clickable = false,
    ariaLabel,
    ariaDescribedby,
    class: className = '',
  }: $Props = $props();

  const paddingClasses = {
    sm: 'p-2',
    md: 'p-4',
    lg: 'p-6',
  };

  const baseClasses = 'bg-netflix-dark border border-netflix-gray rounded-lg transition-all duration-200';
  const hoverClasses = hover ? 'hover:shadow-lg hover:scale-105' : '';
  const clickableClasses = clickable ? 'cursor-pointer' : '';
  const computedClass = `${baseClasses} ${paddingClasses[padding]} ${hoverClasses} ${clickableClasses} ${className}`;

  const handleKeyDown = (e: KeyboardEvent) => {
    if (clickable && (e.key === 'Enter' || e.key === ' ')) {
      e.preventDefault();
      const element = e.currentTarget as HTMLElement;
      element.click();
    }
  };
</script>

<div
  class={computedClass}
  on:click
  on:keydown={handleKeyDown}
  role={clickable ? 'button' : undefined}
  tabindex={clickable ? 0 : undefined}
  aria-label={ariaLabel}
  aria-describedby={ariaDescribedby}
>
  <slot />
</div>
