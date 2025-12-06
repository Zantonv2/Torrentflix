import { describe, it, expect, beforeEach } from 'vitest';
import {
  settingsStore,
  setSettings,
  updateSetting,
  setLoading,
  setSaving,
  setErrors,
  setFieldError,
  clearErrors,
  resetSettings,
  clearSettings,
  isDirty,
  hasErrors,
  isBusy,
} from './settingsStore';
import type { Settings, SettingsState } from './settingsStore';

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
  maxConcurrentDownloads: 3,
  debugMode: false,
  proxyEnabled: false,
};

describe('settingsStore', () => {
  beforeEach(() => {
    clearSettings();
  });

  describe('state updates', () => {
    it('should initialize with default state', () => {
      let state: SettingsState | undefined;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      expect(state?.data).toBeDefined();
      expect(state?.originalData).toBeDefined();
      expect(state?.loading).toBe(false);
      expect(state?.saving).toBe(false);
      expect(state?.errors).toEqual({});

      unsubscribe();
    });

    it('should set settings and update original data', () => {
      let state: SettingsState | undefined;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      setSettings(mockSettings);

      expect(state?.data).toEqual(mockSettings);
      expect(state?.originalData).toEqual(mockSettings);
      expect(state?.loading).toBe(false);
      expect(state?.errors).toEqual({});

      unsubscribe();
    });

    it('should update a single setting', () => {
      let state: SettingsState | undefined;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      setSettings(mockSettings);
      updateSetting('searchLimit', 100);

      expect(state?.data.searchLimit).toBe(100);
      expect(state?.originalData.searchLimit).toBe(50);

      unsubscribe();
    });

    it('should set loading state', () => {
      let state: SettingsState | undefined;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      setLoading(true);
      expect(state?.loading).toBe(true);

      setLoading(false);
      expect(state?.loading).toBe(false);

      unsubscribe();
    });

    it('should set saving state', () => {
      let state: SettingsState | undefined;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      setSaving(true);
      expect(state?.saving).toBe(true);

      setSaving(false);
      expect(state?.saving).toBe(false);

      unsubscribe();
    });

    it('should set multiple errors', () => {
      let state: SettingsState | undefined;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      const errors = {
        libraryPath: 'Path does not exist',
        qbittorrentPort: 'Invalid port number',
      };

      setErrors(errors);

      expect(state?.errors).toEqual(errors);
      unsubscribe();
    });

    it('should set field-level error', () => {
      let state: SettingsState | undefined;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      setFieldError('libraryPath', 'Path does not exist');
      expect(state?.errors.libraryPath).toBe('Path does not exist');

      setFieldError('qbittorrentPort', 'Invalid port');
      expect(state?.errors.qbittorrentPort).toBe('Invalid port');

      unsubscribe();
    });

    it('should clear field error when set to null', () => {
      let state: SettingsState | undefined;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      setFieldError('libraryPath', 'Path does not exist');
      expect(state?.errors.libraryPath).toBe('Path does not exist');

      setFieldError('libraryPath', null);
      expect(state?.errors.libraryPath).toBeUndefined();

      unsubscribe();
    });

    it('should clear all errors', () => {
      let state: SettingsState | undefined;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      setErrors({
        libraryPath: 'Error 1',
        qbittorrentPort: 'Error 2',
      });

      clearErrors();
      expect(state?.errors).toEqual({});

      unsubscribe();
    });

    it('should reset settings to original data', () => {
      let state: SettingsState | undefined;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      setSettings(mockSettings);
      updateSetting('searchLimit', 100);
      setFieldError('libraryPath', 'Error');

      resetSettings();

      expect(state?.data.searchLimit).toBe(50);
      expect(state?.errors).toEqual({});

      unsubscribe();
    });

    it('should clear settings to initial state', () => {
      let state: SettingsState | undefined;
      const unsubscribe = settingsStore.subscribe((s) => {
        state = s;
      });

      setSettings(mockSettings);
      updateSetting('searchLimit', 100);
      clearSettings();

      expect(state?.data.searchLimit).toBe(50);
      expect(state?.originalData.searchLimit).toBe(50);

      unsubscribe();
    });
  });

  describe('dirty state tracking', () => {
    it('should track dirty state when settings change', () => {
      let dirtyValue: boolean | undefined;
      const unsubscribe = isDirty.subscribe((value) => {
        dirtyValue = value;
      });

      setSettings(mockSettings);
      expect(dirtyValue).toBe(false);

      updateSetting('searchLimit', 100);
      expect(dirtyValue).toBe(true);

      resetSettings();
      expect(dirtyValue).toBe(false);

      unsubscribe();
    });

    it('should detect dirty state with multiple changes', () => {
      let dirtyValue: boolean | undefined;
      const unsubscribe = isDirty.subscribe((value) => {
        dirtyValue = value;
      });

      setSettings(mockSettings);
      expect(dirtyValue).toBe(false);

      updateSetting('searchLimit', 100);
      expect(dirtyValue).toBe(true);

      updateSetting('maxConcurrentDownloads', 5);
      expect(dirtyValue).toBe(true);

      resetSettings();
      expect(dirtyValue).toBe(false);

      unsubscribe();
    });

    it('should not be dirty after resetting to original', () => {
      let dirtyValue: boolean | undefined;
      const unsubscribe = isDirty.subscribe((value) => {
        dirtyValue = value;
      });

      setSettings(mockSettings);
      updateSetting('searchLimit', 100);
      expect(dirtyValue).toBe(true);

      updateSetting('searchLimit', 50); // Reset to original
      expect(dirtyValue).toBe(false);

      unsubscribe();
    });
  });

  describe('derived stores', () => {
    it('hasErrors should be true when errors exist', () => {
      let hasErrorsValue: boolean | undefined;
      const unsubscribe = hasErrors.subscribe((value) => {
        hasErrorsValue = value;
      });

      expect(hasErrorsValue).toBe(false);

      setFieldError('libraryPath', 'Error');
      expect(hasErrorsValue).toBe(true);

      clearErrors();
      expect(hasErrorsValue).toBe(false);

      unsubscribe();
    });

    it('isBusy should be true when loading or saving', () => {
      let busyValue: boolean | undefined;
      const unsubscribe = isBusy.subscribe((value) => {
        busyValue = value;
      });

      expect(busyValue).toBe(false);

      setLoading(true);
      expect(busyValue).toBe(true);

      setLoading(false);
      expect(busyValue).toBe(false);

      setSaving(true);
      expect(busyValue).toBe(true);

      setSaving(false);
      expect(busyValue).toBe(false);

      unsubscribe();
    });

    it('isBusy should be true when both loading and saving', () => {
      let busyValue: boolean | undefined;
      const unsubscribe = isBusy.subscribe((value) => {
        busyValue = value;
      });

      setLoading(true);
      setSaving(true);
      expect(busyValue).toBe(true);

      setLoading(false);
      expect(busyValue).toBe(true);

      setSaving(false);
      expect(busyValue).toBe(false);

      unsubscribe();
    });
  });
});
