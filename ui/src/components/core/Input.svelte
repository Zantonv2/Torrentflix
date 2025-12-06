<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  interface $Props extends HTMLInputAttributes {
    label?: string;
    error?: string;
    masked?: boolean;
    ariaLabel?: string;
    ariaDescribedby?: string;
  }

  let {
    label,
    error,
    masked = false,
    type = "text",
    placeholder = "",
    value = $bindable(""),
    ariaLabel,
    ariaDescribedby,
    class: className = "",
    ...rest
  }: $Props = $props();

  let showPassword = $state(false);
  let inputType = $state(type);
  let inputElement: HTMLInputElement | undefined = $state();
  const inputId = `input-${Math.random().toString(36).substr(2, 9)}`;
  const errorId = `${inputId}-error`;
  const labelId = `${inputId}-label`;

  $effect(() => {
    if (type === "password" && masked) {
      inputType = showPassword ? "text" : "password";
    } else {
      inputType = type;
    }
  });

  const togglePasswordVisibility = () => {
    if (type === "password" && masked) {
      showPassword = !showPassword;
    }
  };

  const clearInput = () => {
    value = "";
    inputElement?.focus();
  };

  const describedBy =
    [error ? errorId : null, ariaDescribedby].filter(Boolean).join(" ") ||
    undefined;
</script>

<div class="flex flex-col gap-2">
  {#if label}
    <label
      for={inputId}
      id={labelId}
      class="text-sm font-netflix font-semibold text-white"
    >
      {label}
    </label>
  {/if}

  <div class="relative">
    <input
      bind:this={inputElement}
      id={inputId}
      type={inputType}
      {placeholder}
      bind:value
      aria-label={ariaLabel || label}
      aria-describedby={describedBy}
      aria-invalid={error ? "true" : "false"}
      class={`w-full px-3 py-2 bg-netflix-gray text-white rounded border transition-colors ${
        error ? "border-red-500" : "border-netflix-gray"
      } focus:border-netflix-red focus:outline-none focus:ring-1 focus:ring-netflix-red ${className}`}
      {...rest}
      on:input
      on:change
      on:focus
      on:blur
    />

    <div class="absolute right-3 top-1/2 -translate-y-1/2 flex gap-1">
      {#if value && (type === "text" || type === "email" || type === "search" || type === "number")}
        <button
          type="button"
          class="text-netflix-light hover:text-white transition-colors p-1"
          aria-label="Clear input"
          on:click={clearInput}
        >
          ✕
        </button>
      {/if}

      {#if type === "password" && masked}
        <button
          type="button"
          class="text-netflix-light hover:text-white transition-colors p-1"
          aria-label={showPassword ? "Hide password" : "Show password"}
          on:click={togglePasswordVisibility}
        >
          {showPassword ? "👁️" : "👁️‍🗨️"}
        </button>
      {/if}
    </div>
  </div>

  {#if error}
    <span id={errorId} class="text-xs text-red-500 font-netflix" role="alert">
      {error}
    </span>
  {/if}
</div>

<style>
  :global(input:focus-visible) {
    outline: none;
  }
</style>
