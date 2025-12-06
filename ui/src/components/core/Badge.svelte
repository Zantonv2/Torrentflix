<script lang="ts">
  interface $Props {
    variant?: 'default' | 'success' | 'warning' | 'error' | 'info';
    size?: 'sm' | 'md';
    ariaLabel?: string;
    class?: string;
  }

  let {
    variant = 'default',
    size = 'md',
    ariaLabel,
    class: className = '',
  }: $Props = $props();

  const variantClasses = {
    default: 'bg-netflix-gray text-white',
    success: 'bg-green-600 text-white',
    warning: 'bg-yellow-600 text-white',
    error: 'bg-red-600 text-white',
    info: 'bg-blue-600 text-white',
  };

  const sizeClasses = {
    sm: 'px-2 py-1 text-xs',
    md: 'px-3 py-1 text-sm',
  };

  const baseClasses = 'font-netflix font-semibold rounded-full inline-block';
  const computedClass = `${baseClasses} ${variantClasses[variant]} ${sizeClasses[size]} ${className}`;
</script>

<span class={computedClass} aria-label={ariaLabel}>
  <slot />
</span>
