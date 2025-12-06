import { describe, it, expect, beforeEach } from 'vitest';
import {
  searchStore,
  setQuery,
  setResults,
  setLoading,
  setError,
  clearSearch,
  hasResults,
  isIdle,
} from './searchStore';
import type { SearchState } from './searchStore';
import type { UiSearchResult } from '../types';

describe('searchStore', () => {
  beforeEach(() => {
    clearSearch();
  });

  describe('state updates', () => {
    it('should initialize with empty state', () => {
      let state: SearchState | undefined;
      const unsubscribe = searchStore.subscribe((s) => {
        state = s;
      });

      expect(state).toEqual({
        query: '',
        results: [],
        loading: false,
        error: null,
      });

      unsubscribe();
    });

    it('should update query', () => {
      let state: SearchState | undefined;
      const unsubscribe = searchStore.subscribe((s) => {
        state = s;
      });

      setQuery('test query');

      expect(state?.query).toBe('test query');
      unsubscribe();
    });

    it('should set results and clear loading/error', () => {
      let state: SearchState | undefined;
      const unsubscribe = searchStore.subscribe((s) => {
        state = s;
      });

      const mockResults: UiSearchResult[] = [
        {
          id: '1',
          title: 'Test Movie',
          year: 2024,
          quality_badge: '1080p',
          torrent_info: {
            seeders: 100,
            size_gb: 2.5,
            magnet_link: 'magnet:?xt=urn:btih:test',
          },
        },
      ];

      setResults(mockResults);

      expect(state?.results).toEqual(mockResults);
      expect(state?.loading).toBe(false);
      expect(state?.error).toBeNull();
      unsubscribe();
    });

    it('should set loading state', () => {
      let state: SearchState | undefined;
      const unsubscribe = searchStore.subscribe((s) => {
        state = s;
      });

      setLoading(true);
      expect(state?.loading).toBe(true);

      setLoading(false);
      expect(state?.loading).toBe(false);

      unsubscribe();
    });

    it('should set error and clear loading', () => {
      let state: SearchState | undefined;
      const unsubscribe = searchStore.subscribe((s) => {
        state = s;
      });

      setError('Test error');

      expect(state?.error).toBe('Test error');
      expect(state?.loading).toBe(false);
      unsubscribe();
    });

    it('should clear search to initial state', () => {
      let state: SearchState | undefined;
      const unsubscribe = searchStore.subscribe((s) => {
        state = s;
      });

      setQuery('test');
      setLoading(true);
      clearSearch();

      expect(state).toEqual({
        query: '',
        results: [],
        loading: false,
        error: null,
      });

      unsubscribe();
    });
  });

  describe('subscriptions', () => {
    it('should notify subscribers on state change', () => {
      const updates: string[] = [];

      const unsubscribe = searchStore.subscribe((state) => {
        updates.push(state.query);
      });

      setQuery('first');
      setQuery('second');
      setQuery('third');

      expect(updates).toContain('');
      expect(updates).toContain('first');
      expect(updates).toContain('second');
      expect(updates).toContain('third');

      unsubscribe();
    });

    it('should allow multiple subscribers', () => {
      const updates1: string[] = [];
      const updates2: string[] = [];

      const unsub1 = searchStore.subscribe((state) => {
        updates1.push(state.query);
      });

      const unsub2 = searchStore.subscribe((state) => {
        updates2.push(state.query);
      });

      setQuery('test');

      expect(updates1).toContain('test');
      expect(updates2).toContain('test');

      unsub1();
      unsub2();
    });

    it('should stop notifying after unsubscribe', () => {
      const updates: string[] = [];

      const unsubscribe = searchStore.subscribe((state) => {
        updates.push(state.query);
      });

      setQuery('first');
      unsubscribe();
      setQuery('second');

      expect(updates).toContain('first');
      expect(updates).not.toContain('second');
    });
  });

  describe('derived stores', () => {
    it('hasResults should be true when results exist', () => {
      let hasResultsValue = false;
      const unsubscribe = hasResults.subscribe((value) => {
        hasResultsValue = value;
      });

      expect(hasResultsValue).toBe(false);

      const mockResults: UiSearchResult[] = [
        {
          id: '1',
          title: 'Test Movie',
          quality_badge: '1080p',
          torrent_info: {
            seeders: 100,
            size_gb: 2.5,
            magnet_link: 'magnet:?xt=urn:btih:test',
          },
        },
      ];

      setResults(mockResults);
      expect(hasResultsValue).toBe(true);

      clearSearch();
      expect(hasResultsValue).toBe(false);

      unsubscribe();
    });

    it('isIdle should be true when not loading and no error', () => {
      let isIdleValue = false;
      const unsubscribe = isIdle.subscribe((value) => {
        isIdleValue = value;
      });

      expect(isIdleValue).toBe(true);

      setLoading(true);
      expect(isIdleValue).toBe(false);

      setLoading(false);
      expect(isIdleValue).toBe(true);

      setError('Test error');
      expect(isIdleValue).toBe(false);

      setError(null);
      expect(isIdleValue).toBe(true);

      unsubscribe();
    });
  });
});
