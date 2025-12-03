<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import SettingsSection from './SettingsSection.svelte';
  import SettingsNotification from './SettingsNotification.svelte';
  import SecureInput from './SecureInput.svelte';

  interface Settings {
    // API & Metadata
    tmdb_api_key?: string;
    tmdb_read_access_token?: string;
    kinopoisk_api_token?: string;
    
    // qBittorrent
    qbittorrent_enabled: boolean;
    qbittorrent_host: string;
    qbittorrent_port: number;
    qbittorrent_username: string;
    qbittorrent_password: string;
    qbittorrent_use_ssl: boolean;
    qbittorrent_verify_ssl: boolean;
    qbittorrent_category?: string;
    qbittorrent_tags?: string;
    qbittorrent_timeout: number;
    
    // Filesystem
    library_path: string;
    download_path: string;
    
    // Notifications
    telegram_enabled: boolean;
    telegram_bot_token?: string;
    telegram_chat_id?: string;
    telegram_debounce: number;
    email_enabled: boolean;
    smtp_host: string;
    smtp_port: number;
    smtp_user?: string;
    smtp_password?: string;
    from_email?: string;
    to_email?: string;
    
    // Application
    log_level: string;
    log_format: string;
    debug: boolean;
    search_limit: number;
    search_timeout: number;
    max_concurrent_downloads: number;
    enabled_indexers: string;
    
    // Proxy
    proxy_enabled: boolean;
    proxy_address: string;
    proxy_port: number;
    
    // Database
    database_url: string;
    db_pool_size: number;
    db_max_overflow: number;
  }

  interface ValidationError {
    [key: string]: string;
  }

  let settings: Settings | null = null;
  let loading = true;
  let saving = false;
  let testingConnection = false;
  let errors: ValidationError = {};
  let notification: { message: string; type: 'success' | 'error'; visible: boolean } = {
    message: '',
    type: 'success',
    visible: false
  };

  onMount(async () => {
    await loadSettings();
  });

  async function loadSettings() {
    try {
      loading = true;
      const result = await invoke('get_settings');
      settings = result as Settings;
      errors = {};
    } catch (err) {
      showNotification(`Failed to load settings: ${err}`, 'error');
      console.error('Failed to load settings:', err);
    } finally {
      loading = false;
    }
  }

  async function saveSettings() {
    if (!settings) return;
    
    try {
      saving = true;
      errors = {};
      await invoke('save_settings', { settings });
      showNotification('Settings saved successfully', 'success');
    } catch (err) {
      const errorMsg = err as string;
      // Try to parse validation errors
      if (errorMsg.includes('validation')) {
        errors = parseValidationErrors(errorMsg);
      }
      showNotification(`Failed to save settings: ${errorMsg}`, 'error');
      console.error('Failed to save settings:', err);
    } finally {
      saving = false;
    }
  }

  async function testQBittorrentConnection() {
    if (!settings) return;
    
    try {
      testingConnection = true;
      await invoke('test_qbittorrent', {
        url: `${settings.qbittorrent_use_ssl ? 'https' : 'http'}://${settings.qbittorrent_host}:${settings.qbittorrent_port}`,
        username: settings.qbittorrent_username,
        password: settings.qbittorrent_password
      });
      showNotification('Connection to qBittorrent successful', 'success');
      // Auto-save on successful connection
      await saveSettings();
    } catch (err) {
      showNotification(`Connection failed: ${err}`, 'error');
      console.error('Connection test failed:', err);
    } finally {
      testingConnection = false;
    }
  }

  async function pickFolder(fieldName: 'library_path' | 'download_path') {
    try {
      const path = await invoke('pick_folder');
      if (settings && path) {
        updateSetting(fieldName, path as string);
        // Clear any previous validation errors for this field
        if (errors[fieldName]) {
          delete errors[fieldName];
          errors = errors;
        }
      }
    } catch (err) {
      showNotification(`Failed to open folder picker: ${err}`, 'error');
      console.error('Folder picker error:', err);
    }
  }



  function showNotification(message: string, type: 'success' | 'error') {
    notification = { message, type, visible: true };
    // Auto-dismiss: 3 seconds for success, 5 seconds for errors
    const dismissTime = type === 'success' ? 3000 : 5000;
    setTimeout(() => {
      notification.visible = false;
    }, dismissTime);
  }

  function parseValidationErrors(errorMsg: string): ValidationError {
    const errors: ValidationError = {};
    
    // Parse validation errors from the format: "field_name: error message"
    // The backend returns errors in the format:
    // "Settings validation failed:\nfield1: message1\nfield2: message2"
    
    const lines = errorMsg.split('\n');
    for (const line of lines) {
      // Skip the header line
      if (line.includes('Settings validation failed:')) continue;
      
      // Parse "field_name: error message" format
      const colonIndex = line.indexOf(':');
      if (colonIndex > 0) {
        const field = line.substring(0, colonIndex).trim();
        const message = line.substring(colonIndex + 1).trim();
        errors[field] = message;
      }
    }
    
    return errors;
  }

  function updateSetting<K extends keyof Settings>(key: K, value: Settings[K]) {
    if (settings) {
      settings[key] = value;
    }
  }
</script>

<div class="flex-1 overflow-y-auto p-6 bg-netflix-dark-bg">
  {#if loading}
    <div class="flex items-center justify-center py-20">
      <div class="text-center">
        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-netflix-red mx-auto mb-4"></div>
        <p class="text-gray-400">Loading settings...</p>
      </div>
    </div>
  {:else if settings}
    <div class="max-w-4xl mx-auto">
      <!-- Header -->
      <div class="mb-8">
        <h1 class="text-3xl font-bold text-white mb-2">⚙️ Settings</h1>
        <p class="text-gray-400">Configure your TorrentFlix application</p>
      </div>

      <!-- Notification -->
      {#if notification.visible}
        <SettingsNotification 
          message={notification.message} 
          type={notification.type}
          on:close={() => notification.visible = false}
        />
      {/if}

      <!-- Settings Sections -->
      <div class="space-y-6">
        <!-- API & Metadata Section -->
        <SettingsSection title="🔑 API & Metadata Services" description="Configure API keys for metadata enrichment">
          <div class="space-y-4">
            <SecureInput
              id="tmdb_api_key"
              label="TMDB API Key"
              bind:value={settings.tmdb_api_key}
              placeholder="Enter TMDB API key"
              showMasked={true}
              error={errors.tmdb_api_key}
            />

            <SecureInput
              id="tmdb_read_access_token"
              label="TMDB Read Access Token"
              bind:value={settings.tmdb_read_access_token}
              placeholder="Enter TMDB read access token"
              showMasked={true}
              error={errors.tmdb_read_access_token}
            />

            <SecureInput
              id="kinopoisk_api_token"
              label="Kinopoisk API Token"
              bind:value={settings.kinopoisk_api_token}
              placeholder="Enter Kinopoisk API token"
              showMasked={true}
              error={errors.kinopoisk_api_token}
            />
          </div>
        </SettingsSection>

        <!-- qBittorrent Section -->
        <SettingsSection title="🧲 qBittorrent Configuration" description="Configure your torrent client connection">
          <div class="space-y-4">
            <div class="flex items-center gap-4">
              <input
                type="checkbox"
                id="qbittorrent_enabled"
                checked={settings.qbittorrent_enabled}
                on:change={(e) => updateSetting('qbittorrent_enabled', e.currentTarget.checked)}
                class="w-4 h-4 rounded"
              />
              <label for="qbittorrent_enabled" class="text-sm font-medium text-gray-300">Enable qBittorrent Integration</label>
            </div>

            {#if settings.qbittorrent_enabled}
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label for="qbittorrent_host" class="block text-sm font-medium text-gray-300 mb-2">Host</label>
                  <input
                    id="qbittorrent_host"
                    type="text"
                    placeholder="localhost"
                    value={settings.qbittorrent_host}
                    on:change={(e) => updateSetting('qbittorrent_host', e.currentTarget.value)}
                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-500 focus:outline-none focus:border-netflix-red"
                  />
                  {#if errors.qbittorrent_host}
                    <p class="text-xs text-red-400 mt-1">{errors.qbittorrent_host}</p>
                  {/if}
                </div>

                <div>
                  <label for="qbittorrent_port" class="block text-sm font-medium text-gray-300 mb-2">Port</label>
                  <input
                    id="qbittorrent_port"
                    type="number"
                    placeholder="5555"
                    value={settings.qbittorrent_port}
                    on:change={(e) => updateSetting('qbittorrent_port', parseInt(e.currentTarget.value) || 5555)}
                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-500 focus:outline-none focus:border-netflix-red"
                  />
                </div>
              </div>

              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label for="qbittorrent_username" class="block text-sm font-medium text-gray-300 mb-2">Username</label>
                  <input
                    id="qbittorrent_username"
                    type="text"
                    placeholder="admin"
                    value={settings.qbittorrent_username}
                    on:change={(e) => updateSetting('qbittorrent_username', e.currentTarget.value)}
                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-500 focus:outline-none focus:border-netflix-red"
                  />
                </div>

                <div>
                  <SecureInput
                    id="qbittorrent_password"
                    label="Password"
                    bind:value={settings.qbittorrent_password}
                    placeholder="Enter password"
                    showMasked={false}
                    error={errors.qbittorrent_password}
                  />
                </div>
              </div>

              <div class="flex items-center gap-4">
                <input
                  type="checkbox"
                  id="qbittorrent_use_ssl"
                  checked={settings.qbittorrent_use_ssl}
                  on:change={(e) => updateSetting('qbittorrent_use_ssl', e.currentTarget.checked)}
                  class="w-4 h-4 rounded"
                />
                <label for="qbittorrent_use_ssl" class="text-sm font-medium text-gray-300">Use HTTPS</label>
              </div>

              <div class="flex items-center gap-4">
                <input
                  type="checkbox"
                  id="qbittorrent_verify_ssl"
                  checked={settings.qbittorrent_verify_ssl}
                  on:change={(e) => updateSetting('qbittorrent_verify_ssl', e.currentTarget.checked)}
                  class="w-4 h-4 rounded"
                />
                <label for="qbittorrent_verify_ssl" class="text-sm font-medium text-gray-300">Verify SSL Certificates</label>
              </div>

              <button
                on:click={testQBittorrentConnection}
                disabled={testingConnection || saving}
                class="w-full px-4 py-2 bg-netflix-red text-white rounded hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {testingConnection ? '🔄 Testing...' : '🧪 Test Connection'}
              </button>
            {/if}
          </div>
        </SettingsSection>

        <!-- Filesystem Paths Section -->
        <SettingsSection title="📁 Filesystem Paths" description="Configure where files are stored">
          <div class="space-y-4">
            <div>
              <label for="library_path" class="block text-sm font-medium text-gray-300 mb-2">Library Path</label>
              <div class="flex gap-2">
                <input
                  id="library_path"
                  type="text"
                  placeholder="/home/user/Downloads/TorrentFlix"
                  value={settings.library_path}
                  on:change={(e) => updateSetting('library_path', e.currentTarget.value)}
                  class="flex-1 px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-500 focus:outline-none focus:border-netflix-red"
                />
                <button
                  on:click={() => pickFolder('library_path')}
                  class="px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white hover:bg-gray-600 transition-colors"
                  title="Browse for library path"
                >
                  📂
                </button>
              </div>
              {#if errors.library_path}
                <p class="text-xs text-red-400 mt-1">{errors.library_path}</p>
              {/if}
            </div>

            <div>
              <label for="download_path" class="block text-sm font-medium text-gray-300 mb-2">Download Path</label>
              <div class="flex gap-2">
                <input
                  id="download_path"
                  type="text"
                  placeholder="/home/user/Downloads/qBittorrent"
                  value={settings.download_path}
                  on:change={(e) => updateSetting('download_path', e.currentTarget.value)}
                  class="flex-1 px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-500 focus:outline-none focus:border-netflix-red"
                />
                <button
                  on:click={() => pickFolder('download_path')}
                  class="px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white hover:bg-gray-600 transition-colors"
                  title="Browse for download path"
                >
                  📂
                </button>
              </div>
              {#if errors.download_path}
                <p class="text-xs text-red-400 mt-1">{errors.download_path}</p>
              {/if}
            </div>
          </div>
        </SettingsSection>

        <!-- Notification Settings Section -->
        <SettingsSection title="🔔 Notification Settings" description="Configure notification services">
          <div class="space-y-6">
            <!-- Telegram Configuration -->
            <div class="space-y-4">
              <div class="flex items-center gap-4 pb-2 border-b border-gray-700">
                <input
                  type="checkbox"
                  id="telegram_enabled"
                  checked={settings.telegram_enabled}
                  on:change={(e) => updateSetting('telegram_enabled', e.currentTarget.checked)}
                  class="w-4 h-4 rounded"
                />
                <label for="telegram_enabled" class="text-sm font-medium text-gray-300">Enable Telegram Notifications</label>
              </div>

              {#if settings.telegram_enabled}
                <div class="pl-6 space-y-4">
                  <SecureInput
                    id="telegram_bot_token"
                    label="Telegram Bot Token"
                    bind:value={settings.telegram_bot_token}
                    placeholder="Enter Telegram bot token"
                    showMasked={true}
                    error={errors.telegram_bot_token}
                  />

                  <div>
                    <label for="telegram_chat_id" class="block text-sm font-medium text-gray-300 mb-2">Telegram Chat ID</label>
                    <input
                      id="telegram_chat_id"
                      type="text"
                      placeholder="Enter chat ID"
                      value={settings.telegram_chat_id || ''}
                      on:change={(e) => updateSetting('telegram_chat_id', e.currentTarget.value || undefined)}
                      class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-500 focus:outline-none focus:border-netflix-red"
                    />
                    {#if errors.telegram_chat_id}
                      <p class="text-xs text-red-400 mt-1">{errors.telegram_chat_id}</p>
                    {/if}
                  </div>

                  <div>
                    <label for="telegram_debounce" class="block text-sm font-medium text-gray-300 mb-2">Debounce (seconds)</label>
                    <input
                      id="telegram_debounce"
                      type="number"
                      min="0"
                      max="3600"
                      value={settings.telegram_debounce}
                      on:change={(e) => updateSetting('telegram_debounce', parseInt(e.currentTarget.value) || 300)}
                      class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-netflix-red"
                    />
                    <p class="text-xs text-gray-500 mt-1">Minimum time between notifications (prevents spam)</p>
                  </div>
                </div>
              {/if}
            </div>

            <!-- Email Configuration -->
            <div class="space-y-4">
              <div class="flex items-center gap-4 pb-2 border-b border-gray-700">
                <input
                  type="checkbox"
                  id="email_enabled"
                  checked={settings.email_enabled}
                  on:change={(e) => updateSetting('email_enabled', e.currentTarget.checked)}
                  class="w-4 h-4 rounded"
                />
                <label for="email_enabled" class="text-sm font-medium text-gray-300">Enable Email Notifications</label>
              </div>

              {#if settings.email_enabled}
                <div class="pl-6 space-y-4">
                  <div class="grid grid-cols-2 gap-4">
                    <div>
                      <label for="smtp_host" class="block text-sm font-medium text-gray-300 mb-2">SMTP Host</label>
                      <input
                        id="smtp_host"
                        type="text"
                        placeholder="smtp.gmail.com"
                        value={settings.smtp_host}
                        on:change={(e) => updateSetting('smtp_host', e.currentTarget.value)}
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-500 focus:outline-none focus:border-netflix-red"
                      />
                      {#if errors.smtp_host}
                        <p class="text-xs text-red-400 mt-1">{errors.smtp_host}</p>
                      {/if}
                    </div>

                    <div>
                      <label for="smtp_port" class="block text-sm font-medium text-gray-300 mb-2">SMTP Port</label>
                      <input
                        id="smtp_port"
                        type="number"
                        min="1"
                        max="65535"
                        placeholder="587"
                        value={settings.smtp_port}
                        on:change={(e) => updateSetting('smtp_port', parseInt(e.currentTarget.value) || 587)}
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-500 focus:outline-none focus:border-netflix-red"
                      />
                    </div>
                  </div>

                  <div>
                    <label for="smtp_user" class="block text-sm font-medium text-gray-300 mb-2">SMTP Username</label>
                    <input
                      id="smtp_user"
                      type="text"
                      placeholder="your-email@gmail.com"
                      value={settings.smtp_user || ''}
                      on:change={(e) => updateSetting('smtp_user', e.currentTarget.value || undefined)}
                      class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-500 focus:outline-none focus:border-netflix-red"
                    />
                    {#if errors.smtp_user}
                      <p class="text-xs text-red-400 mt-1">{errors.smtp_user}</p>
                    {/if}
                  </div>

                  <SecureInput
                    id="smtp_password"
                    label="SMTP Password"
                    bind:value={settings.smtp_password}
                    placeholder="Enter SMTP password"
                    showMasked={false}
                    error={errors.smtp_password}
                  />

                  <div class="grid grid-cols-2 gap-4">
                    <div>
                      <label for="from_email" class="block text-sm font-medium text-gray-300 mb-2">From Email</label>
                      <input
                        id="from_email"
                        type="email"
                        placeholder="notifications@example.com"
                        value={settings.from_email || ''}
                        on:change={(e) => updateSetting('from_email', e.currentTarget.value || undefined)}
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-500 focus:outline-none focus:border-netflix-red"
                      />
                      {#if errors.from_email}
                        <p class="text-xs text-red-400 mt-1">{errors.from_email}</p>
                      {/if}
                    </div>

                    <div>
                      <label for="to_email" class="block text-sm font-medium text-gray-300 mb-2">To Email</label>
                      <input
                        id="to_email"
                        type="email"
                        placeholder="your-email@example.com"
                        value={settings.to_email || ''}
                        on:change={(e) => updateSetting('to_email', e.currentTarget.value || undefined)}
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-500 focus:outline-none focus:border-netflix-red"
                      />
                      {#if errors.to_email}
                        <p class="text-xs text-red-400 mt-1">{errors.to_email}</p>
                      {/if}
                    </div>
                  </div>
                </div>
              {/if}
            </div>
          </div>
        </SettingsSection>

        <!-- Application Settings Section -->
        <SettingsSection title="🎛️ Application Settings" description="Configure application behavior">
          <div class="space-y-4">
            <div>
              <label for="log_level" class="block text-sm font-medium text-gray-300 mb-2">Log Level</label>
              <select
                id="log_level"
                value={settings.log_level}
                on:change={(e) => updateSetting('log_level', e.currentTarget.value)}
                class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-netflix-red"
              >
                <option value="DEBUG">DEBUG</option>
                <option value="INFO">INFO</option>
                <option value="WARN">WARN</option>
                <option value="ERROR">ERROR</option>
              </select>
            </div>

            <div class="grid grid-cols-3 gap-4">
              <div>
                <label for="search_limit" class="block text-sm font-medium text-gray-300 mb-2">Search Limit</label>
                <input
                  id="search_limit"
                  type="number"
                  min="1"
                  max="1000"
                  value={settings.search_limit}
                  on:change={(e) => {
                    const value = parseInt(e.currentTarget.value) || 100;
                    const clamped = Math.max(1, Math.min(1000, value));
                    updateSetting('search_limit', clamped);
                    if (value !== clamped) {
                      e.currentTarget.value = clamped.toString();
                    }
                  }}
                  class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-netflix-red"
                />
                <p class="text-xs text-gray-500 mt-1">Range: 1-1000</p>
                {#if errors.search_limit}
                  <p class="text-xs text-red-400 mt-1">{errors.search_limit}</p>
                {/if}
              </div>

              <div>
                <label for="search_timeout" class="block text-sm font-medium text-gray-300 mb-2">Search Timeout (s)</label>
                <input
                  id="search_timeout"
                  type="number"
                  min="1"
                  max="300"
                  value={settings.search_timeout}
                  on:change={(e) => {
                    const value = parseInt(e.currentTarget.value) || 30;
                    const clamped = Math.max(1, Math.min(300, value));
                    updateSetting('search_timeout', clamped);
                    if (value !== clamped) {
                      e.currentTarget.value = clamped.toString();
                    }
                  }}
                  class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-netflix-red"
                />
                <p class="text-xs text-gray-500 mt-1">Range: 1-300 seconds</p>
                {#if errors.search_timeout}
                  <p class="text-xs text-red-400 mt-1">{errors.search_timeout}</p>
                {/if}
              </div>

              <div>
                <label for="max_concurrent_downloads" class="block text-sm font-medium text-gray-300 mb-2">Max Downloads</label>
                <input
                  id="max_concurrent_downloads"
                  type="number"
                  min="1"
                  max="100"
                  value={settings.max_concurrent_downloads}
                  on:change={(e) => {
                    const value = parseInt(e.currentTarget.value) || 3;
                    const clamped = Math.max(1, Math.min(100, value));
                    updateSetting('max_concurrent_downloads', clamped);
                    if (value !== clamped) {
                      e.currentTarget.value = clamped.toString();
                    }
                  }}
                  class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-netflix-red"
                />
                <p class="text-xs text-gray-500 mt-1">Range: 1-100</p>
                {#if errors.max_concurrent_downloads}
                  <p class="text-xs text-red-400 mt-1">{errors.max_concurrent_downloads}</p>
                {/if}
              </div>
            </div>

            <div>
              <label for="enabled_indexers" class="block text-sm font-medium text-gray-300 mb-2">Enabled Indexers</label>
              <input
                id="enabled_indexers"
                type="text"
                placeholder="monna"
                value={settings.enabled_indexers}
                on:change={(e) => updateSetting('enabled_indexers', e.currentTarget.value)}
                class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-500 focus:outline-none focus:border-netflix-red"
              />
              <p class="text-xs text-gray-500 mt-1">Comma-separated list of indexers (e.g., monna,other)</p>
              {#if errors.enabled_indexers}
                <p class="text-xs text-red-400 mt-1">{errors.enabled_indexers}</p>
              {/if}
            </div>

            <div class="flex items-center gap-4">
              <input
                type="checkbox"
                id="debug"
                checked={settings.debug}
                on:change={(e) => updateSetting('debug', e.currentTarget.checked)}
                class="w-4 h-4 rounded"
              />
              <label for="debug" class="text-sm font-medium text-gray-300">Enable Debug Mode</label>
            </div>
          </div>
        </SettingsSection>

        <!-- Proxy Settings Section -->
        <SettingsSection title="🌐 Proxy Configuration" description="Configure SOCKS5 proxy settings">
          <div class="space-y-4">
            <div class="flex items-center gap-4">
              <input
                type="checkbox"
                id="proxy_enabled"
                checked={settings.proxy_enabled}
                on:change={(e) => updateSetting('proxy_enabled', e.currentTarget.checked)}
                class="w-4 h-4 rounded"
              />
              <label for="proxy_enabled" class="text-sm font-medium text-gray-300">Enable Proxy</label>
            </div>

            {#if settings.proxy_enabled}
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label for="proxy_address" class="block text-sm font-medium text-gray-300 mb-2">Proxy Address</label>
                  <input
                    id="proxy_address"
                    type="text"
                    placeholder="127.0.0.1"
                    value={settings.proxy_address}
                    on:change={(e) => updateSetting('proxy_address', e.currentTarget.value)}
                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-500 focus:outline-none focus:border-netflix-red"
                  />
                </div>

                <div>
                  <label for="proxy_port" class="block text-sm font-medium text-gray-300 mb-2">Proxy Port</label>
                  <input
                    id="proxy_port"
                    type="number"
                    placeholder="1080"
                    value={settings.proxy_port}
                    on:change={(e) => updateSetting('proxy_port', parseInt(e.currentTarget.value) || 1080)}
                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-500 focus:outline-none focus:border-netflix-red"
                  />
                </div>
              </div>
            {/if}
          </div>
        </SettingsSection>

        <!-- Save Button -->
        <div class="flex gap-4 pt-6 border-t border-gray-700">
          <button
            on:click={saveSettings}
            disabled={saving}
            class="flex-1 px-6 py-3 bg-netflix-red text-white rounded font-medium hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {saving ? '💾 Saving...' : '💾 Save Settings'}
          </button>
          <button
            on:click={loadSettings}
            disabled={saving}
            class="px-6 py-3 bg-gray-700 text-white rounded font-medium hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            🔄 Reset
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  :global(.bg-netflix-dark-bg) {
    background-color: #0f0f0f;
  }

  :global(.bg-netflix-red) {
    background-color: #e50914;
  }
</style>
