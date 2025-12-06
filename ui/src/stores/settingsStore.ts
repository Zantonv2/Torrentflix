import { writable, derived } from 'svelte/store';

export interface Settings {
  // API & Metadata
  tmdbApiKey?: string;
  kinopoiskToken?: string;

  // qBittorrent
  qbittorrentEnabled: boolean;
  qbittorrentHost: string;
  qbittorrentPort: number;
  qbittorrentUsername?: string;
  qbittorrentPassword?: string;
  qbittorrentUseHttps: boolean;
  qbittorrentVerifySsl: boolean;

  // Filesystem
  libraryPath: string;
  downloadPath: string;

  // Notifications
  telegramEnabled: boolean;
  telegramBotToken?: string;
  telegramChatId?: string;
  telegramDebounce?: number;

  emailEnabled: boolean;
  emailSmtpHost?: string;
  emailSmtpPort?: number;
  emailUsername?: string;
  emailPassword?: string;
  emailFromAddress?: string;
  emailToAddress?: string;

  // Application
  logLevel: 'DEBUG' | 'INFO' | 'WARN' | 'ERROR';
  searchLimit: number;
  searchTimeout: number;
  maxConcurrentDownloads: number;
  enabledIndexers?: string;
  debugMode: boolean;

  // Proxy
  proxyEnabled: boolean;
  proxyAddress?: string;
  proxyPort?: number;
}

export interface SettingsState {
  data: Settings;
  originalData: Settings; // For dirty state tracking
  loading: boolean;
  saving: boolean;
  errors: Record<string, string>; // Field-level validation errors
}

const defaultSettings: Settings = {
  qbittorrentEnabled: true,
  qbittorrentHost: 'localhost',
  qbittorrentPort: 5555,
  qbittorrentUseHttps: false,
  qbittorrentVerifySsl: true,
  libraryPath: '',
  downloadPath: '',
  telegramEnabled: false,
  emailEnabled: false,
  logLevel: 'INFO',
  searchLimit: 50,
  searchTimeout: 30,
  maxConcurrentDownloads: 3,
  debugMode: false,
  proxyEnabled: false,
};

const initialState: SettingsState = {
  data: { ...defaultSettings },
  originalData: { ...defaultSettings },
  loading: false,
  saving: false,
  errors: {},
};

// Create the main settings store
export const settingsStore = writable<SettingsState>(initialState);

// Helper functions to update the store
export const setSettings = (settings: Settings) => {
  settingsStore.update((state) => ({
    ...state,
    data: settings,
    originalData: JSON.parse(JSON.stringify(settings)), // Deep copy
    loading: false,
    errors: {},
  }));
};

export const updateSetting = <K extends keyof Settings>(key: K, value: Settings[K]) => {
  settingsStore.update((state) => ({
    ...state,
    data: { ...state.data, [key]: value },
  }));
};

export const setLoading = (loading: boolean) => {
  settingsStore.update((state) => ({ ...state, loading }));
};

export const setSaving = (saving: boolean) => {
  settingsStore.update((state) => ({ ...state, saving }));
};

export const setErrors = (errors: Record<string, string>) => {
  settingsStore.update((state) => ({ ...state, errors }));
};

export const setFieldError = (field: string, error: string | null) => {
  settingsStore.update((state) => {
    const newErrors = { ...state.errors };
    if (error) {
      newErrors[field] = error;
    } else {
      delete newErrors[field];
    }
    return { ...state, errors: newErrors };
  });
};

export const clearErrors = () => {
  settingsStore.update((state) => ({ ...state, errors: {} }));
};

export const resetSettings = () => {
  settingsStore.update((state) => ({
    ...state,
    data: JSON.parse(JSON.stringify(state.originalData)), // Deep copy
    errors: {},
  }));
};

export const clearSettings = () => {
  settingsStore.set(initialState);
};

// Derived store for dirty state (has unsaved changes)
export const isDirty = derived(settingsStore, ($settingsStore) => {
  return JSON.stringify($settingsStore.data) !== JSON.stringify($settingsStore.originalData);
});

// Derived store for checking if there are validation errors
export const hasErrors = derived(
  settingsStore,
  ($settingsStore) => Object.keys($settingsStore.errors).length > 0
);

// Derived store for checking if form is busy (loading or saving)
export const isBusy = derived(
  settingsStore,
  ($settingsStore) => $settingsStore.loading || $settingsStore.saving
);
