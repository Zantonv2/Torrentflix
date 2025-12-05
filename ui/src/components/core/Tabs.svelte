<script lang="ts">
  interface Tab {
    id: string;
    label: string;
  }

  interface $Props {
    tabs: Tab[];
    activeTab?: string;
    onChange?: (tabId: string) => void;
    ariaLabel?: string;
    class?: string;
  }

  let {
    tabs,
    activeTab = tabs[0]?.id,
    onChange,
    ariaLabel,
    class: className = '',
  }: $Props = $props();

  let currentTab = $state(activeTab);

  const handleTabClick = (tabId: string) => {
    currentTab = tabId;
    onChange?.(tabId);
  };

  const handleKeyDown = (e: KeyboardEvent, index: number) => {
    let newIndex = index;

    if (e.key === 'ArrowLeft') {
      e.preventDefault();
      newIndex = index === 0 ? tabs.length - 1 : index - 1;
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      newIndex = index === tabs.length - 1 ? 0 : index + 1;
    } else if (e.key === 'Home') {
      e.preventDefault();
      newIndex = 0;
    } else if (e.key === 'End') {
      e.preventDefault();
      newIndex = tabs.length - 1;
    }

    if (newIndex !== index) {
      handleTabClick(tabs[newIndex].id);
    }
  };
</script>

<div class={`flex border-b border-netflix-gray ${className}`} role="tablist" aria-label={ariaLabel}>
  {#each tabs as tab, index (tab.id)}
    <button
      type="button"
      class={`px-4 py-3 font-netflix font-semibold transition-colors relative focus:outline-none focus:ring-2 focus:ring-netflix-red ${
        currentTab === tab.id
          ? 'text-netflix-red'
          : 'text-netflix-light hover:text-white'
      }`}
      aria-selected={currentTab === tab.id}
      aria-controls={`tabpanel-${tab.id}`}
      role="tab"
      tabindex={currentTab === tab.id ? 0 : -1}
      on:click={() => handleTabClick(tab.id)}
      on:keydown={(e) => handleKeyDown(e, index)}
    >
      {tab.label}
      {#if currentTab === tab.id}
        <div class="absolute bottom-0 left-0 right-0 h-1 bg-netflix-red" aria-hidden="true"></div>
      {/if}
    </button>
  {/each}
</div>

<div class="mt-4">
  <slot {activeTab: currentTab} />
</div>
