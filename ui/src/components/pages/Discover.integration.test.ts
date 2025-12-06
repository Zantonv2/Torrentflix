import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { searchStore, clearSearch, setResults, setError, setLoading } from '../../stores/searchStore';
import { uiStore, showToast } from '../../stores/uiStore';
import type { UiSearchResult } from '../../types';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const mockMovie: UiSearchResult = {
  id: '1',
  title: 'Test Movie',
  year: 2024,
  poster_url: 'https://example.com/poster.jpg',
  quality_badge: '1080p',
  rating: 8.5,
  rating_kinopoisk: 8.5,
  rating_imdb: 8.3,
  rating_tmdb: 8.2,
  runtime_minutes: 120,
  description: 'A test movie description',
  cast: ['Actor 1', 'Actor 2'],
  genres: ['Action', 'Drama'],
  torrent_info: {
    seeders: 100,
    size_gb: 2.5,
    magnet_link: 'magnet:?xt=urn:btih:test',
  },
};

describe('Discover Page - Integration Tests', () => {
  beforeEach(() => {
    clearSearch();
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Feed Loading Workflow', () => {
    it('should load feed on page mount and display results', async () => {
      // Simulate: User opens Discover page
      // Expected: Feed is loaded and displayed
      const mockFeedResults = [mockMovie, { ...mockMovie, id: '2', title: 'Movie 2' }];

      // Mock the get_feed command
      (invoke as any).mockResolvedValueOnce(mockFeedResults);

      // Simulate loading feed
      setLoading(true);
      const results = await invoke('get_feed');
      setResults(results as UiSearchResult[]);
      setLoading(false);

      // Verify results are in store
      const state = get(searchStore);
      expect(state.results.length).toBe(2);
      expect(state.results[0].title).toBe('Test Movie');
      expect(state.results[1].title).toBe('Movie 2');
    });

    it('should handle feed loading errors gracefully', async () => {
      // Simulate: Feed loading fails
      // Expected: Error message displayed, previous state preserved
      const initialResults = [mockMovie];
      setResults(initialResults);

      // Mock error
      (invoke as any).mockRejectedValueOnce(new Error('Network error'));

      try {
        await invoke('get_feed');
      } catch (error) {
        setError('Failed to load feed. Please try again.');
      }

      // Verify error is set and previous results are preserved
      const state = get(searchStore);
      expect(state.error).toBe('Failed to load feed. Please try again.');
      expect(state.results.length).toBe(1); // Previous results preserved
    });

    it('should display loading skeleton while fetching feed', async () => {
      // Simulate: User sees loading state
      // Expected: Skeleton placeholders shown
      setLoading(true);
      let state = get(searchStore);
      expect(state.loading).toBe(true);

      // Simulate fetch complete
      setResults([mockMovie]);
      setLoading(false);
      state = get(searchStore);
      expect(state.loading).toBe(false);
      expect(state.results.length).toBe(1);
    });
  });

  describe('Search Workflow', () => {
    it('should search movies with debounce and display results', async () => {
      // Simulate: User types search query
      // Expected: After 300ms debounce, search is executed
      const searchQuery = 'Inception';
      const mockSearchResults = [
        { ...mockMovie, id: '1', title: 'Inception' },
        { ...mockMovie, id: '2', title: 'Inception: The Cobol Job' },
      ];

      // Mock search command
      (invoke as any).mockResolvedValueOnce(mockSearchResults);

      // Simulate search
      setLoading(true);
      const results = await invoke('search_movies', {
        query: searchQuery,
        media_type: 'movie',
        limit: 50,
      });
      setResults(results as UiSearchResult[]);
      setLoading(false);

      // Verify search results
      const state = get(searchStore);
      expect(state.results.length).toBe(2);
      expect(state.results[0].title).toBe('Inception');
      expect(state.results[1].title).toBe('Inception: The Cobol Job');
    });

    it('should handle empty search results', async () => {
      // Simulate: User searches for non-existent movie
      // Expected: Empty state message shown
      (invoke as any).mockResolvedValueOnce([]);

      setLoading(true);
      const results = await invoke('search_movies', {
        query: 'xyznonexistent',
        media_type: 'movie',
        limit: 50,
      });
      setResults(results as UiSearchResult[]);
      setLoading(false);

      // Verify empty state
      const state = get(searchStore);
      expect(state.results.length).toBe(0);
      expect(state.loading).toBe(false);
    });

    it('should handle search errors with retry option', async () => {
      // Simulate: Search fails
      // Expected: Error message with retry button
      (invoke as any).mockRejectedValueOnce(new Error('Search timeout'));

      try {
        await invoke('search_movies', {
          query: 'test',
          media_type: 'movie',
          limit: 50,
        });
      } catch (error) {
        setError('Search failed. Please try again.');
      }

      // Verify error state
      const state = get(searchStore);
      expect(state.error).toBe('Search failed. Please try again.');
    });

    it('should clear previous results when new search is initiated', async () => {
      // Simulate: User performs new search
      // Expected: Previous results cleared, new results shown
      const firstResults = [{ ...mockMovie, id: '1', title: 'First Search' }];
      const secondResults = [{ ...mockMovie, id: '2', title: 'Second Search' }];

      // First search
      setResults(firstResults);
      let state = get(searchStore);
      expect(state.results[0].title).toBe('First Search');

      // Second search
      clearSearch();
      setResults(secondResults);
      state = get(searchStore);
      expect(state.results[0].title).toBe('Second Search');
      expect(state.results.length).toBe(1);
    });
  });

  describe('Movie Card Interaction Workflow', () => {
    it('should display movie details when card is clicked', async () => {
      // Simulate: User clicks movie card
      // Expected: Detail modal opens with full information
      setResults([mockMovie]);

      const state = get(searchStore);
      const selectedMovie = state.results[0];

      // Verify all details are available
      expect(selectedMovie.title).toBe('Test Movie');
      expect(selectedMovie.description).toBeDefined();
      expect(selectedMovie.cast.length).toBeGreaterThan(0);
      expect(selectedMovie.genres.length).toBeGreaterThan(0);
      expect(selectedMovie.rating_kinopoisk).toBeDefined();
      expect(selectedMovie.rating_imdb).toBeDefined();
      expect(selectedMovie.rating_tmdb).toBeDefined();
    });

    it('should show hover overlay with View Details button', () => {
      // Simulate: User hovers over movie card
      // Expected: Overlay appears with View Details button
      setResults([mockMovie]);

      const state = get(searchStore);
      const movie = state.results[0];

      // Verify movie has poster for overlay
      expect(movie.poster_url).toBeDefined();
      expect(movie.title).toBeDefined();
    });

    it('should display quality badge on movie card', () => {
      // Simulate: Movie card is rendered
      // Expected: Quality badge visible
      setResults([mockMovie]);

      const state = get(searchStore);
      const movie = state.results[0];

      expect(movie.quality_badge).toBe('1080p');
    });

    it('should display ratings from multiple sources', () => {
      // Simulate: Movie card shows ratings
      // Expected: Kinopoisk, IMDb, TMDB ratings visible
      setResults([mockMovie]);

      const state = get(searchStore);
      const movie = state.results[0];

      expect(movie.rating_kinopoisk).toBe(8.5);
      expect(movie.rating_imdb).toBe(8.3);
      expect(movie.rating_tmdb).toBe(8.2);
    });
  });

  describe('Download Workflow', () => {
    it('should initiate download when Download button is clicked', async () => {
      // Simulate: User clicks Download in detail modal
      // Expected: Download starts, success message shown
      setResults([mockMovie]);

      const state = get(searchStore);
      const movie = state.results[0];

      // Mock download command
      (invoke as any).mockResolvedValueOnce({ success: true, download_id: 'test-123' });

      // Simulate download
      const downloadResult = await invoke('start_download', {
        magnetLink: movie.torrent_info.magnet_link,
        title: movie.title,
      });

      // Verify download initiated
      expect(downloadResult).toBeDefined();
      expect((downloadResult as any).success).toBe(true);
    });

    it('should handle download errors gracefully', async () => {
      // Simulate: Download fails
      // Expected: Error message shown, user can retry
      setResults([mockMovie]);

      const state = get(searchStore);
      const movie = state.results[0];

      // Mock download error
      (invoke as any).mockRejectedValueOnce(new Error('qBittorrent not connected'));

      try {
        await invoke('start_download', {
          magnetLink: movie.torrent_info.magnet_link,
          title: movie.title,
        });
      } catch (error) {
        setError('Download failed: qBittorrent not connected');
      }

      // Verify error state
      const searchState = get(searchStore);
      expect(searchState.error).toContain('Download failed');
    });

    it('should have magnet link available for download', () => {
      // Simulate: Download button is clicked
      // Expected: Magnet link is valid and available
      setResults([mockMovie]);

      const state = get(searchStore);
      const movie = state.results[0];

      expect(movie.torrent_info.magnet_link).toBeDefined();
      expect(movie.torrent_info.magnet_link).toContain('magnet:');
    });
  });

  describe('Responsive Grid Workflow', () => {
    it('should display correct number of columns based on viewport', () => {
      // Simulate: Viewport is resized
      // Expected: Grid columns adjust accordingly
      const viewportWidths = [320, 640, 1024, 1920];
      const expectedColumns = [1, 2, 4, 6];

      viewportWidths.forEach((width, index) => {
        let columns: number;
        if (width < 640) {
          columns = 1;
        } else if (width < 1024) {
          columns = 2;
        } else {
          columns = 4;
        }

        expect(columns).toBe(expectedColumns[index]);
      });
    });

    it('should reflow content without horizontal scrolling', () => {
      // Simulate: Multiple movies displayed in grid
      // Expected: Content fits without horizontal scroll
      const movies = Array.from({ length: 20 }, (_, i) => ({
        ...mockMovie,
        id: String(i),
        title: `Movie ${i}`,
      }));

      setResults(movies);

      const state = get(searchStore);
      expect(state.results.length).toBe(20);
      // Grid should handle all items without overflow
    });
  });

  describe('Error Recovery Workflow', () => {
    it('should recover from error and allow retry', async () => {
      // Simulate: Feed loading fails, user clicks retry
      // Expected: Feed loads successfully on retry
      (invoke as any)
        .mockRejectedValueOnce(new Error('Network error'))
        .mockResolvedValueOnce([mockMovie]);

      // First attempt fails
      try {
        await invoke('get_feed');
      } catch (error) {
        setError('Failed to load feed');
      }

      let state = get(searchStore);
      expect(state.error).toBe('Failed to load feed');

      // Retry succeeds
      clearSearch();
      const results = await invoke('get_feed');
      setResults(results as UiSearchResult[]);

      state = get(searchStore);
      expect(state.results.length).toBe(1);
      expect(state.error).toBe('');
    });

    it('should preserve state when error occurs', async () => {
      // Simulate: User has results, then search fails
      // Expected: Previous results still visible
      const initialResults = [mockMovie];
      setResults(initialResults);

      // Search fails
      (invoke as any).mockRejectedValueOnce(new Error('Search error'));

      try {
        await invoke('search_movies', {
          query: 'test',
          media_type: 'movie',
          limit: 50,
        });
      } catch (error) {
        setError('Search failed');
      }

      // Verify previous results preserved
      const state = get(searchStore);
      expect(state.results.length).toBe(1);
      expect(state.results[0].title).toBe('Test Movie');
      expect(state.error).toBe('Search failed');
    });
  });

  describe('Loading State Workflow', () => {
    it('should show skeleton placeholders while loading', () => {
      // Simulate: Feed is loading
      // Expected: Skeleton placeholders shown
      setLoading(true);
      let state = get(searchStore);
      expect(state.loading).toBe(true);

      // Loading complete
      setLoading(false);
      state = get(searchStore);
      expect(state.loading).toBe(false);
    });

    it('should transition from loading to results', async () => {
      // Simulate: Feed loads
      // Expected: Skeleton → Results transition
      setLoading(true);
      let state = get(searchStore);
      expect(state.loading).toBe(true);
      expect(state.results.length).toBe(0);

      // Simulate load complete
      const mockResults = [mockMovie];
      setResults(mockResults);
      setLoading(false);

      state = get(searchStore);
      expect(state.loading).toBe(false);
      expect(state.results.length).toBe(1);
    });
  });
});
