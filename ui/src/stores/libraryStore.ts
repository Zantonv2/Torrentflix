import { writable, derived } from 'svelte/store';

export interface LibraryFilters {
  year?: number;
  resolution?: string;
  minRating?: number;
  watchStatus?: 'not_watched' | 'watching' | 'watched' | 'dropped';
}

export interface Collection {
  id: string;
  name: string;
  description?: string;
  itemCount: number;
  isSmart: boolean;
}

// Generic media item type that works with both backend and UI types
export interface MediaItemBase {
  id: string | number;
  title: string;
  year?: number;
  overview?: string;
  genres?: string[];
  user_rating?: number;
  watch_status?: string;
  [key: string]: unknown;
}

export interface LibraryState {
  items: MediaItemBase[];
  filters: LibraryFilters;
  selectedItem: MediaItemBase | null;
  collections: Collection[];
  loading: boolean;
  error: string | null;
}

const initialState: LibraryState = {
  items: [],
  filters: {},
  selectedItem: null,
  collections: [],
  loading: false,
  error: null,
};

// Create the main library store
export const libraryStore = writable<LibraryState>(initialState);

// Helper functions to update the store
export const setItems = (items: MediaItemBase[]) => {
  libraryStore.update((state) => ({ ...state, items, loading: false, error: null }));
};

export const setFilters = (filters: LibraryFilters) => {
  libraryStore.update((state) => ({ ...state, filters }));
};

export const setSelectedItem = (item: MediaItemBase | null) => {
  libraryStore.update((state) => ({ ...state, selectedItem: item }));
};

export const setCollections = (collections: Collection[]) => {
  libraryStore.update((state) => ({ ...state, collections }));
};

export const addCollection = (collection: Collection) => {
  libraryStore.update((state) => ({
    ...state,
    collections: [...state.collections, collection],
  }));
};

export const setLoading = (loading: boolean) => {
  libraryStore.update((state) => ({ ...state, loading }));
};

export const setError = (error: string | null) => {
  libraryStore.update((state) => ({ ...state, error, loading: false }));
};

export const clearLibrary = () => {
  libraryStore.set(initialState);
};

// Derived store for checking if library has items
export const hasItems = derived(libraryStore, ($libraryStore) => $libraryStore.items.length > 0);

// Derived store for checking if an item is selected
export const itemSelected = derived(
  libraryStore,
  ($libraryStore) => $libraryStore.selectedItem !== null
);

// Derived store for filtered items count
export const filteredItemsCount = derived(
  libraryStore,
  ($libraryStore) => $libraryStore.items.length
);
