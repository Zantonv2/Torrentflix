# 🔥 TorrentFlix - Brutally Honest Action Plan

*Stop planning. Start shipping.*

---

## 🎯 Reality Check

**What you have**: A search engine with fancy rating aggregation
**What you need**: A torrent downloader
**Current state**: 45% complete (not 70%)
**Time wasted on docs**: Too much
**Time spent coding**: Not enough

---

## ✅ COMPLETED: Download Management (Week 1)

**Status**: FULLY WORKING - qBittorrent client implemented and tested

**What's done**:
- ✅ qBittorrent WebAPI client (`engine/src/torrent/client.rs`)
- ✅ Download commands (add, pause, resume, delete, get_all)
- ✅ Downloads UI tab (`ui/src/components/DownloadsTab.svelte`)
- ✅ Real-time progress updates
- ✅ Tauri command integration

**No changes needed** - move to next priority.

---

## ALREADY DONE!!! ## 🚨 PRIORITY 1: Improve Monna Indexer Metadata Parsing (DO THIS NOW)

**File**: `engine/src/indexers/monna.rs`

**Problem**: Current regex patterns fail on field label variations. See `docs/Selectors.md` for examples.

**Current regex patterns (lines 15-48)**:
```rust
pub struct QBittorrentClient {
    base_url: String,
    client: reqwest::Client,
    cookie: Arc<Mutex<Option<String>>>,
}

impl QBittorrentClient {
    pub async fn login(&self, username: &str, password: &str) -> Result<()> {
        let response = self.client
            .post(&format!("{}/api/v2/auth/login", self.base_url))
            .form(&[("username", username), ("password", password)])
            .send()
            .await?;
        
        if response.status().is_success() {
            if let Some(cookie) = response.headers().get("set-cookie") {
                *self.cookie.lock().unwrap() = Some(cookie.to_str()?.to_string());
            }
            Ok(())
        } else {
            Err(anyhow!("Login failed"))
        }
    }
    
    pub async fn add_torrent(&self, magnet_url: &str) -> Result<String> {
        let cookie = self.cookie.lock().unwrap().clone()
            .ok_or_else(|| anyhow!("Not logged in"))?;
        
        let response = self.client
            .post(&format!("{}/api/v2/torrents/add", self.base_url))
            .header("Cookie", cookie)
            .form(&[("urls", magnet_url)])
            .send()
            .await?;
        
        if response.status().is_success() {
            // Extract hash from magnet link
            let hash = extract_hash_from_magnet(magnet_url)?;
            Ok(hash)
        } else {
            Err(anyhow!("Failed to add torrent"))
        }
    }
    
    pub async fn get_torrent_info(&self, hash: &str) -> Result<TorrentInfo> {
        let cookie = self.cookie.lock().unwrap().clone()
            .ok_or_else(|| anyhow!("Not logged in"))?;
        
        let response = self.client
            .get(&format!("{}/api/v2/torrents/info", self.base_url))
            .header("Cookie", cookie)
            .query(&[("hashes", hash)])
            .send()
            .await?;
        
        let torrents: Vec<TorrentInfo> = response.json().await?;
        torrents.into_iter().next()
            .ok_or_else(|| anyhow!("Torrent not found"))
    }
}

#[derive(Debug, Deserialize)]
pub struct TorrentInfo {
    pub hash: String,
    pub name: String,
    pub progress: f32,
    pub dlspeed: u64,
    pub eta: i64,
    pub num_seeds: i32,
    pub num_leechs: i32,
    pub state: String,
}

fn extract_hash_from_magnet(magnet: &str) -> Result<String> {
    // Parse magnet link and extract btih
    let hash = magnet
        .split("btih:")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .ok_or_else(|| anyhow!("Invalid magnet link"))?;
    Ok(hash.to_lowercase())
}
```

**Test it**:
```bash
# Start qBittorrent with Web UI enabled
# Default: http://localhost:8080
# Username: admin, Password: adminadmin

cargo test --package engine -- torrent::client --nocapture
```

---

### Day 3: Download Queue

**File**: `engine/src/torrent/queue.rs`

**Stop pretending, start implementing**:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::SqlitePool;

pub struct DownloadQueue {
    client: Arc<QBittorrentClient>,
    db: Arc<SqlitePool>,
    active: Arc<RwLock<HashMap<String, DownloadStatus>>>,
}

#[derive(Debug, Clone)]
pub struct DownloadStatus {
    pub hash: String,
    pub title: String,
    pub progress: f32,
    pub speed: u64,
    pub eta: i64,
    pub seeders: i32,
    pub leechers: i32,
    pub state: String,
}

impl DownloadQueue {
    pub async fn add_download(&self, magnet: &str, title: &str) -> Result<String> {
        // Add to qBittorrent
        let hash = self.client.add_torrent(magnet).await?;
        
        // Save to database
        sqlx::query!(
            "INSERT INTO downloads (hash, title, magnet, status, created_at) 
             VALUES (?, ?, ?, 'downloading', ?)",
            hash, title, magnet, chrono::Utc::now().timestamp()
        )
        .execute(&*self.db)
        .await?;
        
        Ok(hash)
    }
    
    pub async fn update_progress(&self) -> Result<()> {
        // Get all active downloads from qBittorrent
        let torrents = self.client.get_all_torrents().await?;
        
        let mut active = self.active.write().await;
        for torrent in torrents {
            let status = DownloadStatus {
                hash: torrent.hash.clone(),
                title: torrent.name,
                progress: torrent.progress,
                speed: torrent.dlspeed,
                eta: torrent.eta,
                seeders: torrent.num_seeds,
                leechers: torrent.num_leechs,
                state: torrent.state,
            };
            active.insert(torrent.hash, status);
        }
        
        Ok(())
    }
    
    pub async fn get_active_downloads(&self) -> Vec<DownloadStatus> {
        self.active.read().await.values().cloned().collect()
    }
}
```

---

### Day 4: Downloads UI

**File**: `ui/src/components/DownloadsTab.svelte`

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  
  interface Download {
    hash: string;
    title: string;
    progress: number;
    speed: number;
    eta: number;
    seeders: number;
    leechers: number;
    state: string;
  }
  
  let downloads: Download[] = [];
  
  async function loadDownloads() {
    const response = await invoke('get_active_downloads');
    downloads = response as Download[];
  }
  
  async function pauseDownload(hash: string) {
    await invoke('pause_download', { hash });
    await loadDownloads();
  }
  
  async function resumeDownload(hash: string) {
    await invoke('resume_download', { hash });
    await loadDownloads();
  }
  
  async function cancelDownload(hash: string) {
    if (confirm('Delete this download?')) {
      await invoke('delete_download', { hash });
      await loadDownloads();
    }
  }
  
  function formatSpeed(bytesPerSec: number): string {
    const mbps = bytesPerSec / (1024 * 1024);
    return `${mbps.toFixed(1)} MB/s`;
  }
  
  function formatEta(seconds: number): string {
    if (seconds < 0) return '∞';
    const hours = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${mins}m`;
  }
  
  onMount(async () => {
    await loadDownloads();
    
    // Listen for progress updates
    const unlisten = await listen('download-progress', () => {
      loadDownloads();
    });
    
    // Poll every 2 seconds
    const interval = setInterval(loadDownloads, 2000);
    
    return () => {
      unlisten();
      clearInterval(interval);
    };
  });
</script>

<div class="p-6">
  <h2 class="text-2xl font-bold mb-6">Downloads</h2>
  
  {#if downloads.length === 0}
    <div class="text-center py-12 text-gray-400">
      No active downloads
    </div>
  {:else}
    <div class="space-y-4">
      {#each downloads as download}
        <div class="bg-netflix-gray rounded-lg p-4">
          <div class="flex justify-between items-start mb-2">
            <h3 class="font-semibold">{download.title}</h3>
            <div class="flex gap-2">
              {#if download.state === 'pausedDL'}
                <button on:click={() => resumeDownload(download.hash)} 
                        class="px-3 py-1 bg-green-600 rounded hover:bg-green-700">
                  Resume
                </button>
              {:else}
                <button on:click={() => pauseDownload(download.hash)}
                        class="px-3 py-1 bg-yellow-600 rounded hover:bg-yellow-700">
                  Pause
                </button>
              {/if}
              <button on:click={() => cancelDownload(download.hash)}
                      class="px-3 py-1 bg-red-600 rounded hover:bg-red-700">
                Cancel
              </button>
            </div>
          </div>
          
          <div class="mb-2">
            <div class="flex justify-between text-sm mb-1">
              <span>{(download.progress * 100).toFixed(1)}%</span>
              <span>{formatSpeed(download.speed)} • ETA: {formatEta(download.eta)}</span>
            </div>
            <div class="w-full bg-gray-700 rounded-full h-2">
              <div class="bg-netflix-red h-2 rounded-full transition-all"
                   style="width: {download.progress * 100}%"></div>
            </div>
          </div>
          
          <div class="flex gap-4 text-sm text-gray-400">
            <span>↑ {download.seeders} seeders</span>
            <span>↓ {download.leechers} leechers</span>
            <span class="capitalize">{download.state}</span>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
```

---

## 🚨 Week 2: Library Management

### Day 1-2: File Scanner

**File**: `engine/src/library/scanner.rs`

```rust
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use sqlx::SqlitePool;

pub struct LibraryScanner {
    db: Arc<SqlitePool>,
    library_paths: Vec<PathBuf>,
}

impl LibraryScanner {
    pub async fn scan(&self) -> Result<ScanResult> {
        let mut found = 0;
        let mut added = 0;
        
        for path in &self.library_paths {
            for entry in WalkDir::new(path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if is_video_file(&entry.path()) {
                    found += 1;
                    
                    if self.add_to_library(&entry.path()).await? {
                        added += 1;
                    }
                }
            }
        }
        
        Ok(ScanResult { found, added })
    }
    
    async fn add_to_library(&self, path: &Path) -> Result<bool> {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid filename"))?;
        
        // Check if already in library
        let exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM library WHERE file_path = ?",
            path.to_str()
        )
        .fetch_one(&*self.db)
        .await? > 0;
        
        if exists {
            return Ok(false);
        }
        
        // Parse filename
        let normalizer = Normalizer::default();
        let parsed = normalizer.normalize_filename(file_name)?;
        
        // Insert into library
        sqlx::query!(
            "INSERT INTO library (file_path, title, year, size_bytes, added_at)
             VALUES (?, ?, ?, ?, ?)",
            path.to_str(),
            parsed.title,
            parsed.year,
            std::fs::metadata(path)?.len() as i64,
            chrono::Utc::now().timestamp()
        )
        .execute(&*self.db)
        .await?;
        
        Ok(true)
    }
}

fn is_video_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("mkv" | "mp4" | "avi" | "mov" | "wmv" | "flv" | "webm")
    )
}
```

---

### Day 3-4: Library UI

**File**: `ui/src/components/LibraryTab.svelte`

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import MovieGrid from './MovieGrid.svelte';
  
  let library = [];
  let filter = '';
  let sortBy = 'added_at';
  
  async function loadLibrary() {
    const response = await invoke('get_library', { filter, sortBy });
    library = response;
  }
  
  async function scanLibrary() {
    await invoke('scan_library');
    await loadLibrary();
  }
  
  onMount(loadLibrary);
</script>

<div class="p-6">
  <div class="flex justify-between items-center mb-6">
    <h2 class="text-2xl font-bold">Library</h2>
    <button on:click={scanLibrary} 
            class="px-4 py-2 bg-netflix-red rounded hover:bg-red-700">
      Scan Library
    </button>
  </div>
  
  <div class="mb-6 flex gap-4">
    <input type="text" 
           bind:value={filter}
           on:input={loadLibrary}
           placeholder="Search library..."
           class="flex-1 px-4 py-2 bg-netflix-gray rounded" />
    
    <select bind:value={sortBy} on:change={loadLibrary}
            class="px-4 py-2 bg-netflix-gray rounded">
      <option value="added_at">Recently Added</option>
      <option value="title">Title</option>
      <option value="year">Year</option>
      <option value="rating">Rating</option>
    </select>
  </div>
  
  <MovieGrid movies={library} />
</div>
```

---

## 🚨 Week 3: Settings UI

**File**: `ui/src/components/SettingsTab.svelte`

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  
  let settings = {
    tmdb_api_key: '',
    kinopoisk_api_key: '',
    qbittorrent_url: 'http://localhost:8080',
    qbittorrent_username: 'admin',
    qbittorrent_password: '',
    library_paths: [],
    download_path: '',
  };
  
  async function loadSettings() {
    settings = await invoke('get_settings');
  }
  
  async function saveSettings() {
    await invoke('save_settings', { settings });
    alert('Settings saved!');
  }
  
  async function testQBittorrent() {
    try {
      await invoke('test_qbittorrent', { 
        url: settings.qbittorrent_url,
        username: settings.qbittorrent_username,
        password: settings.qbittorrent_password
      });
      alert('Connection successful!');
    } catch (e) {
      alert(`Connection failed: ${e}`);
    }
  }
  
  onMount(loadSettings);
</script>

<div class="p-6 max-w-2xl">
  <h2 class="text-2xl font-bold mb-6">Settings</h2>
  
  <div class="space-y-6">
    <section>
      <h3 class="text-xl font-semibold mb-4">API Keys</h3>
      <div class="space-y-4">
        <div>
          <label class="block mb-2">TMDB API Key</label>
          <input type="password" bind:value={settings.tmdb_api_key}
                 class="w-full px-4 py-2 bg-netflix-gray rounded" />
        </div>
        <div>
          <label class="block mb-2">Kinopoisk API Key (Optional)</label>
          <input type="password" bind:value={settings.kinopoisk_api_key}
                 class="w-full px-4 py-2 bg-netflix-gray rounded" />
        </div>
      </div>
    </section>
    
    <section>
      <h3 class="text-xl font-semibold mb-4">qBittorrent</h3>
      <div class="space-y-4">
        <div>
          <label class="block mb-2">Web UI URL</label>
          <input type="text" bind:value={settings.qbittorrent_url}
                 class="w-full px-4 py-2 bg-netflix-gray rounded" />
        </div>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block mb-2">Username</label>
            <input type="text" bind:value={settings.qbittorrent_username}
                   class="w-full px-4 py-2 bg-netflix-gray rounded" />
          </div>
          <div>
            <label class="block mb-2">Password</label>
            <input type="password" bind:value={settings.qbittorrent_password}
                   class="w-full px-4 py-2 bg-netflix-gray rounded" />
          </div>
        </div>
        <button on:click={testQBittorrent}
                class="px-4 py-2 bg-blue-600 rounded hover:bg-blue-700">
          Test Connection
        </button>
      </div>
    </section>
    
    <div class="pt-6 border-t border-gray-700">
      <button on:click={saveSettings}
              class="px-6 py-3 bg-netflix-red rounded hover:bg-red-700 font-semibold">
        Save Settings
      </button>
    </div>
  </div>
</div>
```

---

## 🎯 Success Criteria (No Bullshit)

### Week 1 Done When:
- [ ] Can add magnet link to qBittorrent via UI
- [ ] Can see download progress in real-time
- [ ] Can pause/resume/cancel downloads
- [ ] Downloads persist in database

### Week 2 Done When:
- [ ] Can scan library folder
- [ ] Downloaded files appear in library automatically
- [ ] Can browse/search library
- [ ] Can mark movies as watched

### Week 3 Done When:
- [ ] Can configure API keys via UI
- [ ] Can configure qBittorrent connection
- [ ] Settings persist across restarts
- [ ] Test buttons actually work

---

## 🚫 What NOT to Do

- ❌ Don't write more documentation
- ❌ Don't refactor existing code
- ❌ Don't add new features
- ❌ Don't optimize performance
- ❌ Don't write tests (yet)

**Just ship the fucking features.**

---

## 📊 Actual Progress Tracking

Update this daily:

**Week 1**:
- [ ] Day 1: qBittorrent login + add_torrent
- [ ] Day 2: get_torrent_info + pause/resume/delete
- [ ] Day 3: Download queue + database persistence
- [ ] Day 4: Downloads UI + real-time updates

**Week 2**:
- [ ] Day 1: File scanner implementation
- [ ] Day 2: Library database operations
- [ ] Day 3: Library UI grid view
- [ ] Day 4: Library filters + search

**Week 3**:
- [ ] Day 1: Settings data model + persistence
- [ ] Day 2: Settings UI layout
- [ ] Day 3: API key management + test buttons
- [ ] Day 4: Polish + bug fixes

---

## 🏁 Ship It

After 3 weeks, you'll have:
- ✅ Working download management
- ✅ Working library management
- ✅ Working settings UI
- ✅ An app people can actually use

Then you can:
1. Get user feedback
2. Fix bugs
3. Add tests
4. Add more features

**But first: SHIP THE CORE FEATURES.**

Stop planning. Start coding. Ship it. 🚀
