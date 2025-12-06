import { writable, derived } from 'svelte/store';

export type Page = 'discover' | 'library' | 'downloads' | 'settings';

export interface Modal {
  isOpen: boolean;
  type?: string; // 'details', 'confirmation', 'error', etc.
  title?: string;
  content?: string;
  data?: Record<string, unknown>; // Additional data for the modal
}

export interface Toast {
  id: string;
  message: string;
  type: 'success' | 'error' | 'info' | 'warning';
  duration?: number; // milliseconds, null for no auto-dismiss
}

export interface UiState {
  currentPage: Page;
  modal: Modal;
  toasts: Toast[];
  sidebarCollapsed: boolean;
}

const initialState: UiState = {
  currentPage: 'discover',
  modal: {
    isOpen: false,
  },
  toasts: [],
  sidebarCollapsed: false,
};

// Create the main UI store
export const uiStore = writable<UiState>(initialState);

// Helper functions to update the store
export const setCurrentPage = (page: Page) => {
  uiStore.update((state) => ({ ...state, currentPage: page }));
};

export const openModal = (
  type: string,
  title?: string,
  content?: string,
  data?: Record<string, unknown>
) => {
  uiStore.update((state) => ({
    ...state,
    modal: {
      isOpen: true,
      type,
      title,
      content,
      data,
    },
  }));
};

export const closeModal = () => {
  uiStore.update((state) => ({
    ...state,
    modal: {
      isOpen: false,
    },
  }));
};

export const addToast = (message: string, type: 'success' | 'error' | 'info' | 'warning', duration?: number) => {
  const id = `toast-${Date.now()}-${Math.random()}`;
  const toast: Toast = { id, message, type, duration };

  uiStore.update((state) => ({
    ...state,
    toasts: [...state.toasts, toast],
  }));

  // Auto-dismiss if duration is specified
  if (duration) {
    setTimeout(() => {
      removeToast(id);
    }, duration);
  }

  return id;
};

export const removeToast = (id: string) => {
  uiStore.update((state) => ({
    ...state,
    toasts: state.toasts.filter((t) => t.id !== id),
  }));
};

export const clearToasts = () => {
  uiStore.update((state) => ({
    ...state,
    toasts: [],
  }));
};

export const toggleSidebar = () => {
  uiStore.update((state) => ({
    ...state,
    sidebarCollapsed: !state.sidebarCollapsed,
  }));
};

export const setSidebarCollapsed = (collapsed: boolean) => {
  uiStore.update((state) => ({
    ...state,
    sidebarCollapsed: collapsed,
  }));
};

export const resetUi = () => {
  uiStore.set(initialState);
};

// Derived store for checking if modal is open
export const isModalOpen = derived(uiStore, ($uiStore) => $uiStore.modal.isOpen);

// Derived store for checking if there are toasts
export const hasToasts = derived(uiStore, ($uiStore) => $uiStore.toasts.length > 0);

// Derived store for toast count
export const toastCount = derived(uiStore, ($uiStore) => $uiStore.toasts.length);
