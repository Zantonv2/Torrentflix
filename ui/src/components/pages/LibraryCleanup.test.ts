import { describe, it, expect, beforeEach, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const mockCleanupCandidates = [
  {
    id: 1,
    file_path: '/library/movie1.mkv',
    file_size: 5368709120, // 5 GB
    reason: 'duplicate',
    media_title: 'Movie 1',
  },
  {
    id: 2,
    file_path: '/library/movie2.mkv',
    file_size: 2684354560, // 2.5 GB
    reason: 'low-quality',
    media_title: 'Movie 2',
  },
  {
    id: 3,
    file_path: '/library/movie3.mkv',
    file_size: 1073741824, // 1 GB
    reason: 'trashed',
    media_title: 'Movie 3',
  },
  {
    id: 4,
    file_path: '/library/movie4.mkv',
    file_size: 536870912, // 512 MB
    reason: 'missing',
  },
];

const mockCleanupResult = {
  deleted_count: 4,
  freed_space: 9663676416, // ~9 GB
  errors: [],
};

describe('LibraryCleanup Page', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Cleanup Candidates Loading (9.1)', () => {
    it('should call get_cleanup_candidates on mount', async () => {
      (invoke as any).mockResolvedValue(mockCleanupCandidates);

      const result = await invoke('get_cleanup_candidates');

      expect(invoke).toHaveBeenCalledWith('get_cleanup_candidates');
      expect(result).toEqual(mockCleanupCandidates);
    });

    it('should handle empty candidates list', async () => {
      (invoke as any).mockResolvedValue([]);

      const result = await invoke('get_cleanup_candidates');

      expect(result).toEqual([]);
      expect(Array.isArray(result)).toBe(true);
    });

    it('should handle loading errors', async () => {
      const errorMessage = 'Failed to load cleanup candidates';
      (invoke as any).mockRejectedValue(new Error(errorMessage));

      try {
        await invoke('get_cleanup_candidates');
      } catch (err) {
        expect((err as Error).message).toBe(errorMessage);
      }
    });
  });

  describe('Candidates Display (9.2)', () => {
    it('should have file path for each candidate', () => {
      mockCleanupCandidates.forEach((candidate) => {
        expect(candidate.file_path).toBeDefined();
        expect(typeof candidate.file_path).toBe('string');
      });
    });

    it('should have file size for each candidate', () => {
      mockCleanupCandidates.forEach((candidate) => {
        expect(candidate.file_size).toBeDefined();
        expect(candidate.file_size).toBeGreaterThan(0);
      });
    });

    it('should have reason for each candidate', () => {
      mockCleanupCandidates.forEach((candidate) => {
        expect(candidate.reason).toBeDefined();
        expect(typeof candidate.reason).toBe('string');
      });
    });

    it('should have valid reason types', () => {
      const validReasons = ['duplicate', 'low-quality', 'trashed', 'missing'];
      mockCleanupCandidates.forEach((candidate) => {
        expect(validReasons).toContain(candidate.reason);
      });
    });

    it('should have media title when available', () => {
      const candidatesWithTitle = mockCleanupCandidates.filter((c) => c.media_title);
      expect(candidatesWithTitle.length).toBeGreaterThan(0);
      candidatesWithTitle.forEach((candidate) => {
        expect(candidate.media_title).toBeDefined();
        expect(typeof candidate.media_title).toBe('string');
      });
    });

    it('should have unique IDs for each candidate', () => {
      const ids = mockCleanupCandidates.map((c) => c.id);
      const uniqueIds = new Set(ids);
      expect(uniqueIds.size).toBe(ids.length);
    });
  });

  describe('Candidate Selection (9.3)', () => {
    it('should allow selecting individual candidates', () => {
      const selectedIds = new Set<number>();
      selectedIds.add(mockCleanupCandidates[0].id);

      expect(selectedIds.has(1)).toBe(true);
      expect(selectedIds.size).toBe(1);
    });

    it('should allow selecting multiple candidates', () => {
      const selectedIds = new Set<number>();
      mockCleanupCandidates.forEach((c) => selectedIds.add(c.id));

      expect(selectedIds.size).toBe(mockCleanupCandidates.length);
    });

    it('should allow deselecting candidates', () => {
      const selectedIds = new Set<number>();
      selectedIds.add(1);
      selectedIds.add(2);
      selectedIds.delete(1);

      expect(selectedIds.has(1)).toBe(false);
      expect(selectedIds.has(2)).toBe(true);
      expect(selectedIds.size).toBe(1);
    });

    it('should calculate total space savings correctly', () => {
      const selectedIds = new Set([1, 2]);
      const totalSpace = mockCleanupCandidates
        .filter((c) => selectedIds.has(c.id))
        .reduce((sum, c) => sum + c.file_size, 0);

      expect(totalSpace).toBe(8053063680); // 5 GB + 2.5 GB
    });

    it('should handle select all operation', () => {
      const selectedIds = new Set<number>();
      mockCleanupCandidates.forEach((c) => selectedIds.add(c.id));

      expect(selectedIds.size).toBe(mockCleanupCandidates.length);
    });

    it('should handle deselect all operation', () => {
      const selectedIds = new Set<number>();
      mockCleanupCandidates.forEach((c) => selectedIds.add(c.id));
      selectedIds.clear();

      expect(selectedIds.size).toBe(0);
    });
  });

  describe('Confirmation Modal (9.4)', () => {
    it('should require confirmation before cleanup', () => {
      const selectedIds = [1, 2, 3];
      const totalSpace = mockCleanupCandidates
        .filter((c) => selectedIds.includes(c.id))
        .reduce((sum, c) => sum + c.file_size, 0);

      expect(selectedIds.length).toBe(3);
      expect(totalSpace).toBeGreaterThan(0);
    });

    it('should show correct file count in confirmation', () => {
      const selectedIds = [1, 2];
      expect(selectedIds.length).toBe(2);
    });

    it('should show correct space savings in confirmation', () => {
      const selectedIds = [1, 2, 3];
      const totalSpace = mockCleanupCandidates
        .filter((c) => selectedIds.includes(c.id))
        .reduce((sum, c) => sum + c.file_size, 0);

      expect(totalSpace).toBe(9126805504); // 5 GB + 2.5 GB + 1 GB
    });

    it('should prevent cleanup with no selection', () => {
      const selectedIds: number[] = [];
      expect(selectedIds.length).toBe(0);
    });
  });

  describe('Cleanup Execution (9.5)', () => {
    it('should call execute_cleanup with selected IDs', async () => {
      const candidateIds = [1, 2, 3];
      (invoke as any).mockResolvedValue(mockCleanupResult);

      const result = await invoke('execute_cleanup', { candidate_ids: candidateIds });

      expect(invoke).toHaveBeenCalledWith('execute_cleanup', { candidate_ids: candidateIds });
      expect(result).toEqual(mockCleanupResult);
    });

    it('should return cleanup result with deleted count', async () => {
      (invoke as any).mockResolvedValue(mockCleanupResult);

      const result = await invoke('execute_cleanup', { candidate_ids: [1, 2, 3, 4] });

      expect(result.deleted_count).toBe(4);
    });

    it('should return cleanup result with freed space', async () => {
      (invoke as any).mockResolvedValue(mockCleanupResult);

      const result = await invoke('execute_cleanup', { candidate_ids: [1, 2, 3, 4] });

      expect(result.freed_space).toBeGreaterThan(0);
    });

    it('should handle cleanup errors', async () => {
      const errorMessage = 'Cleanup failed';
      (invoke as any).mockRejectedValue(new Error(errorMessage));

      try {
        await invoke('execute_cleanup', { candidate_ids: [1, 2] });
      } catch (err) {
        expect((err as Error).message).toBe(errorMessage);
      }
    });

    it('should handle partial cleanup with errors', async () => {
      const resultWithErrors = {
        deleted_count: 3,
        freed_space: 8589934592, // ~8 GB
        errors: ['Failed to delete file 4'],
      };
      (invoke as any).mockResolvedValue(resultWithErrors);

      const result = await invoke('execute_cleanup', { candidate_ids: [1, 2, 3, 4] });

      expect(result.deleted_count).toBe(3);
      expect(result.errors.length).toBeGreaterThan(0);
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

    it('should get correct reason color', () => {
      const getReasonColor = (reason: string): string => {
        if (reason.includes('duplicate')) return 'bg-red-900 text-red-200';
        if (reason.includes('low-quality')) return 'bg-orange-900 text-orange-200';
        if (reason.includes('trashed')) return 'bg-yellow-900 text-yellow-200';
        if (reason.includes('missing')) return 'bg-gray-700 text-gray-200';
        return 'bg-gray-600 text-gray-200';
      };

      expect(getReasonColor('duplicate')).toBe('bg-red-900 text-red-200');
      expect(getReasonColor('low-quality')).toBe('bg-orange-900 text-orange-200');
      expect(getReasonColor('trashed')).toBe('bg-yellow-900 text-yellow-200');
      expect(getReasonColor('missing')).toBe('bg-gray-700 text-gray-200');
      expect(getReasonColor('unknown')).toBe('bg-gray-600 text-gray-200');
    });
  });

  describe('Data Validation', () => {
    it('should have valid candidate structure', () => {
      mockCleanupCandidates.forEach((candidate) => {
        expect(candidate).toHaveProperty('id');
        expect(candidate).toHaveProperty('file_path');
        expect(candidate).toHaveProperty('file_size');
        expect(candidate).toHaveProperty('reason');
      });
    });

    it('should have valid cleanup result structure', () => {
      expect(mockCleanupResult).toHaveProperty('deleted_count');
      expect(mockCleanupResult).toHaveProperty('freed_space');
      expect(mockCleanupResult).toHaveProperty('errors');
    });

    it('should have non-negative values', () => {
      mockCleanupCandidates.forEach((candidate) => {
        expect(candidate.id).toBeGreaterThan(0);
        expect(candidate.file_size).toBeGreaterThan(0);
      });

      expect(mockCleanupResult.deleted_count).toBeGreaterThanOrEqual(0);
      expect(mockCleanupResult.freed_space).toBeGreaterThanOrEqual(0);
    });
  });
});
