import { writable, derived } from 'svelte/store';

export interface Download {
  hash: string;
  title: string;
  progress: number; // 0-100
  downloadedBytes: number;
  totalBytes: number;
  speed: number; // bytes per second
  status: 'downloading' | 'paused' | 'seeding' | 'completed';
}

export interface DownloadState {
  active: Download[];
  loading: boolean;
  error: string | null;
  pollInterval: number; // milliseconds
  isPolling: boolean;
}

const initialState: DownloadState = {
  active: [],
  loading: false,
  error: null,
  pollInterval: 5000, // 5 seconds
  isPolling: false,
};

// Create the main download store
export const downloadStore = writable<DownloadState>(initialState);

// Helper functions to update the store
export const setActive = (downloads: Download[]) => {
  downloadStore.update((state) => ({ ...state, active: downloads, loading: false, error: null }));
};

export const updateDownload = (hash: string, updates: Partial<Download>) => {
  downloadStore.update((state) => ({
    ...state,
    active: state.active.map((d) => (d.hash === hash ? { ...d, ...updates } : d)),
  }));
};

export const removeDownload = (hash: string) => {
  downloadStore.update((state) => ({
    ...state,
    active: state.active.filter((d) => d.hash !== hash),
  }));
};

export const setLoading = (loading: boolean) => {
  downloadStore.update((state) => ({ ...state, loading }));
};

export const setError = (error: string | null) => {
  downloadStore.update((state) => ({ ...state, error, loading: false }));
};

export const setPolling = (isPolling: boolean) => {
  downloadStore.update((state) => ({ ...state, isPolling }));
};

export const setPollInterval = (interval: number) => {
  downloadStore.update((state) => ({ ...state, pollInterval: interval }));
};

export const clearDownloads = () => {
  downloadStore.set(initialState);
};

// Derived store for checking if there are active downloads
export const hasActive = derived(downloadStore, ($downloadStore) => $downloadStore.active.length > 0);

// Derived store for total progress
export const totalProgress = derived(downloadStore, ($downloadStore) => {
  if ($downloadStore.active.length === 0) return 0;
  const totalProgress = $downloadStore.active.reduce((sum, d) => sum + d.progress, 0);
  return Math.round(totalProgress / $downloadStore.active.length);
});

// Derived store for total speed
export const totalSpeed = derived(downloadStore, ($downloadStore) => {
  return $downloadStore.active.reduce((sum, d) => sum + d.speed, 0);
});
