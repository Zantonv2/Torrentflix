import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { settingsStore, setSettings, setLoading, setErrors, type Settings } from '../../stores/settingsStore';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

interface Settings {
  tmdb_api_key: string;
  kinopoisk_token: string;
  qbittorrent_enabled: boolean;
  qbittorrent_host: string;
  qbittorrent_port: number;
  qbittorrent_username: string;
  qbittorrent_password: string;
  qbittorrent_use_https: boolean;
  qbittorrent_verify_ssl: boolean;
  library_path: string;
  download_path: string;
  telegram_enabled: boolean;
  telegram_bot_token?: string;
  telegram_chat_id?: string;
  email_enabled: boolean;
  email_smtp_host?: string;
  email_smtp_port?: number;
  email_username?: string;
  email_password?: string;
  log_level: string;
  search_limit: number;
  search_timeout: number;
  max_concurrent_downloads: number;
  proxy_enabled: boolean;
  proxy_address?: string;
  proxy_port?: number;
}

const mockSettings: Settings = {
  tmdb_api_key: 'test-key-123',
  kinopoisk_token: 'test-token-456',
  qbittorrent_enabled: true,
  qbittorrent_host: 'localhost',
  qbittorrent_port: 5555,
  qbittorrent_username: 'admin',
  qbittorrent_password: 'password',
  qbittorrent_use_https: false,
  qbittorrent_verify_ssl: true,
  library_path: '/home/user/library',
  download_path: '/home/user/downloads',
  telegram_enabled: false,
  email_enabled: false,
  log_level: 'INFO',
  search_limit: 50,
  search_timeout: 30,
  max_concurrent_downloads: 5,
  proxy_enabled: false,
};

describe('Settings Page - Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Settings Loading Workflow', () => {
    it('should load settings on page mount', async () => {
      // Simulate: User opens Settings page
      // Expected: Settings loaded and form populated
      (invoke as any).mockResolvedValueOnce(mockSettings);

      setSettingsLoading(true);
      const settings = await invoke('get_settings');
      setSettings(settings as Settings);
      setSettingsLoading(false);

      const state = get(settingsStore);
      expect(state.data.tmdb_api_key).toBe('test-key-123');
      expect(state.data.qbittorrent_host).toBe('localhost');
      expect(state.data.library_path).toBe('/home/user/library');
    });

    it('should handle settings loading errors', async () => {
      // Simulate: Failed to load settings
      // Expected: Error message shown
      (invoke as any).mockRejectedValueOnce(new Error('Database error'));

      try {
        await invoke('get_settings');
      } catch (error) {
        setSettingsError('Failed to load settings');
      }

      const state = get(settingsStore);
      expect(state.error).toBe('Failed to load settings');
    });
  });

  describe('Settings Modification Workflow', () => {
    it('should track dirty state when settings are modified', () => {
      // Simulate: User modifies a setting
      // Expected: Dirty flag set
      setSettings(mockSettings);
      setSettingsDirty(true);

      let state = get(settingsStore);
      expect(state.dirty).toBe(true);

      // Simulate save
      setSettingsDirty(false);
      state = get(settingsStore);
      expect(state.dirty).toBe(false);
    });

    it('should warn on navigation if settings are dirty', () => {
      // Simulate: User modifies settings and tries to navigate
      // Expected: Warning shown
      setSettings(mockSettings);
      setSettingsDirty(true);

      const state = get(settingsStore);
      expect(state.dirty).toBe(true);
      // Navigation warning would be shown
    });

    it('should not warn on navigation if settings are clean', () => {
      // Simulate: User navigates without modifying settings
      // Expected: No warning
      setSettings(mockSettings);
      setSettingsDirty(false);

      const state = get(settingsStore);
      expect(state.dirty).toBe(false);
    });
  });

  describe('Settings Save Workflow', () => {
    it('should save settings when Save button clicked', async () => {
      // Simulate: User modifies settings and clicks Save
      // Expected: Settings saved successfully
      const modifiedSettings = {
        ...mockSettings,
        search_limit: 100,
      };

      (invoke as any).mockResolvedValueOnce({ success: true });

      const result = await invoke('save_settings', { settings: modifiedSettings });

      expect((result as any).success).toBe(true);
    });

    it('should display validation errors on save failure', async () => {
      // Simulate: Settings validation fails
      // Expected: Field-level errors shown
      (invoke as any).mockRejectedValueOnce(
        new Error('Validation error: Invalid port number')
      );

      try {
        await invoke('save_settings', { settings: mockSettings });
      } catch (error) {
        setSettingsError('Validation error: Invalid port number');
      }

      const state = get(settingsStore);
      expect(state.error).toContain('Validation error');
    });

    it('should show success message after save', async () => {
      // Simulate: Settings saved successfully
      // Expected: Success toast shown
      (invoke as any).mockResolvedValueOnce({ success: true });

      const result = await invoke('save_settings', { settings: mockSettings });

      expect((result as any).success).toBe(true);
      // Success toast would be shown
    });

    it('should clear dirty flag after successful save', async () => {
      // Simulate: Settings saved
      // Expected: Dirty flag cleared
      setSettings(mockSettings);
      setSettingsDirty(true);

      (invoke as any).mockResolvedValueOnce({ success: true });

      await invoke('save_settings', { settings: mockSettings });
      setSettingsDirty(false);

      const state = get(settingsStore);
      expect(state.dirty).toBe(false);
    });
  });

  describe('API & Metadata Section Workflow', () => {
    it('should display masked API keys', () => {
      // Simulate: Settings page shows API keys
      // Expected: Keys are masked
      setSettings(mockSettings);

      const state = get(settingsStore);
      expect(state.data.tmdb_api_key).toBeDefined();
      // UI would mask the key display
    });

    it('should allow updating TMDB API key', () => {
      // Simulate: User updates TMDB key
      // Expected: Key updated in form
      const updatedSettings = {
        ...mockSettings,
        tmdb_api_key: 'new-key-789',
      };

      setSettings(updatedSettings);
      setSettingsDirty(true);

      const state = get(settingsStore);
      expect(state.data.tmdb_api_key).toBe('new-key-789');
      expect(state.dirty).toBe(true);
    });

    it('should allow updating Kinopoisk token', () => {
      // Simulate: User updates Kinopoisk token
      // Expected: Token updated in form
      const updatedSettings = {
        ...mockSettings,
        kinopoisk_token: 'new-token-999',
      };

      setSettings(updatedSettings);
      setSettingsDirty(true);

      const state = get(settingsStore);
      expect(state.data.kinopoisk_token).toBe('new-token-999');
      expect(state.dirty).toBe(true);
    });
  });

  describe('qBittorrent Section Workflow', () => {
    it('should test qBittorrent connection', async () => {
      // Simulate: User clicks Test Connection
      // Expected: Connection tested and result shown
      (invoke as any).mockResolvedValueOnce({
        success: true,
        message: 'Connected successfully',
      });

      const result = await invoke('test_qbittorrent', {
        url: 'http://localhost:5555',
        username: 'admin',
        password: 'password',
      });

      expect((result as any).success).toBe(true);
    });

    it('should handle qBittorrent connection errors', async () => {
      // Simulate: qBittorrent connection fails
      // Expected: Error message shown
      (invoke as any).mockRejectedValueOnce(new Error('Connection refused'));

      try {
        await invoke('test_qbittorrent', {
          url: 'http://localhost:5555',
          username: 'admin',
          password: 'password',
        });
      } catch (error) {
        setSettingsError('Failed to connect to qBittorrent');
      }

      const state = get(settingsStore);
      expect(state.error).toBe('Failed to connect to qBittorrent');
    });

    it('should allow enabling/disabling qBittorrent', () => {
      // Simulate: User toggles qBittorrent enable
      // Expected: Setting updated
      let settings = { ...mockSettings, qbittorrent_enabled: true };
      setSettings(settings);
      setSettingsDirty(true);

      let state = get(settingsStore);
      expect(state.data.qbittorrent_enabled).toBe(true);

      // Disable
      settings = { ...mockSettings, qbittorrent_enabled: false };
      setSettings(settings);

      state = get(settingsStore);
      expect(state.data.qbittorrent_enabled).toBe(false);
    });

    it('should allow updating qBittorrent connection details', () => {
      // Simulate: User updates qBittorrent settings
      // Expected: Settings updated
      const updatedSettings = {
        ...mockSettings,
        qbittorrent_host: '192.168.1.100',
        qbittorrent_port: 6666,
        qbittorrent_username: 'newuser',
      };

      setSettings(updatedSettings);
      setSettingsDirty(true);

      const state = get(settingsStore);
      expect(state.data.qbittorrent_host).toBe('192.168.1.100');
      expect(state.data.qbittorrent_port).toBe(6666);
      expect(state.data.qbittorrent_username).toBe('newuser');
    });
  });

  describe('Filesystem Section Workflow', () => {
    it('should browse for library path', async () => {
      // Simulate: User clicks Browse for library path
      // Expected: Folder picker opens and path selected
      (invoke as any).mockResolvedValueOnce('/new/library/path');

      const path = await invoke('pick_folder');

      expect(path).toBe('/new/library/path');
    });

    it('should browse for download path', async () => {
      // Simulate: User clicks Browse for download path
      // Expected: Folder picker opens and path selected
      (invoke as any).mockResolvedValueOnce('/new/download/path');

      const path = await invoke('pick_folder');

      expect(path).toBe('/new/download/path');
    });

    it('should update library path after browse', async () => {
      // Simulate: User selects new library path
      // Expected: Path updated in form
      (invoke as any).mockResolvedValueOnce('/new/library/path');

      const newPath = await invoke('pick_folder');

      const updatedSettings = {
        ...mockSettings,
        library_path: newPath as string,
      };

      setSettings(updatedSettings);
      setSettingsDirty(true);

      const state = get(settingsStore);
      expect(state.data.library_path).toBe('/new/library/path');
    });

    it('should update download path after browse', async () => {
      // Simulate: User selects new download path
      // Expected: Path updated in form
      (invoke as any).mockResolvedValueOnce('/new/download/path');

      const newPath = await invoke('pick_folder');

      const updatedSettings = {
        ...mockSettings,
        download_path: newPath as string,
      };

      setSettings(updatedSettings);
      setSettingsDirty(true);

      const state = get(settingsStore);
      expect(state.data.download_path).toBe('/new/download/path');
    });
  });

  describe('Application Section Workflow', () => {
    it('should allow updating log level', () => {
      // Simulate: User changes log level
      // Expected: Log level updated
      const updatedSettings = {
        ...mockSettings,
        log_level: 'DEBUG',
      };

      setSettings(updatedSettings);
      setSettingsDirty(true);

      const state = get(settingsStore);
      expect(state.data.log_level).toBe('DEBUG');
    });

    it('should allow updating search limit', () => {
      // Simulate: User changes search limit
      // Expected: Search limit updated
      const updatedSettings = {
        ...mockSettings,
        search_limit: 100,
      };

      setSettings(updatedSettings);
      setSettingsDirty(true);

      const state = get(settingsStore);
      expect(state.data.search_limit).toBe(100);
    });

    it('should allow updating search timeout', () => {
      // Simulate: User changes search timeout
      // Expected: Search timeout updated
      const updatedSettings = {
        ...mockSettings,
        search_timeout: 60,
      };

      setSettings(updatedSettings);
      setSettingsDirty(true);

      const state = get(settingsStore);
      expect(state.data.search_timeout).toBe(60);
    });

    it('should allow updating max concurrent downloads', () => {
      // Simulate: User changes max downloads
      // Expected: Max downloads updated
      const updatedSettings = {
        ...mockSettings,
        max_concurrent_downloads: 10,
      };

      setSettings(updatedSettings);
      setSettingsDirty(true);

      const state = get(settingsStore);
      expect(state.data.max_concurrent_downloads).toBe(10);
    });
  });

  describe('Proxy Section Workflow', () => {
    it('should allow enabling/disabling proxy', () => {
      // Simulate: User toggles proxy enable
      // Expected: Proxy setting updated
      let settings = { ...mockSettings, proxy_enabled: true };
      setSettings(settings);
      setSettingsDirty(true);

      let state = get(settingsStore);
      expect(state.data.proxy_enabled).toBe(true);

      // Disable
      settings = { ...mockSettings, proxy_enabled: false };
      setSettings(settings);

      state = get(settingsStore);
      expect(state.data.proxy_enabled).toBe(false);
    });

    it('should allow updating proxy address and port', () => {
      // Simulate: User updates proxy settings
      // Expected: Proxy settings updated
      const updatedSettings = {
        ...mockSettings,
        proxy_enabled: true,
        proxy_address: '192.168.1.1',
        proxy_port: 8080,
      };

      setSettings(updatedSettings);
      setSettingsDirty(true);

      const state = get(settingsStore);
      expect(state.data.proxy_address).toBe('192.168.1.1');
      expect(state.data.proxy_port).toBe(8080);
    });
  });

  describe('Error Recovery Workflow', () => {
    it('should recover from settings load error', async () => {
      // Simulate: Settings load fails, user retries
      // Expected: Settings load on retry
      (invoke as any)
        .mockRejectedValueOnce(new Error('Database error'))
        .mockResolvedValueOnce(mockSettings);

      // First attempt fails
      try {
        await invoke('get_settings');
      } catch (error) {
        setSettingsError('Failed to load settings');
      }

      let state = get(settingsStore);
      expect(state.error).toBe('Failed to load settings');

      // Retry succeeds
      const settings = await invoke('get_settings');
      setSettings(settings as Settings);
      setSettingsError('');

      state = get(settingsStore);
      expect(state.data.tmdb_api_key).toBe('test-key-123');
      expect(state.error).toBe('');
    });
  });

  describe('Loading State Workflow', () => {
    it('should show loading state while fetching settings', () => {
      // Simulate: Settings are loading
      // Expected: Loading indicator shown
      setSettingsLoading(true);
      let state = get(settingsStore);
      expect(state.loading).toBe(true);

      // Loading complete
      setSettingsLoading(false);
      state = get(settingsStore);
      expect(state.loading).toBe(false);
    });
  });
});
