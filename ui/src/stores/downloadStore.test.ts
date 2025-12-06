import { describe, it, expect, beforeEach } from 'vitest';
import {
  downloadStore,
  setActive,
  updateDownload,
  removeDownload,
  setLoading,
  setError,
  setPolling,
  setPollInterval,
  clearDownloads,
  hasActive,
  totalProgress,
  totalSpeed,
} from './downloadStore';
import type { Download, DownloadState } from './downloadStore';

const mockDownload: Download = {
  hash: 'abc123',
  title: 'Test Movie',
  progress: 50,
  downloadedBytes: 1024 * 1024 * 1024, // 1 GB
  totalBytes: 2 * 1024 * 1024 * 1024, // 2 GB
  speed: 1024 * 1024, // 1 MB/s
  status: 'downloading',
};

describe('downloadStore', () => {
  beforeEach(() => {
    clearDownloads();
  });

  describe('state updates', () => {
    it('should initialize with empty state', () => {
      let state: DownloadState | undefined;
      const unsubscribe = downloadStore.subscribe((s) => {
        state = s;
      });

      expect(state).toEqual({
        active: [],
        loading: false,
        error: null,
        pollInterval: 5000,
        isPolling: false,
      });

      unsubscribe();
    });

    it('should set active downloads and clear loading/error', () => {
      let state: DownloadState | undefined;
      const unsubscribe = downloadStore.subscribe((s) => {
        state = s;
      });

      const downloads = [mockDownload];
      setActive(downloads);

      expect(state?.active).toEqual(downloads);
      expect(state?.loading).toBe(false);
      expect(state?.error).toBeNull();
      unsubscribe();
    });

    it('should update a specific download', () => {
      let state: DownloadState | undefined;
      const unsubscribe = downloadStore.subscribe((s) => {
        state = s;
      });

      setActive([mockDownload]);
      updateDownload('abc123', { progress: 75 });

      expect(state?.active[0].progress).toBe(75);
      expect(state?.active[0].title).toBe('Test Movie');
      unsubscribe();
    });

    it('should remove a download', () => {
      let state: DownloadState | undefined;
      const unsubscribe = downloadStore.subscribe((s) => {
        state = s;
      });

      const download2 = { ...mockDownload, hash: 'def456', title: 'Another Movie' };
      setActive([mockDownload, download2]);

      removeDownload('abc123');

      expect(state?.active).toHaveLength(1);
      expect(state?.active[0].hash).toBe('def456');
      unsubscribe();
    });

    it('should set loading state', () => {
      let state: DownloadState | undefined;
      const unsubscribe = downloadStore.subscribe((s) => {
        state = s;
      });

      setLoading(true);
      expect(state?.loading).toBe(true);

      setLoading(false);
      expect(state?.loading).toBe(false);

      unsubscribe();
    });

    it('should set error and clear loading', () => {
      let state: DownloadState | undefined;
      const unsubscribe = downloadStore.subscribe((s) => {
        state = s;
      });

      setError('Connection failed');

      expect(state?.error).toBe('Connection failed');
      expect(state?.loading).toBe(false);
      unsubscribe();
    });

    it('should set polling state', () => {
      let state: DownloadState | undefined;
      const unsubscribe = downloadStore.subscribe((s) => {
        state = s;
      });

      setPolling(true);
      expect(state?.isPolling).toBe(true);

      setPolling(false);
      expect(state?.isPolling).toBe(false);

      unsubscribe();
    });

    it('should set poll interval', () => {
      let state: DownloadState | undefined;
      const unsubscribe = downloadStore.subscribe((s) => {
        state = s;
      });

      setPollInterval(3000);
      expect(state?.pollInterval).toBe(3000);

      setPollInterval(5000);
      expect(state?.pollInterval).toBe(5000);

      unsubscribe();
    });

    it('should clear downloads to initial state', () => {
      let state: DownloadState | undefined;
      const unsubscribe = downloadStore.subscribe((s) => {
        state = s;
      });

      setActive([mockDownload]);
      setPolling(true);
      setPollInterval(3000);
      clearDownloads();

      expect(state).toEqual({
        active: [],
        loading: false,
        error: null,
        pollInterval: 5000,
        isPolling: false,
      });

      unsubscribe();
    });
  });

  describe('polling lifecycle', () => {
    it('should track polling state independently', () => {
      let state: DownloadState | undefined;
      const unsubscribe = downloadStore.subscribe((s) => {
        state = s;
      });

      expect(state?.isPolling).toBe(false);

      setPolling(true);
      expect(state?.isPolling).toBe(true);

      setActive([mockDownload]);
      expect(state?.isPolling).toBe(true);

      removeDownload('abc123');
      expect(state?.isPolling).toBe(true);

      setPolling(false);
      expect(state?.isPolling).toBe(false);

      unsubscribe();
    });

    it('should maintain poll interval across updates', () => {
      let state: DownloadState | undefined;
      const unsubscribe = downloadStore.subscribe((s) => {
        state = s;
      });

      setPollInterval(3000);
      setActive([mockDownload]);
      updateDownload('abc123', { progress: 75 });

      expect(state?.pollInterval).toBe(3000);
      unsubscribe();
    });

    it('should allow starting and stopping polling', () => {
      let state: DownloadState | undefined;
      const unsubscribe = downloadStore.subscribe((s) => {
        state = s;
      });

      // Start polling
      setPolling(true);
      setActive([mockDownload]);
      expect(state?.isPolling).toBe(true);
      expect(state?.active).toHaveLength(1);

      // Update during polling
      updateDownload('abc123', { progress: 75 });
      expect(state?.isPolling).toBe(true);

      // Stop polling
      setPolling(false);
      expect(state?.isPolling).toBe(false);

      unsubscribe();
    });
  });

  describe('derived stores', () => {
    it('hasActive should be true when downloads exist', () => {
      let hasActiveValue = false;
      const unsubscribe = hasActive.subscribe((value) => {
        hasActiveValue = value;
      });

      expect(hasActiveValue).toBe(false);

      setActive([mockDownload]);
      expect(hasActiveValue).toBe(true);

      clearDownloads();
      expect(hasActiveValue).toBe(false);

      unsubscribe();
    });

    it('totalProgress should calculate average progress', () => {
      let progressValue = 0;
      const unsubscribe = totalProgress.subscribe((value) => {
        progressValue = value;
      });

      expect(progressValue).toBe(0);

      const download1 = { ...mockDownload, progress: 50 };
      const download2 = { ...mockDownload, hash: 'def456', progress: 100 };

      setActive([download1, download2]);
      expect(progressValue).toBe(75); // (50 + 100) / 2

      unsubscribe();
    });

    it('totalSpeed should sum all download speeds', () => {
      let speedValue = 0;
      const unsubscribe = totalSpeed.subscribe((value) => {
        speedValue = value;
      });

      expect(speedValue).toBe(0);

      const download1 = { ...mockDownload, speed: 1024 * 1024 }; // 1 MB/s
      const download2 = { ...mockDownload, hash: 'def456', speed: 512 * 1024 }; // 0.5 MB/s

      setActive([download1, download2]);
      expect(speedValue).toBe(1024 * 1024 + 512 * 1024); // 1.5 MB/s

      unsubscribe();
    });
  });
});
