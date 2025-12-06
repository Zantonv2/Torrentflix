import { describe, it, expect, beforeEach } from 'vitest';
import {
  libraryStore,
  setItems,
  setFilters,
  setSelectedItem,
  setCollections,
  addCollection,
  setLoading,
  setError,
  clearLibrary,
  hasItems,
  itemSelected,
  filteredItemsCount,
} from './libraryStore';
import type { Collection, LibraryState } from './libraryStore';
import type { UiSearchResult } from '../types';

const mockMovie: UiSearchResult = {
  id: '1',
  title: 'Test Movie',
  year: 2024,
  quality_badge: '1080p',
  torrent_info: {
    seeders: 100,
    size_gb: 2.5,
    magnet_link: 'magnet:?xt=urn:btih:test',
  },
};

describe('libraryStore', () => {
  beforeEach(() => {
    clearLibrary();
  });

  describe('state updates', () => {
    it('should initialize with empty state', () => {
      let state: LibraryState | undefined;
      const unsubscribe = libraryStore.subscribe((s) => {
        state = s;
      });

      expect(state).toEqual({
        items: [],
        filters: {},
        selectedItem: null,
        collections: [],
        loading: false,
        error: null,
      });

      unsubscribe();
    });

    it('should set items and clear loading/error', () => {
      let state: LibraryState | undefined;
      const unsubscribe = libraryStore.subscribe((s) => {
        state = s;
      });

      const items = [mockMovie];
      setItems(items);

      expect(state?.items).toEqual(items);
      expect(state?.loading).toBe(false);
      expect(state?.error).toBeNull();
      unsubscribe();
    });

    it('should update filters', () => {
      let state: LibraryState | undefined;
      const unsubscribe = libraryStore.subscribe((s) => {
        state = s;
      });

      const filters = { year: 2024, resolution: '1080p' };
      setFilters(filters);

      expect(state?.filters).toEqual(filters);
      unsubscribe();
    });

    it('should set selected item', () => {
      let state: LibraryState | undefined;
      const unsubscribe = libraryStore.subscribe((s) => {
        state = s;
      });

      setSelectedItem(mockMovie);
      expect(state?.selectedItem).toEqual(mockMovie);

      setSelectedItem(null);
      expect(state?.selectedItem).toBeNull();

      unsubscribe();
    });

    it('should set collections', () => {
      let state: LibraryState | undefined;
      const unsubscribe = libraryStore.subscribe((s) => {
        state = s;
      });

      const collections: Collection[] = [
        {
          id: '1',
          name: 'Favorites',
          itemCount: 5,
          isSmart: false,
        },
      ];

      setCollections(collections);
      expect(state?.collections).toEqual(collections);
      unsubscribe();
    });

    it('should add collection to existing collections', () => {
      let state: LibraryState | undefined;
      const unsubscribe = libraryStore.subscribe((s) => {
        state = s;
      });

      const collection1: Collection = {
        id: '1',
        name: 'Favorites',
        itemCount: 5,
        isSmart: false,
      };

      const collection2: Collection = {
        id: '2',
        name: 'Watched',
        itemCount: 10,
        isSmart: false,
      };

      setCollections([collection1]);
      addCollection(collection2);

      expect(state?.collections).toHaveLength(2);
      expect(state?.collections).toContainEqual(collection1);
      expect(state?.collections).toContainEqual(collection2);

      unsubscribe();
    });

    it('should set loading state', () => {
      let state: LibraryState | undefined;
      const unsubscribe = libraryStore.subscribe((s) => {
        state = s;
      });

      setLoading(true);
      expect(state?.loading).toBe(true);

      setLoading(false);
      expect(state?.loading).toBe(false);

      unsubscribe();
    });

    it('should set error and clear loading', () => {
      let state: LibraryState | undefined;
      const unsubscribe = libraryStore.subscribe((s) => {
        state = s;
      });

      setError('Test error');

      expect(state?.error).toBe('Test error');
      expect(state?.loading).toBe(false);
      unsubscribe();
    });

    it('should clear library to initial state', () => {
      let state: LibraryState | undefined;
      const unsubscribe = libraryStore.subscribe((s) => {
        state = s;
      });

      setItems([mockMovie]);
      setFilters({ year: 2024 });
      setSelectedItem(mockMovie);
      clearLibrary();

      expect(state).toEqual({
        items: [],
        filters: {},
        selectedItem: null,
        collections: [],
        loading: false,
        error: null,
      });

      unsubscribe();
    });
  });

  describe('filters and selections', () => {
    it('should apply multiple filters', () => {
      let state: LibraryState | undefined;
      const unsubscribe = libraryStore.subscribe((s) => {
        state = s;
      });

      const filters = {
        year: 2024,
        resolution: '1080p',
        minRating: 7.5,
        watchStatus: 'watched' as const,
      };

      setFilters(filters);

      expect(state?.filters).toEqual(filters);
      expect(state?.filters.year).toBe(2024);
      expect(state?.filters.resolution).toBe('1080p');
      expect(state?.filters.minRating).toBe(7.5);
      expect(state?.filters.watchStatus).toBe('watched');

      unsubscribe();
    });

    it('should update filters without affecting items', () => {
      let state: LibraryState | undefined;
      const unsubscribe = libraryStore.subscribe((s) => {
        state = s;
      });

      const items = [mockMovie];
      setItems(items);
      setFilters({ year: 2024 });

      expect(state?.items).toEqual(items);
      expect(state?.filters.year).toBe(2024);

      unsubscribe();
    });

    it('should track selected item independently', () => {
      let state: LibraryState | undefined;
      const unsubscribe = libraryStore.subscribe((s) => {
        state = s;
      });

      const items = [mockMovie];
      setItems(items);
      setSelectedItem(mockMovie);

      expect(state?.items).toEqual(items);
      expect(state?.selectedItem).toEqual(mockMovie);

      setSelectedItem(null);
      expect(state?.items).toEqual(items);
      expect(state?.selectedItem).toBeNull();

      unsubscribe();
    });
  });

  describe('derived stores', () => {
    it('hasItems should be true when items exist', () => {
      let hasItemsValue = false;
      const unsubscribe = hasItems.subscribe((value) => {
        hasItemsValue = value;
      });

      expect(hasItemsValue).toBe(false);

      setItems([mockMovie]);
      expect(hasItemsValue).toBe(true);

      clearLibrary();
      expect(hasItemsValue).toBe(false);

      unsubscribe();
    });

    it('itemSelected should be true when item is selected', () => {
      let itemSelectedValue = false;
      const unsubscribe = itemSelected.subscribe((value) => {
        itemSelectedValue = value;
      });

      expect(itemSelectedValue).toBe(false);

      setSelectedItem(mockMovie);
      expect(itemSelectedValue).toBe(true);

      setSelectedItem(null);
      expect(itemSelectedValue).toBe(false);

      unsubscribe();
    });

    it('filteredItemsCount should match items length', () => {
      let countValue = 0;
      const unsubscribe = filteredItemsCount.subscribe((value) => {
        countValue = value;
      });

      expect(countValue).toBe(0);

      setItems([mockMovie]);
      expect(countValue).toBe(1);

      setItems([mockMovie, mockMovie]);
      expect(countValue).toBe(2);

      clearLibrary();
      expect(countValue).toBe(0);

      unsubscribe();
    });
  });
});
