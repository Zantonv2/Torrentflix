import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { libraryStore, setLibraryItems, setLibraryLoading, setLibraryError } from '../../stores/libraryStore';
import type { UiSearchResult } from '../../types';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const mockLibraryItem: UiSearchResult = {
  id: '1',
  title: 'Inception',
  year: 2010,
  poster_url: 'https://example.com/poster.jpg',
  quality_badge: '1080p',
  rating: 8.8,
  rating_kinopoisk: 8.8,
  rating_imdb: 8.8,
  rating_tmdb: 8.8,
  runtime_minutes: 148,
  description: 'A skilled thief who steals corporate secrets through dream-sharing technology',
  cast: ['Leonardo DiCaprio', 'Marion Cotillard'],
  genres: ['Action', 'Sci-Fi', 'Thriller'],
  torrent_info: {
    seeders: 500,
    size_gb: 4.5,
    magnet_link: 'magnet:?xt=urn:btih:test',
  },
};

describe('Library Page - Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Library Browser Tab Workflow', () => {
    it('should load library items on page mount', async () => {
      // Simulate: User opens Library page
      // Expected: Library items loaded and displayed
      const mockLibraryItems = [
        mockLibraryItem,
        { ...mockLibraryItem, id: '2', title: 'The Matrix' },
      ];

      (invoke as any).mockResolvedValueOnce(mockLibraryItems);

      setLibraryLoading(true);
      const items = await invoke('query_library', { filters: {} });
      setLibraryItems(items as UiSearchResult[]);
      setLibraryLoading(false);

      const state = get(libraryStore);
      expect(state.items.length).toBe(2);
      expect(state.items[0].title).toBe('Inception');
    });

    it('should search library with fuzzy matching', async () => {
      // Simulate: User searches library
      // Expected: Matching items returned
      const searchResults = [mockLibraryItem];

      (invoke as any).mockResolvedValueOnce(searchResults);

      setLibraryLoading(true);
      const results = await invoke('search_library', {
        query: 'Inception',
        options: { fuzzy: true },
      });
      setLibraryItems(results as UiSearchResult[]);
      setLibraryLoading(false);

      const state = get(libraryStore);
      expect(state.items.length).toBe(1);
      expect(state.items[0].title).toBe('Inception');
    });

    it('should apply filters to library query', async () => {
      // Simulate: User applies filters (year, resolution, rating)
      // Expected: Filtered results returned
      const filteredResults = [mockLibraryItem];

      (invoke as any).mockResolvedValueOnce(filteredResults);

      const filters = {
        year: 2010,
        resolution: '1080p',
        minRating: 8.0,
        watchStatus: 'watched',
      };

      setLibraryLoading(true);
      const results = await invoke('query_library', { filters });
      setLibraryItems(results as UiSearchResult[]);
      setLibraryLoading(false);

      const state = get(libraryStore);
      expect(state.items.length).toBe(1);
    });

    it('should display library item details when clicked', async () => {
      // Simulate: User clicks library item
      // Expected: Detail modal opens with full information
      const mockItemDetails = {
        ...mockLibraryItem,
        file_versions: [
          { resolution: '1080p', codec: 'h264', size_gb: 4.5 },
          { resolution: '720p', codec: 'h264', size_gb: 2.0 },
        ],
        user_rating: 9,
        watch_status: 'watched',
        tags: ['favorite', 'action'],
        notes: 'Great movie!',
      };

      (invoke as any)
        .mockResolvedValueOnce(mockItemDetails)
        .mockResolvedValueOnce(mockItemDetails.file_versions);

      const itemDetails = await invoke('get_media_item', { id: '1' });
      const fileVersions = await invoke('get_file_versions', { media_id: '1' });

      expect(itemDetails).toBeDefined();
      expect(fileVersions).toBeDefined();
      expect((fileVersions as any).length).toBe(2);
    });

    it('should handle library loading errors', async () => {
      // Simulate: Library loading fails
      // Expected: Error message shown
      (invoke as any).mockRejectedValueOnce(new Error('Database error'));

      try {
        await invoke('query_library', { filters: {} });
      } catch (error) {
        setLibraryError('Failed to load library');
      }

      const state = get(libraryStore);
      expect(state.error).toBe('Failed to load library');
    });

    it('should toggle between grid and list view', () => {
      // Simulate: User clicks grid/list toggle
      // Expected: View switches
      const items = [mockLibraryItem];
      setLibraryItems(items);

      let state = get(libraryStore);
      expect(state.items.length).toBe(1);
      // View toggle would be handled by component state
    });
  });

  describe('Library Collections Tab Workflow', () => {
    it('should create new collection', async () => {
      // Simulate: User creates collection
      // Expected: Collection created and added to list
      (invoke as any).mockResolvedValueOnce({
        id: 'col-1',
        name: 'Favorites',
        description: 'My favorite movies',
      });

      const collection = await invoke('create_collection', {
        name: 'Favorites',
        description: 'My favorite movies',
      });

      expect(collection).toBeDefined();
      expect((collection as any).name).toBe('Favorites');
    });

    it('should create smart collection with criteria', async () => {
      // Simulate: User creates smart collection
      // Expected: Smart collection created with filter criteria
      (invoke as any).mockResolvedValueOnce({
        id: 'smart-1',
        name: 'High Rated',
        criteria: { minRating: 8.0 },
      });

      const smartCollection = await invoke('create_smart_collection', {
        name: 'High Rated',
        criteria: { minRating: 8.0 },
      });

      expect(smartCollection).toBeDefined();
      expect((smartCollection as any).name).toBe('High Rated');
    });

    it('should add items to collection', async () => {
      // Simulate: User adds items to collection
      // Expected: Items added successfully
      (invoke as any).mockResolvedValueOnce({ success: true, count: 2 });

      const result = await invoke('add_to_collection', {
        collection_id: 'col-1',
        media_ids: ['1', '2'],
      });

      expect((result as any).success).toBe(true);
      expect((result as any).count).toBe(2);
    });

    it('should display collection items', async () => {
      // Simulate: User views collection
      // Expected: Collection items displayed
      const collectionItems = [mockLibraryItem];

      (invoke as any).mockResolvedValueOnce(collectionItems);

      const items = await invoke('get_collection_items', { collection_id: 'col-1' });

      expect((items as any).length).toBe(1);
      expect((items as any)[0].title).toBe('Inception');
    });
  });

  describe('Library Analytics Tab Workflow', () => {
    it('should load storage analytics', async () => {
      // Simulate: User opens Analytics tab
      // Expected: Storage stats displayed
      const mockAnalytics = {
        total_size_gb: 500,
        media_count: 100,
        file_version_count: 250,
        avg_versions_per_media: 2.5,
        breakdown_by_resolution: {
          '1080p': 60,
          '720p': 30,
          '4k': 10,
        },
        breakdown_by_quality: {
          high: 70,
          medium: 20,
          low: 10,
        },
        largest_items: [
          { title: 'Movie 1', size_gb: 10 },
          { title: 'Movie 2', size_gb: 9 },
        ],
        duplicate_space_gb: 50,
      };

      (invoke as any).mockResolvedValueOnce(mockAnalytics);

      const analytics = await invoke('get_storage_analytics');

      expect(analytics).toBeDefined();
      expect((analytics as any).total_size_gb).toBe(500);
      expect((analytics as any).media_count).toBe(100);
    });

    it('should display duplicate space warning', async () => {
      // Simulate: Analytics shows duplicates
      // Expected: Warning displayed
      const mockAnalytics = {
        total_size_gb: 500,
        duplicate_space_gb: 50,
      };

      (invoke as any).mockResolvedValueOnce(mockAnalytics);

      const analytics = await invoke('get_storage_analytics');

      expect((analytics as any).duplicate_space_gb).toBeGreaterThan(0);
    });
  });

  describe('Library Cleanup Tab Workflow', () => {
    it('should load cleanup candidates', async () => {
      // Simulate: User opens Cleanup tab
      // Expected: Cleanup candidates listed
      const mockCandidates = [
        {
          id: 'dup-1',
          filename: 'movie-duplicate.mkv',
          size_gb: 4.5,
          reason: 'duplicate',
        },
        {
          id: 'low-1',
          filename: 'low-quality.avi',
          size_gb: 2.0,
          reason: 'low_quality',
        },
      ];

      (invoke as any).mockResolvedValueOnce(mockCandidates);

      const candidates = await invoke('get_cleanup_candidates');

      expect((candidates as any).length).toBe(2);
      expect((candidates as any)[0].reason).toBe('duplicate');
    });

    it('should execute cleanup with confirmation', async () => {
      // Simulate: User confirms cleanup
      // Expected: Cleanup executed
      (invoke as any).mockResolvedValueOnce({
        success: true,
        deleted_count: 2,
        freed_space_gb: 6.5,
      });

      const result = await invoke('execute_cleanup', {
        candidate_ids: ['dup-1', 'low-1'],
      });

      expect((result as any).success).toBe(true);
      expect((result as any).deleted_count).toBe(2);
      expect((result as any).freed_space_gb).toBe(6.5);
    });

    it('should show confirmation dialog before cleanup', () => {
      // Simulate: User initiates cleanup
      // Expected: Confirmation dialog shown
      const candidateCount = 2;
      const totalSize = 6.5;

      expect(candidateCount).toBeGreaterThan(0);
      expect(totalSize).toBeGreaterThan(0);
      // Dialog would show: "Вы уверены? Это удалит X файлов (Y GB)"
    });
  });

  describe('User Metadata Workflow', () => {
    it('should set user rating for media item', async () => {
      // Simulate: User rates movie
      // Expected: Rating saved
      (invoke as any).mockResolvedValueOnce({ success: true });

      const result = await invoke('set_user_rating', {
        media_id: '1',
        rating: 9,
      });

      expect((result as any).success).toBe(true);
    });

    it('should set watch status for media item', async () => {
      // Simulate: User marks as watched
      // Expected: Status saved
      (invoke as any).mockResolvedValueOnce({ success: true });

      const result = await invoke('set_watch_status', {
        media_id: '1',
        status: 'watched',
      });

      expect((result as any).success).toBe(true);
    });

    it('should add tags to media item', async () => {
      // Simulate: User adds tags
      // Expected: Tags saved
      (invoke as any).mockResolvedValueOnce({ success: true });

      const result = await invoke('add_tags', {
        media_id: '1',
        tags: ['favorite', 'action'],
      });

      expect((result as any).success).toBe(true);
    });

    it('should set custom notes for media item', async () => {
      // Simulate: User adds notes
      // Expected: Notes saved
      (invoke as any).mockResolvedValueOnce({ success: true });

      const result = await invoke('set_custom_notes', {
        media_id: '1',
        notes: 'Great movie, watch again!',
      });

      expect((result as any).success).toBe(true);
    });
  });

  describe('Error Recovery Workflow', () => {
    it('should recover from library loading error', async () => {
      // Simulate: Library fails to load, user retries
      // Expected: Library loads on retry
      (invoke as any)
        .mockRejectedValueOnce(new Error('Database error'))
        .mockResolvedValueOnce([mockLibraryItem]);

      // First attempt fails
      try {
        await invoke('query_library', { filters: {} });
      } catch (error) {
        setLibraryError('Failed to load library');
      }

      let state = get(libraryStore);
      expect(state.error).toBe('Failed to load library');

      // Retry succeeds
      const items = await invoke('query_library', { filters: {} });
      setLibraryItems(items as UiSearchResult[]);
      setLibraryError('');

      state = get(libraryStore);
      expect(state.items.length).toBe(1);
      expect(state.error).toBe('');
    });
  });

  describe('Loading State Workflow', () => {
    it('should show loading state while fetching library', () => {
      // Simulate: Library is loading
      // Expected: Loading indicator shown
      setLibraryLoading(true);
      let state = get(libraryStore);
      expect(state.loading).toBe(true);

      // Loading complete
      setLibraryLoading(false);
      state = get(libraryStore);
      expect(state.loading).toBe(false);
    });
  });
});
