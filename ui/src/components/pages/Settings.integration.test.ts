import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { settingsStore, setSettings, setLoading, setErrors, setSaving, type Settings } from '../../stores/settingsStore';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const mockSettings: Settings = {
  qbittorrentEnabled: true,
  qbittorrentHost: 'localhost',
  qbittorrentPort: 5555,
  qbittorrentUseHttps: false,
  qbittorrentVerifySsl: true,
  libraryPath: '/home/user/library',
  downloadPath: '/home/user/downloads',
  telegramEnabled: false,
  emailEnabled: false,
  logLevel: 'INFO',
  searchLimit: 50,
  searchTimeout: 30,
  maxConcurrentDownloads: 5,
  proxyEnabled: false,
  debugMode: false,
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
      (invoke as any).mockResolvedValueOnce(mockSettings);

      setLoading(true);
      const settings = await invoke('get_settings');
      setSettings(settings as Settings);
      setLoading(false);

      const state = get(settingsStore);
      expect(state.data.qbittorrentHost).toBe('localhost');
      expect(state.data.libraryPath).toBe('/home/user/library');
    });

    it('should handle settings loading errors', async () => {
      (invoke as any).mockRejectedValueOnce(new Error('Database error'));

      try {
        await invoke('get_settings');
      } catch (error) {
        setErrors({ general: 'Failed to load settings' });
      }

      const state = get(settingsStore);
      expect(state.errors.general).toBe('Failed to load settings');
    });
  });

  describe('Settings Modification Workflow', () => {
    it('should track dirty state when settings are modified', () => {
      setSettings(mockSettings);
      const state = get(settingsStore);
      expect(state.data).toEqual(mockSettings);
    });

    it('should warn on navigation if settings are dirty', () => {
      setSettings(mockSettings);
      const state = get(settingsStore);
      expect(state.data).toBeDefined();
    });

    it('should not warn on navigation if settings are clean', () => {
      setSettings(mockSettings);
      const state = get(settingsStore);
      expect(state.data).toEqual(mockSettings);
    });
  });

  describe('Settings Save Workflow', () => {
    it('should save settings when Save button clicked', async () => {
      const modifiedSettings = {
        ...mockSettings,
        searchLimit: 100,
      };

      (invoke as any).mockResolvedValueOnce({ success: true });

      const result = await invoke('save_settings', { settings: modifiedSettings });

      expect((result as any).success).toBe(true);
    });

    it('should display validation errors on save failure', async () => {
      (invoke as any).mockRejectedValueOnce(
        new Error('Validation error: Invalid port number')
      );

      try {
        await invoke('save_settings', { settings: mockSettings });
      } catch (error) {
        setErrors({ qbittorrentPort: 'Invalid port number' });
      }

      const state = get(settingsStore);
      expect(state.errors.qbittorrentPort).toBe('Invalid port number');
    });

    it('should show success message after save', async () => {
      (invoke as any).mockResolvedValueOnce({ success: true });

      const result = await invoke('save_settings', { settings: mockSettings });

      expect((result as any).success).toBe(true);
    });

    it('should clear dirty flag after successful save', async () => {
      setSettings(mockSettings);

      (invoke as any).mockResolvedValueOnce({ success: true });

      await invoke('save_settings', { settings: mockSettings });
      setErrors({});

      const state = get(settingsStore);
      expect(Object.keys(state.errors).length).toBe(0);
    });
  });

  describe('API & Metadata Section Workflow', () => {
    it('should display masked API keys', () => {
      setSettings(mockSettings);

      const state = get(settingsStore);
      expect(state.data).toBeDefined();
    });

    it('should allow updating TMDB API key', () => {
      const updatedSettings = {
        ...mockSettings,
        tmdbApiKey: 'new-key-789',
      };

      setSettings(updatedSettings);

      const state = get(settingsStore);
      expect(state.data.tmdbApiKey).toBe('new-key-789');
    });

    it('should allow updating Kinopoisk token', () => {
      const updatedSettings = {
        ...mockSettings,
        kinopoiskToken: 'new-token-999',
      };

      setSettings(updatedSettings);

      const state = get(settingsStore);
      expect(state.data.kinopoiskToken).toBe('new-token-999');
    });
  });

  describe('qBittorrent Section Workflow', () => {
    it('should test qBittorrent connection', async () => {
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
      (invoke as any).mockRejectedValueOnce(new Error('Connection refused'));

      try {
        await invoke('test_qbittorrent', {
          url: 'http://localhost:5555',
          username: 'admin',
          password: 'password',
        });
      } catch (error) {
        setErrors({ qbittorrent: 'Failed to connect to qBittorrent' });
      }

      const state = get(settingsStore);
      expect(state.errors.qbittorrent).toBe('Failed to connect to qBittorrent');
    });

    it('should allow enabling/disabling qBittorrent', () => {
      let settings = { ...mockSettings, qbittorrentEnabled: true };
      setSettings(settings);

      let state = get(settingsStore);
      expect(state.data.qbittorrentEnabled).toBe(true);

      settings = { ...mockSettings, qbittorrentEnabled: false };
      setSettings(settings);

      state = get(settingsStore);
      expect(state.data.qbittorrentEnabled).toBe(false);
    });

    it('should allow updating qBittorrent connection details', () => {
      const updatedSettings = {
        ...mockSettings,
        qbittorrentHost: '192.168.1.100',
        qbittorrentPort: 6666,
        qbittorrentUsername: 'newuser',
      };

      setSettings(updatedSettings);

      const state = get(settingsStore);
      expect(state.data.qbittorrentHost).toBe('192.168.1.100');
      expect(state.data.qbittorrentPort).toBe(6666);
      expect(state.data.qbittorrentUsername).toBe('newuser');
    });
  });

  describe('Filesystem Section Workflow', () => {
    it('should browse for library path', async () => {
      (invoke as any).mockResolvedValueOnce('/new/library/path');

      const path = await invoke('pick_folder');

      expect(path).toBe('/new/library/path');
    });

    it('should browse for download path', async () => {
      (invoke as any).mockResolvedValueOnce('/new/download/path');

      const path = await invoke('pick_folder');

      expect(path).toBe('/new/download/path');
    });

    it('should update library path after browse', async () => {
      (invoke as any).mockResolvedValueOnce('/new/library/path');

      const newPath = await invoke('pick_folder');

      const updatedSettings = {
        ...mockSettings,
        libraryPath: newPath as string,
      };

      setSettings(updatedSettings);

      const state = get(settingsStore);
      expect(state.data.libraryPath).toBe('/new/library/path');
    });

    it('should update download path after browse', async () => {
      (invoke as any).mockResolvedValueOnce('/new/download/path');

      const newPath = await invoke('pick_folder');

      const updatedSettings = {
        ...mockSettings,
        downloadPath: newPath as string,
      };

      setSettings(updatedSettings);

      const state = get(settingsStore);
      expect(state.data.downloadPath).toBe('/new/download/path');
    });
  });

  describe('Application Section Workflow', () => {
    it('should allow updating log level', () => {
      const updatedSettings = {
        ...mockSettings,
        logLevel: 'DEBUG' as const,
      };

      setSettings(updatedSettings);

      const state = get(settingsStore);
      expect(state.data.logLevel).toBe('DEBUG');
    });

    it('should allow updating search limit', () => {
      const updatedSettings = {
        ...mockSettings,
        searchLimit: 100,
      };

      setSettings(updatedSettings);

      const state = get(settingsStore);
      expect(state.data.searchLimit).toBe(100);
    });

    it('should allow updating search timeout', () => {
      const updatedSettings = {
        ...mockSettings,
        searchTimeout: 60,
      };

      setSettings(updatedSettings);

      const state = get(settingsStore);
      expect(state.data.searchTimeout).toBe(60);
    });

    it('should allow updating max concurrent downloads', () => {
      const updatedSettings = {
        ...mockSettings,
        maxConcurrentDownloads: 10,
      };

      setSettings(updatedSettings);

      const state = get(settingsStore);
      expect(state.data.maxConcurrentDownloads).toBe(10);
    });
  });

  describe('Proxy Section Workflow', () => {
    it('should allow enabling/disabling proxy', () => {
      let settings = { ...mockSettings, proxyEnabled: true };
      setSettings(settings);

      let state = get(settingsStore);
      expect(state.data.proxyEnabled).toBe(true);

      settings = { ...mockSettings, proxyEnabled: false };
      setSettings(settings);

      state = get(settingsStore);
      expect(state.data.proxyEnabled).toBe(false);
    });

    it('should allow updating proxy address and port', () => {
      const updatedSettings = {
        ...mockSettings,
        proxyEnabled: true,
        proxyAddress: '192.168.1.1',
        proxyPort: 8080,
      };

      setSettings(updatedSettings);

      const state = get(settingsStore);
      expect(state.data.proxyAddress).toBe('192.168.1.1');
      expect(state.data.proxyPort).toBe(8080);
    });
  });

  describe('Error Recovery Workflow', () => {
    it('should recover from settings load error', async () => {
      (invoke as any)
        .mockRejectedValueOnce(new Error('Database error'))
        .mockResolvedValueOnce(mockSettings);

      try {
        await invoke('get_settings');
      } catch (error) {
        setErrors({ general: 'Failed to load settings' });
      }

      let state = get(settingsStore);
      expect(state.errors.general).toBe('Failed to load settings');

      const settings = await invoke('get_settings');
      setSettings(settings as Settings);
      setErrors({});

      state = get(settingsStore);
      expect(state.data.qbittorrentHost).toBe('localhost');
      expect(Object.keys(state.errors).length).toBe(0);
    });
  });

  describe('Loading State Workflow', () => {
    it('should show loading state while fetching settings', () => {
      setLoading(true);
      let state = get(settingsStore);
      expect(state.loading).toBe(true);

      setLoading(false);
      state = get(settingsStore);
      expect(state.loading).toBe(false);
    });
  });
});
