import { writable, derived } from 'svelte/store';
import type { UiSearchResult } from '../types';

export interface SearchState {
  query: string;
  results: UiSearchResult[];
  loading: boolean;
  error: string | null;
  // Caching
  cache: Map<string, { results: UiSearchResult[]; timestamp: number }>;
  cacheTimestamp: number;
  scrollPosition: number;
  // Search history
  history: string[];
  // Autocomplete suggestions
  suggestions: string[];
}

const initialState: SearchState = {
  query: '',
  results: [],
  loading: false,
  error: null,
  cache: new Map(),
  cacheTimestamp: 0,
  scrollPosition: 0,
  history: [],
  suggestions: [],
};

// Cache invalidation time: 10 minutes
const CACHE_INVALIDATION_TIME = 10 * 60 * 1000;
// Max search history items
const MAX_HISTORY_ITEMS = 10;
// LocalStorage key for search history
const SEARCH_HISTORY_KEY = 'torrentflix_search_history';

// Load search history from localStorage
function loadSearchHistory(): string[] {
  if (typeof window === 'undefined') return [];
  try {
    const stored = localStorage.getItem(SEARCH_HISTORY_KEY);
    return stored ? JSON.parse(stored) : [];
  } catch {
    return [];
  }
}

// Save search history to localStorage
function saveSearchHistory(history: string[]): void {
  if (typeof window === 'undefined') return;
  try {
    localStorage.setItem(SEARCH_HISTORY_KEY, JSON.stringify(history));
  } catch {
    // Silently fail if localStorage is unavailable
  }
}

// Create the main search store
const store = writable<SearchState>({
  ...initialState,
  history: loadSearchHistory(),
});

// Export the store with helper methods attached
export const searchStore = {
  subscribe: store.subscribe,
  update: store.update,
  set: store.set,
  setQuery: (query: string) => {
    store.update((state) => ({ ...state, query }));
  },
  setResults: (results: UiSearchResult[], query?: string) => {
    store.update((state) => {
      const cacheKey = query || state.query;
      const newCache = new Map(state.cache);
      newCache.set(cacheKey, { results, timestamp: Date.now() });
      return {
        ...state,
        results,
        loading: false,
        error: null,
        cache: newCache,
        cacheTimestamp: Date.now(),
      };
    });
  },
  setLoading: (loading: boolean) => {
    store.update((state) => ({ ...state, loading }));
  },
  setError: (error: string | null) => {
    store.update((state) => ({ ...state, error, loading: false }));
  },
  getCachedResults: (query: string): UiSearchResult[] | null => {
    let cachedResults: UiSearchResult[] | null = null;
    store.update((state) => {
      const cached = state.cache.get(query);
      if (cached && Date.now() - cached.timestamp < CACHE_INVALIDATION_TIME) {
        cachedResults = cached.results;
      }
      return state;
    });
    return cachedResults;
  },
  setScrollPosition: (position: number) => {
    store.update((state) => ({ ...state, scrollPosition: position }));
  },
  clearCache: () => {
    store.update((state) => ({
      ...state,
      cache: new Map(),
      cacheTimestamp: 0,
    }));
  },
  addToHistory: (query: string) => {
    store.update((state) => {
      const trimmedQuery = query.trim();
      if (!trimmedQuery) return state;

      // Remove if already exists, then add to front
      const newHistory = state.history.filter((item) => item !== trimmedQuery);
      newHistory.unshift(trimmedQuery);

      // Keep only MAX_HISTORY_ITEMS
      const limitedHistory = newHistory.slice(0, MAX_HISTORY_ITEMS);
      saveSearchHistory(limitedHistory);

      return {
        ...state,
        history: limitedHistory,
      };
    });
  },
  setSuggestions: (suggestions: string[]) => {
    store.update((state) => ({ ...state, suggestions }));
  },
  clearHistory: () => {
    store.update((state) => {
      saveSearchHistory([]);
      return {
        ...state,
        history: [],
      };
    });
  },
  clearSearch: () => {
    store.set({
      ...initialState,
      history: loadSearchHistory(),
    });
  },
};

// Helper functions to update the store (for backward compatibility)
export const setQuery = (query: string) => {
  searchStore.setQuery(query);
};

export const setResults = (results: UiSearchResult[], query?: string) => {
  searchStore.setResults(results, query);
};

export const setLoading = (loading: boolean) => {
  searchStore.setLoading(loading);
};

export const setError = (error: string | null) => {
  searchStore.setError(error);
};

export const getCachedResults = (query: string): UiSearchResult[] | null => {
  return searchStore.getCachedResults(query);
};

export const setScrollPosition = (position: number) => {
  searchStore.setScrollPosition(position);
};

export const clearCache = () => {
  searchStore.clearCache();
};

export const addToHistory = (query: string) => {
  searchStore.addToHistory(query);
};

export const setSuggestions = (suggestions: string[]) => {
  searchStore.setSuggestions(suggestions);
};

export const clearHistory = () => {
  searchStore.clearHistory();
};

export const clearSearch = () => {
  searchStore.clearSearch();
};

// Derived store for checking if search has results
export const hasResults = derived(searchStore, ($searchStore) => $searchStore.results.length > 0);

// Derived store for checking if search is idle (not loading and no error)
export const isIdle = derived(
  searchStore,
  ($searchStore) => !$searchStore.loading && !$searchStore.error
);

// Derived store for search history
export const searchHistory = derived(searchStore, ($searchStore) => $searchStore.history);

// Derived store for suggestions
export const searchSuggestions = derived(searchStore, ($searchStore) => $searchStore.suggestions);
