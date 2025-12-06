import { writable, derived } from 'svelte/store';
import type { UiSearchResult } from '../types';

export interface SearchState {
  query: string;
  results: UiSearchResult[];
  loading: boolean;
  error: string | null;
}

const initialState: SearchState = {
  query: '',
  results: [],
  loading: false,
  error: null,
};

// Create the main search store
const store = writable<SearchState>(initialState);

// Export the store with helper methods attached
export const searchStore = {
  subscribe: store.subscribe,
  update: store.update,
  set: store.set,
  setQuery: (query: string) => {
    store.update((state) => ({ ...state, query }));
  },
  setResults: (results: UiSearchResult[]) => {
    store.update((state) => ({ ...state, results, loading: false, error: null }));
  },
  setLoading: (loading: boolean) => {
    store.update((state) => ({ ...state, loading }));
  },
  setError: (error: string | null) => {
    store.update((state) => ({ ...state, error, loading: false }));
  },
  clearSearch: () => {
    store.set(initialState);
  },
};

// Helper functions to update the store (for backward compatibility)
export const setQuery = (query: string) => {
  searchStore.setQuery(query);
};

export const setResults = (results: UiSearchResult[]) => {
  searchStore.setResults(results);
};

export const setLoading = (loading: boolean) => {
  searchStore.setLoading(loading);
};

export const setError = (error: string | null) => {
  searchStore.setError(error);
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
