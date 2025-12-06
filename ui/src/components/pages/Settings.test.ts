import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  settingsStore,
  setSettings,
  updateSetting,
  setSaving,
  setErrors,
  clearSettings,
  isDirty,
  isBusy,
} from '../../stores/settingsStore';
import type { Settings } from '../../stores/settingsStore';

const mockSettings: Settings = {
  tmdbApiKey: 'test-key',
  kinopoiskToken: 'test-token',
  qbittorrentEnabled: true,
  qbittorrentHost: 'localhost',
  qbittorrentPort: 5555,
  qbittorrentUsername: 'admin',
  qbittorrentPassword: 'password',
  qbittorrentUseHttps: false,
  qbittorrentVerifySsl: true,
  libraryPath: '/home/user/library',
  downloadPath: '/home/user/downloads',
  telegramEnabled: false,
  telegramBotToken: '',
  telegramChatId: '',
  telegramDebounce: 300,
  emailEnabled: false,
  emailSmtpHost: '',
  emailSmtpPort: 587,
  emailUsername: '',
  emailPassword: '',
  emailFromAddress: '',
  emailToAddress: '',
  logLevel: 'INFO',
  searchLimit: 50,
  searchTimeout: 30,
  maxConcurrentDownloads: 3,
  enabledIndexers: 'monna',
  debugMode: false,
  proxyEnabled: false,
  proxyAddress: '',
  proxyPort: 1080,
};

describe('Settings Page Logic', () => {
  beforeEach(() => {
    clearSettings();
    vi.clearAllMocks();
  });

  describe('settings loading', () => {
    it('should initialize settings store with loaded data', () => {
      setSettings(mockSettings);

      let state;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      expect(state?.data).toEqual(mockSettings);
      expect(state?.loading).toBe(false);
      unsubscribe();
    });

    it('should handle settings with all fields populated', () => {
      const fullSettings: Settings = {
        ...mockSettings,
        tmdbApiKey: 'tmdb-key-123',
        kinopoiskToken: 'kino-token-456',
        telegramEnabled: true,
        telegramBotToken: 'bot-token',
        telegramChatId: 'chat-123',
        emailEnabled: true,
        emailSmtpHost: 'smtp.gmail.com',
        emailSmtpPort: 587,
        emailUsername: 'user@gmail.com',
        emailPassword: 'pass',
        emailFromAddress: 'from@example.com',
        emailToAddress: 'to@example.com',
        proxyEnabled: true,
        proxyAddress: '127.0.0.1',
        proxyPort: 1080,
      };

      setSettings(fullSettings);

      let state;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      expect(state?.data.tmdbApiKey).toBe('tmdb-key-123');
      expect(state?.data.telegramEnabled).toBe(true);
      expect(state?.data.emailEnabled).toBe(true);
      expect(state?.data.proxyEnabled).toBe(true);
      unsubscribe();
    });
  });

  describe('form validation', () => {
    beforeEach(() => {
      setSettings(mockSettings);
    });

    it('should display all settings sections by having data for each', () => {
      let state;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      // Verify all sections have data
      expect(state?.data.tmdbApiKey).toBeDefined();
      expect(state?.data.qbittorrentHost).toBeDefined();
      expect(state?.data.libraryPath).toBeDefined();
      expect(state?.data.telegramEnabled).toBeDefined();
      expect(state?.data.logLevel).toBeDefined();
      expect(state?.data.proxyEnabled).toBeDefined();

      unsubscribe();
    });

    it('should handle validation errors', () => {
      const errors = {
        libraryPath: 'Path does not exist',
        qbittorrentPort: 'Invalid port',
      };

      setErrors(errors);

      let state;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      expect(state?.errors).toEqual(errors);
      unsubscribe();
    });
  });

  describe('settings saving', () => {
    beforeEach(() => {
      setSettings(mockSettings);
    });

    it('should track saving state', () => {
      let busyValue = false;
      const unsubscribe = isBusy.subscribe((value) => {
        busyValue = value;
      });

      expect(busyValue).toBe(false);

      setSaving(true);
      expect(busyValue).toBe(true);

      setSaving(false);
      expect(busyValue).toBe(false);

      unsubscribe();
    });

    it('should prepare settings for save', () => {
      let state;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      // Settings should be ready to save
      expect(state?.data).toBeDefined();
      expect(state?.data.qbittorrentHost).toBe('localhost');
      expect(state?.data.searchLimit).toBe(50);

      unsubscribe();
    });
  });

  describe('dirty state tracking', () => {
    beforeEach(() => {
      setSettings(mockSettings);
    });

    it('should track dirty state when settings change', () => {
      let dirtyValue = false;
      const unsubscribe = isDirty.subscribe((value) => {
        dirtyValue = value;
      });

      expect(dirtyValue).toBe(false);

      updateSetting('searchLimit', 100);
      expect(dirtyValue).toBe(true);

      unsubscribe();
    });

    it('should not be dirty when reverted to original', () => {
      let dirtyValue = false;
      const unsubscribe = isDirty.subscribe((value) => {
        dirtyValue = value;
      });

      updateSetting('searchLimit', 100);
      expect(dirtyValue).toBe(true);

      updateSetting('searchLimit', 50); // Revert to original
      expect(dirtyValue).toBe(false);

      unsubscribe();
    });
  });

  describe('qBittorrent configuration', () => {
    beforeEach(() => {
      setSettings(mockSettings);
    });

    it('should handle qBittorrent enabled/disabled state', () => {
      let state;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      expect(state?.data.qbittorrentEnabled).toBe(true);

      updateSetting('qbittorrentEnabled', false);
      expect(state?.data.qbittorrentEnabled).toBe(false);

      unsubscribe();
    });

    it('should handle qBittorrent connection parameters', () => {
      let state;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      updateSetting('qbittorrentHost', '192.168.1.100');
      updateSetting('qbittorrentPort', 8080);
      updateSetting('qbittorrentUsername', 'newuser');

      expect(state?.data.qbittorrentHost).toBe('192.168.1.100');
      expect(state?.data.qbittorrentPort).toBe(8080);
      expect(state?.data.qbittorrentUsername).toBe('newuser');

      unsubscribe();
    });

    it('should handle SSL options', () => {
      let state;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      updateSetting('qbittorrentUseHttps', true);
      updateSetting('qbittorrentVerifySsl', false);

      expect(state?.data.qbittorrentUseHttps).toBe(true);
      expect(state?.data.qbittorrentVerifySsl).toBe(false);

      unsubscribe();
    });
  });

  describe('filesystem paths', () => {
    beforeEach(() => {
      setSettings(mockSettings);
    });

    it('should handle library and download paths', () => {
      let state;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      updateSetting('libraryPath', '/new/library');
      updateSetting('downloadPath', '/new/downloads');

      expect(state?.data.libraryPath).toBe('/new/library');
      expect(state?.data.downloadPath).toBe('/new/downloads');

      unsubscribe();
    });
  });

  describe('notification settings', () => {
    beforeEach(() => {
      setSettings(mockSettings);
    });

    it('should handle Telegram settings', () => {
      let state;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      updateSetting('telegramEnabled', true);
      updateSetting('telegramBotToken', 'bot-token-123');
      updateSetting('telegramChatId', 'chat-456');

      expect(state?.data.telegramEnabled).toBe(true);
      expect(state?.data.telegramBotToken).toBe('bot-token-123');
      expect(state?.data.telegramChatId).toBe('chat-456');

      unsubscribe();
    });

    it('should handle Email settings', () => {
      let state;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      updateSetting('emailEnabled', true);
      updateSetting('emailSmtpHost', 'smtp.gmail.com');
      updateSetting('emailSmtpPort', 587);
      updateSetting('emailUsername', 'user@gmail.com');

      expect(state?.data.emailEnabled).toBe(true);
      expect(state?.data.emailSmtpHost).toBe('smtp.gmail.com');
      expect(state?.data.emailSmtpPort).toBe(587);
      expect(state?.data.emailUsername).toBe('user@gmail.com');

      unsubscribe();
    });
  });

  describe('application settings', () => {
    beforeEach(() => {
      setSettings(mockSettings);
    });

    it('should handle log level', () => {
      let state;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      updateSetting('logLevel', 'DEBUG');
      expect(state?.data.logLevel).toBe('DEBUG');

      updateSetting('logLevel', 'ERROR');
      expect(state?.data.logLevel).toBe('ERROR');

      unsubscribe();
    });

    it('should handle search and download limits', () => {
      let state;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      updateSetting('searchLimit', 100);
      updateSetting('searchTimeout', 60);
      updateSetting('maxConcurrentDownloads', 5);

      expect(state?.data.searchLimit).toBe(100);
      expect(state?.data.searchTimeout).toBe(60);
      expect(state?.data.maxConcurrentDownloads).toBe(5);

      unsubscribe();
    });

    it('should handle debug mode', () => {
      let state;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      updateSetting('debugMode', true);
      expect(state?.data.debugMode).toBe(true);

      unsubscribe();
    });
  });

  describe('proxy settings', () => {
    beforeEach(() => {
      setSettings(mockSettings);
    });

    it('should handle proxy enabled/disabled state', () => {
      let state;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      expect(state?.data.proxyEnabled).toBe(false);

      updateSetting('proxyEnabled', true);
      expect(state?.data.proxyEnabled).toBe(true);

      unsubscribe();
    });

    it('should handle proxy address and port', () => {
      let state;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      updateSetting('proxyAddress', '10.0.0.1');
      updateSetting('proxyPort', 9050);

      expect(state?.data.proxyAddress).toBe('10.0.0.1');
      expect(state?.data.proxyPort).toBe(9050);

      unsubscribe();
    });
  });
});
