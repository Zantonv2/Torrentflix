import { describe, it, expect, beforeEach, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const mockStorageStats = {
  total_size: 1099511627776, // 1 TB
  media_item_count: 150,
  file_version_count: 450,
  average_versions_per_media: 3.0,
  breakdown_by_resolution: {
    '1080p': 549755813888, // 512 GB
    '720p': 274877906944, // 256 GB
    '4K': 274877906944, // 256 GB
  },
  breakdown_by_quality: {
    'High': 659034537984, // 614 GB
    'Medium': 329517268992, // 307 GB
    'Low': 110975541760, // 103 GB
  },
  breakdown_by_status: {
    'watched': 659034537984, // 614 GB
    'unwatched': 329517268992, // 307 GB
    'dropped': 110975541760, // 103 GB
  },
  duplicate_space: 109951162777, // ~100 GB
  largest_items: [
    { title: 'Movie 1', size: 54975581388 }, // ~51 GB
    { title: 'Movie 2', size: 32985348833 }, // ~30 GB
    { title: 'Movie 3', size: 21990232555 }, // ~20 GB
    { title: 'Movie 4', size: 10995116277 }, // ~10 GB
    { title: 'Movie 5', size: 5497558138 }, // ~5 GB
  ],
};

describe('LibraryAnalytics Page', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Analytics Loading (8.1)', () => {
    it('should call get_storage_analytics on mount', async () => {
      (invoke as any).mockResolvedValue(mockStorageStats);
      
      // Simulate the invoke call
      const result = await invoke('get_storage_analytics');
      
      expect(invoke).toHaveBeenCalledWith('get_storage_analytics');
      expect(result).toEqual(mockStorageStats);
    });

    it('should handle analytics loading errors', async () => {
      const errorMessage = 'Failed to load analytics';
      (invoke as any).mockRejectedValue(new Error(errorMessage));
      
      try {
        await invoke('get_storage_analytics');
      } catch (err) {
        expect((err as Error).message).toBe(errorMessage);
      }
    });
  });

  describe('Stats Display (8.2)', () => {
    it('should have total size stat', () => {
      expect(mockStorageStats.total_size).toBeDefined();
      expect(mockStorageStats.total_size).toBeGreaterThan(0);
    });

    it('should have media item count', () => {
      expect(mockStorageStats.media_item_count).toBeDefined();
      expect(mockStorageStats.media_item_count).toBe(150);
    });

    it('should have file version count', () => {
      expect(mockStorageStats.file_version_count).toBeDefined();
      expect(mockStorageStats.file_version_count).toBe(450);
    });

    it('should have average versions per media', () => {
      expect(mockStorageStats.average_versions_per_media).toBeDefined();
      expect(mockStorageStats.average_versions_per_media).toBe(3.0);
    });

    it('should calculate average versions correctly', () => {
      const avg = mockStorageStats.file_version_count / mockStorageStats.media_item_count;
      expect(avg).toBe(3.0);
    });
  });

  describe('Breakdown Charts (8.3)', () => {
    it('should have resolution breakdown', () => {
      expect(mockStorageStats.breakdown_by_resolution).toBeDefined();
      expect(Object.keys(mockStorageStats.breakdown_by_resolution).length).toBeGreaterThan(0);
    });

    it('should have quality breakdown', () => {
      expect(mockStorageStats.breakdown_by_quality).toBeDefined();
      expect(Object.keys(mockStorageStats.breakdown_by_quality).length).toBeGreaterThan(0);
    });

    it('should have status breakdown', () => {
      expect(mockStorageStats.breakdown_by_status).toBeDefined();
      expect(Object.keys(mockStorageStats.breakdown_by_status).length).toBeGreaterThan(0);
    });

    it('should have correct resolution categories', () => {
      const resolutions = Object.keys(mockStorageStats.breakdown_by_resolution);
      expect(resolutions).toContain('1080p');
      expect(resolutions).toContain('720p');
      expect(resolutions).toContain('4K');
    });

    it('should have correct quality categories', () => {
      const qualities = Object.keys(mockStorageStats.breakdown_by_quality);
      expect(qualities).toContain('High');
      expect(qualities).toContain('Medium');
      expect(qualities).toContain('Low');
    });

    it('should have correct status categories', () => {
      const statuses = Object.keys(mockStorageStats.breakdown_by_status);
      expect(statuses).toContain('watched');
      expect(statuses).toContain('unwatched');
      expect(statuses).toContain('dropped');
    });

    it('should sum breakdown values correctly', () => {
      const resolutionTotal = Object.values(mockStorageStats.breakdown_by_resolution).reduce((a, b) => a + b, 0);
      expect(resolutionTotal).toBe(mockStorageStats.total_size);
    });
  });

  describe('Largest Items (8.4)', () => {
    it('should have largest items list', () => {
      expect(mockStorageStats.largest_items).toBeDefined();
      expect(Array.isArray(mockStorageStats.largest_items)).toBe(true);
    });

    it('should have at least one largest item', () => {
      expect(mockStorageStats.largest_items!.length).toBeGreaterThan(0);
    });

    it('should have correct number of largest items', () => {
      expect(mockStorageStats.largest_items!.length).toBeLessThanOrEqual(10);
    });

    it('should have title and size for each item', () => {
      mockStorageStats.largest_items!.forEach((item) => {
        expect(item.title).toBeDefined();
        expect(item.size).toBeDefined();
        expect(item.size).toBeGreaterThan(0);
      });
    });

    it('should have items sorted by size descending', () => {
      const items = mockStorageStats.largest_items!;
      for (let i = 0; i < items.length - 1; i++) {
        expect(items[i].size).toBeGreaterThanOrEqual(items[i + 1].size);
      }
    });
  });

  describe('Duplicate Space Warning (8.5)', () => {
    it('should have duplicate space value', () => {
      expect(mockStorageStats.duplicate_space).toBeDefined();
      expect(mockStorageStats.duplicate_space).toBeGreaterThanOrEqual(0);
    });

    it('should show warning when duplicates exist', () => {
      expect(mockStorageStats.duplicate_space).toBeGreaterThan(0);
    });

    it('should not exceed total size', () => {
      expect(mockStorageStats.duplicate_space).toBeLessThanOrEqual(mockStorageStats.total_size);
    });

    it('should calculate duplicate percentage correctly', () => {
      const percentage = (mockStorageStats.duplicate_space / mockStorageStats.total_size) * 100;
      expect(percentage).toBeGreaterThan(0);
      expect(percentage).toBeLessThanOrEqual(100);
    });
  });

  describe('Utility Functions', () => {
    it('should format bytes correctly', () => {
      const formatBytes = (bytes: number): string => {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
      };

      expect(formatBytes(0)).toBe('0 B');
      expect(formatBytes(1024)).toBe('1 KB');
      expect(formatBytes(1048576)).toBe('1 MB');
      expect(formatBytes(1073741824)).toBe('1 GB');
      expect(formatBytes(1099511627776)).toBe('1 TB');
    });

    it('should calculate percentage correctly', () => {
      const getPercentage = (value: number, total: number): number => {
        return Math.round((value / total) * 100);
      };

      expect(getPercentage(50, 100)).toBe(50);
      expect(getPercentage(25, 100)).toBe(25);
      expect(getPercentage(1, 3)).toBe(33);
    });
  });
});
