# 🎬 TORRENTFLIX UI - FULL REDESIGN

**Scope**: Complete UI overhaul with lean component architecture  
**Backend**: NO CHANGES (keep all Tauri commands as-is)  
**Goal**: Reduce 29 components to ~12 core components  
**Timeline**: 4 weeks

---

## NEW COMPONENT ARCHITECTURE

### Core Components (12 total)

```
src/components/
├── core/
│   ├── Button.svelte          # All button variants
│   ├── Input.svelte           # All input types
│   ├── Modal.svelte           # All modal types
│   ├── Card.svelte            # Card wrapper
│   ├── Badge.svelte           # Status badges
│   ├── Loading.svelte         # Spinner/skeleton/dots
│   ├── Toast.svelte           # Notifications
│   └── Tabs.svelte            # Tab navigation
├── layout/
│   ├── MainLayout.svelte      # Main app layout
│   ├── Header.svelte          # Top bar
│   └── Sidebar.svelte         # Left navigation
└── pages/
    ├── Discover.svelte        # Search + Grid (Feed)
    ├── Library.svelte         # Browser + Collections
    ├── Downloads.svelte       # Download management
    └── Settings.svelte        # All settings
```

**Total**: 12 components (vs 29 current)

---

## WHAT GETS DELETED

These 17 components are consolidated or removed:

```
❌ TopBar.svelte              → Header.svelte
❌ TabBar.svelte              → Sidebar.svelte
❌ MovieGrid.svelte           → Discover.svelte
❌ MovieCard.svelte           → Discover.svelte (inline)
❌ MovieTile.svelte           → Discover.svelte (inline)
❌ DetailPanel.svelte         → Modal.svelte
❌ DownloadsTab.svelte        → Downloads.svelte
❌ SettingsTab.svelte         → Settings.svelte
❌ LibraryBrowser.svelte      → Library.svelte
❌ MediaItemDetail.svelte     → Modal.svelte
❌ CollectionManager.svelte   → Library.svelte
❌ StorageAnalytics.svelte    → Library.svelte (tab)
❌ ManagementPanel.svelte     → Library.svelte (tab)
❌ JobsDashboard.svelte       → Library.svelte (tab)
❌ CleanupManager.svelte      → Library.svelte (tab)
❌ VersionManager.svelte      → Library.svelte (tab)
❌ SkeletonTile.svelte        → Loading.svelte
❌ LoadingSpinner.svelte      → Loading.svelte
❌ LoadingDots.svelte         → Loading.svelte
❌ SearchBar.svelte           → Header.svelte (inline)
❌ SecureInput.svelte         → Input.svelte
❌ SettingsSection.svelte     → Card.svelte
❌ SettingsNotification.svelte → Toast.svelte
```

---

## NEW STRUCTURE

### App.svelte (50 lines)
```svelte
<script>
  import MainLayout from './components/layout/MainLayout.svelte';
  import { currentPage } = './stores/ui';
</script>

<MainLayout>
  {#if $currentPage === 'discover'}
    <Discover />
  {:else if $currentPage === 'library'}
    <Library />
  {:else if $currentPage === 'downloads'}
    <Downloads />
  {:else if $currentPage === 'settings'}
    <Settings />
  {/if}
</MainLayout>
```

### MainLayout.svelte (80 lines)
```svelte
<script>
  import Header from './Header.svelte';
  import Sidebar from './Sidebar.svelte';
</script>

<div class="app-container">
  <Header />
  <div class="main-content">
    <Sidebar />
    <div class="content">
      <slot />
    </div>
  </div>
</div>

<style>
  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  
  .main-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  
  .content {
    flex: 1;
    overflow-y: auto;
  }
</style>
```

### Header.svelte (60 lines)
```svelte
<script>
  import { searchStore } from '../stores/search';
  import Button from './core/Button.svelte';
  import Input from './core/Input.svelte';
  
  let query = '';
  
  async function handleSearch() {
    if (!query.trim()) return;
    await searchStore.search(query);
  }
</script>

<header class="header">
  <div class="logo">TORRENTFLIX</div>
  <div class="search">
    <Input
      type="text"
      placeholder="Search movies..."
      bind:value={query}
      on:keydown={(e) => e.key === 'Enter' && handleSearch()}
    />
    <Button on:click={handleSearch}>Search</Button>
  </div>
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    gap: 2rem;
    padding: 1rem 2rem;
    background: linear-gradient(to bottom, #1a1a1a, transparent);
    border-bottom: 1px solid #404040;
  }
  
  .logo {
    font-size: 1.5rem;
    font-weight: bold;
    color: #E50914;
  }
  
  .search {
    flex: 1;
    display: flex;
    gap: 0.5rem;
  }
</style>
```

### Sidebar.svelte (80 lines)
```svelte
<script>
  import { currentPage } = '../stores/ui';
  import Button from './core/Button.svelte';
  
  const pages = [
    { id: 'discover', label: '🎬 Discover', icon: '🎬' },
    { id: 'library', label: '📚 Library', icon: '📚' },
    { id: 'downloads', label: '📥 Downloads', icon: '📥' },
    { id: 'settings', label: '⚙️ Settings', icon: '⚙️' },
  ];
</script>

<aside class="sidebar">
  <nav>
    {#each pages as page}
      <Button
        variant={$currentPage === page.id ? 'primary' : 'ghost'}
        on:click={() => currentPage.set(page.id)}
      >
        {page.icon} {page.label}
      </Button>
    {/each}
  </nav>
</aside>

<style>
  .sidebar {
    width: 200px;
    background: #1a1a1a;
    border-right: 1px solid #404040;
    padding: 1rem;
    overflow-y: auto;
  }
  
  nav {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
</style>
```

### Discover.svelte (120 lines)
```svelte
<script>
  import { searchStore } from '../stores/search';
  import Card from './core/Card.svelte';
  import Button from './core/Button.svelte';
  import Modal from './core/Modal.svelte';
  import Loading from './core/Loading.svelte';
  
  let selectedMovie = null;
  let showDetail = false;
  
  async function handleDownload(movie) {
    try {
      const hash = await invoke('start_download', {
        magnetLink: movie.torrent_info.magnet_link,
        title: movie.title,
      });
      // Show success toast
    } catch (e) {
      // Show error toast
    }
  }
</script>

<div class="discover">
  {#if $searchStore.loading}
    <Loading type="spinner" />
  {:else if $searchStore.results.length === 0}
    <div class="empty">No movies found</div>
  {:else}
    <div class="grid">
      {#each $searchStore.results as movie (movie.id)}
        <Card
          hoverable
          on:click={() => {
            selectedMovie = movie;
            showDetail = true;
          }}
        >
          <div class="movie-card">
            {#if movie.poster_url}
              <img src={movie.poster_url} alt={movie.title} />
            {:else}
              <div class="no-poster">🎬</div>
            {/if}
            <div class="info">
              <h3>{movie.title}</h3>
              <p>{movie.year}</p>
              <Button on:click|stopPropagation={() => handleDownload(movie)}>
                Download
              </Button>
            </div>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>

<Modal open={showDetail} on:close={() => showDetail = false}>
  {#if selectedMovie}
    <div class="detail">
      <h2>{selectedMovie.title}</h2>
      <p>{selectedMovie.description}</p>
      <Button on:click={() => handleDownload(selectedMovie)}>
        Download
      </Button>
    </div>
  {/if}
</Modal>

<style>
  .discover {
    padding: 2rem;
  }
  
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 1.5rem;
  }
  
  .movie-card {
    display: flex;
    flex-direction: column;
  }
  
  img {
    width: 100%;
    aspect-ratio: 2/3;
    object-fit: cover;
    border-radius: 0.5rem;
  }
  
  .no-poster {
    width: 100%;
    aspect-ratio: 2/3;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 3rem;
    background: #2a2a2a;
    border-radius: 0.5rem;
  }
  
  .info {
    padding: 1rem 0;
  }
  
  h3 {
    margin: 0;
    font-size: 1rem;
  }
  
  p {
    margin: 0.5rem 0;
    color: #999;
    font-size: 0.875rem;
  }
</style>
```

### Library.svelte (150 lines)
```svelte
<script>
  import { libraryStore } from '../stores/library';
  import Tabs from './core/Tabs.svelte';
  import Card from './core/Card.svelte';
  import Button from './core/Button.svelte';
  import Input from './core/Input.svelte';
  import Loading from './core/Loading.svelte';
  
  let activeTab = 'browser';
  let searchQuery = '';
  
  async function handleSearch() {
    if (!searchQuery.trim()) return;
    await libraryStore.search(searchQuery);
  }
</script>

<div class="library">
  <Tabs
    tabs={[
      { id: 'browser', label: 'Browser' },
      { id: 'collections', label: 'Collections' },
      { id: 'analytics', label: 'Analytics' },
      { id: 'cleanup', label: 'Cleanup' },
      { id: 'jobs', label: 'Jobs' },
    ]}
    bind:activeTab
  >
    {#if activeTab === 'browser'}
      <div class="tab-content">
        <div class="search-bar">
          <Input
            type="text"
            placeholder="Search library..."
            bind:value={searchQuery}
          />
          <Button on:click={handleSearch}>Search</Button>
        </div>
        
        {#if $libraryStore.loading}
          <Loading type="skeleton" />
        {:else if $libraryStore.items.length === 0}
          <div class="empty">No items in library</div>
        {:else}
          <div class="list">
            {#each $libraryStore.items as item (item.id)}
              <Card>
                <div class="item">
                  <h4>{item.title}</h4>
                  <p>{item.year}</p>
                </div>
              </Card>
            {/each}
          </div>
        {/if}
      </div>
    {:else if activeTab === 'collections'}
      <div class="tab-content">
        <!-- Collections UI -->
      </div>
    {:else if activeTab === 'analytics'}
      <div class="tab-content">
        <!-- Analytics UI -->
      </div>
    {:else if activeTab === 'cleanup'}
      <div class="tab-content">
        <!-- Cleanup UI -->
      </div>
    {:else if activeTab === 'jobs'}
      <div class="tab-content">
        <!-- Jobs UI -->
      </div>
    {/if}
  </Tabs>
</div>

<style>
  .library {
    padding: 2rem;
  }
  
  .tab-content {
    padding: 1rem 0;
  }
  
  .search-bar {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }
  
  .list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  
  .item {
    padding: 1rem;
  }
  
  h4 {
    margin: 0;
  }
  
  p {
    margin: 0.5rem 0 0 0;
    color: #999;
    font-size: 0.875rem;
  }
</style>
```

### Downloads.svelte (100 lines)
```svelte
<script>
  import { downloadStore } from '../stores/downloads';
  import Card from './core/Card.svelte';
  import Button from './core/Button.svelte';
  import Loading from './core/Loading.svelte';
  
  onMount(() => {
    downloadStore.startPolling();
  });
  
  onDestroy(() => {
    downloadStore.stopPolling();
  });
</script>

<div class="downloads">
  <h2>Downloads</h2>
  
  {#if $downloadStore.loading}
    <Loading type="spinner" />
  {:else if $downloadStore.active.length === 0}
    <div class="empty">No active downloads</div>
  {:else}
    <div class="list">
      {#each $downloadStore.active as download (download.hash)}
        <Card>
          <div class="download-item">
            <div class="info">
              <h4>{download.title}</h4>
              <div class="progress">
                <div class="bar" style="width: {download.progress * 100}%"></div>
              </div>
              <p>{(download.progress * 100).toFixed(1)}%</p>
            </div>
            <div class="actions">
              <Button on:click={() => downloadStore.pause(download.hash)}>
                Pause
              </Button>
              <Button on:click={() => downloadStore.delete(download.hash)}>
                Delete
              </Button>
            </div>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>

<style>
  .downloads {
    padding: 2rem;
  }
  
  .list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  
  .download-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .info {
    flex: 1;
  }
  
  h4 {
    margin: 0;
  }
  
  .progress {
    height: 4px;
    background: #404040;
    border-radius: 2px;
    margin: 0.5rem 0;
    overflow: hidden;
  }
  
  .bar {
    height: 100%;
    background: #E50914;
    transition: width 0.3s ease;
  }
  
  p {
    margin: 0.5rem 0 0 0;
    color: #999;
    font-size: 0.875rem;
  }
  
  .actions {
    display: flex;
    gap: 0.5rem;
  }
</style>
```

### Settings.svelte (200 lines)
```svelte
<script>
  import { settingsStore } from '../stores/settings';
  import Card from './core/Card.svelte';
  import Button from './core/Button.svelte';
  import Input from './core/Input.svelte';
  import Toast from './core/Toast.svelte';
  
  let showToast = false;
  let toastMessage = '';
  
  async function handleSave() {
    try {
      await settingsStore.save();
      toastMessage = 'Settings saved successfully';
      showToast = true;
    } catch (e) {
      toastMessage = `Error: ${e.message}`;
      showToast = true;
    }
  }
</script>

<div class="settings">
  <h2>Settings</h2>
  
  <div class="sections">
    <!-- API Settings -->
    <Card>
      <h3>API & Metadata</h3>
      <Input
        label="TMDB API Key"
        type="password"
        bind:value={$settingsStore.tmdb_api_key}
      />
      <Input
        label="Kinopoisk Token"
        type="password"
        bind:value={$settingsStore.kinopoisk_api_token}
      />
    </Card>
    
    <!-- qBittorrent Settings -->
    <Card>
      <h3>qBittorrent</h3>
      <Input
        label="Host"
        bind:value={$settingsStore.qbittorrent_host}
      />
      <Input
        label="Port"
        type="number"
        bind:value={$settingsStore.qbittorrent_port}
      />
      <Input
        label="Username"
        bind:value={$settingsStore.qbittorrent_username}
      />
      <Input
        label="Password"
        type="password"
        bind:value={$settingsStore.qbittorrent_password}
      />
      <Button on:click={() => settingsStore.testConnection()}>
        Test Connection
      </Button>
    </Card>
    
    <!-- Filesystem Settings -->
    <Card>
      <h3>Filesystem</h3>
      <Input
        label="Library Path"
        bind:value={$settingsStore.library_path}
      />
      <Input
        label="Download Path"
        bind:value={$settingsStore.download_path}
      />
    </Card>
  </div>
  
  <div class="actions">
    <Button variant="primary" on:click={handleSave}>
      Save Settings
    </Button>
  </div>
</div>

<Toast open={showToast} message={toastMessage} />

<style>
  .settings {
    padding: 2rem;
    max-width: 800px;
  }
  
  .sections {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    margin-bottom: 2rem;
  }
  
  h3 {
    margin-top: 0;
  }
  
  .actions {
    display: flex;
    gap: 1rem;
  }
</style>
```

---

## STORES (State Management)

### src/stores/search.ts
```typescript
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export const searchStore = writable({
  query: '',
  results: [],
  loading: false,
  error: null,
});

export const search = async (query: string) => {
  searchStore.update(s => ({ ...s, loading: true, error: null }));
  try {
    const response = await invoke('search_movies', {
      request: { query, media_type: 'movie', limit: 50 }
    });
    searchStore.update(s => ({
      ...s,
      results: response.results,
      loading: false,
    }));
  } catch (e) {
    searchStore.update(s => ({
      ...s,
      error: e.message,
      loading: false,
    }));
  }
};
```

### src/stores/library.ts
```typescript
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export const libraryStore = writable({
  items: [],
  loading: false,
  error: null,
});

export const search = async (query: string) => {
  libraryStore.update(s => ({ ...s, loading: true }));
  try {
    const items = await invoke('search_library', { query, options: {} });
    libraryStore.update(s => ({
      ...s,
      items,
      loading: false,
    }));
  } catch (e) {
    libraryStore.update(s => ({
      ...s,
      error: e.message,
      loading: false,
    }));
  }
};
```

### src/stores/downloads.ts
```typescript
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export const downloadStore = writable({
  active: [],
  loading: false,
  error: null,
});

let pollInterval: number | null = null;

export const startPolling = () => {
  if (pollInterval) return;
  pollInterval = window.setInterval(loadDownloads, 5000);
};

export const stopPolling = () => {
  if (pollInterval) {
    clearInterval(pollInterval);
    pollInterval = null;
  }
};

const loadDownloads = async () => {
  try {
    const downloads = await invoke('get_active_downloads');
    downloadStore.update(s => ({ ...s, active: downloads }));
  } catch (e) {
    console.error('Failed to load downloads:', e);
  }
};

export const pause = async (hash: string) => {
  await invoke('pause_download', { hash });
  await loadDownloads();
};

export const delete = async (hash: string) => {
  await invoke('delete_download', { hash, deleteFiles: false });
  await loadDownloads();
};
```

### src/stores/settings.ts
```typescript
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export const settingsStore = writable({
  tmdb_api_key: '',
  kinopoisk_api_token: '',
  qbittorrent_host: 'localhost',
  qbittorrent_port: 5555,
  qbittorrent_username: '',
  qbittorrent_password: '',
  library_path: '',
  download_path: '',
});

export const load = async () => {
  const settings = await invoke('get_settings');
  settingsStore.set(settings);
};

export const save = async () => {
  const settings = get(settingsStore);
  await invoke('save_settings', { settings });
};

export const testConnection = async () => {
  const settings = get(settingsStore);
  const url = `http://${settings.qbittorrent_host}:${settings.qbittorrent_port}`;
  await invoke('test_qbittorrent', {
    url,
    username: settings.qbittorrent_username,
    password: settings.qbittorrent_password,
  });
};
```

### src/stores/ui.ts
```typescript
import { writable } from 'svelte/store';

export const currentPage = writable('discover');
export const showModal = writable(false);
export const modalContent = writable(null);
```

---

## MIGRATION PLAN

### Week 1: Core Components
- [ ] Create Button.svelte
- [ ] Create Input.svelte
- [ ] Create Modal.svelte
- [ ] Create Card.svelte
- [ ] Create Badge.svelte
- [ ] Create Loading.svelte
- [ ] Create Toast.svelte
- [ ] Create Tabs.svelte

### Week 2: Layout & Stores
- [ ] Create MainLayout.svelte
- [ ] Create Header.svelte
- [ ] Create Sidebar.svelte
- [ ] Create all stores
- [ ] Update App.svelte

### Week 3: Pages
- [ ] Create Discover.svelte
- [ ] Create Library.svelte
- [ ] Create Downloads.svelte
- [ ] Create Settings.svelte

### Week 4: Polish & Testing
- [ ] Test all functionality
- [ ] Fix bugs
- [ ] Optimize performance
- [ ] Delete old components

---

## BENEFITS

✅ **12 components** instead of 29 (-59%)  
✅ **Cleaner code** - each component has single responsibility  
✅ **Better maintainability** - easier to find and fix bugs  
✅ **Better performance** - less overhead  
✅ **Better UX** - consistent design  
✅ **Better DX** - easier to add features  
✅ **No backend changes** - all Tauri commands work as-is  

