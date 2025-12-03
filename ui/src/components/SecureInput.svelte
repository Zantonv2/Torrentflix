<script lang="ts">
  import { maskSensitiveValue } from '../utils/masking';

  export let id: string;
  export let label: string;
  export let value: string | undefined = '';
  export let placeholder: string = '';
  export let showMasked: boolean = false;
  export let error: string | undefined = undefined;

  let showPassword = false;
  let copyFeedback = false;

  function togglePasswordVisibility() {
    showPassword = !showPassword;
  }

  function handleChange(e: Event) {
    const target = e.target as HTMLInputElement;
    value = target.value || undefined;
  }

  async function copyToClipboard() {
    if (!value) return;
    
    try {
      await navigator.clipboard.writeText(value);
      copyFeedback = true;
      setTimeout(() => {
        copyFeedback = false;
      }, 2000);
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
    }
  }
</script>

<div>
  <div class="flex items-center justify-between mb-2">
    <label for={id} class="block text-sm font-medium text-gray-300">{label}</label>
    <div class="flex gap-2">
      {#if value && showMasked}
        <button
          type="button"
          on:click={togglePasswordVisibility}
          class="text-xs text-gray-400 hover:text-gray-300 transition-colors"
          title={showPassword ? 'Hide' : 'Show'}
        >
          {showPassword ? '👁️ Hide' : '👁️ Show'}
        </button>
      {/if}
      {#if value}
        <button
          type="button"
          on:click={copyToClipboard}
          class="text-xs text-gray-400 hover:text-gray-300 transition-colors"
          title="Copy to clipboard"
        >
          {copyFeedback ? '✓ Copied' : '📋 Copy'}
        </button>
      {/if}
    </div>
  </div>

  <input
    {id}
    type={showPassword ? 'text' : 'password'}
    {placeholder}
    value={value || ''}
    on:change={handleChange}
    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-500 focus:outline-none focus:border-netflix-red"
  />

  {#if value && showMasked}
    <p class="text-xs text-gray-500 mt-1">Saved: {maskSensitiveValue(value)}</p>
  {/if}

  {#if error}
    <p class="text-xs text-red-400 mt-1">{error}</p>
  {/if}
</div>

<style>
  /* Prevent password managers from auto-filling and exposing sensitive data */
  input[type='password'] {
    -webkit-text-security: disc;
  }
</style>
