import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  uiStore,
  setCurrentPage,
  openModal,
  closeModal,
  addToast,
  removeToast,
  clearToasts,
  toggleSidebar,
  setSidebarCollapsed,
  resetUi,
  isModalOpen,
  hasToasts,
  toastCount,
} from './uiStore';
import type { Page, UiState } from './uiStore';

describe('uiStore', () => {
  beforeEach(() => {
    resetUi();
    vi.clearAllTimers();
  });

  describe('page routing', () => {
    it('should initialize with discover page', () => {
      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      expect(state?.currentPage).toBe('discover');
      unsubscribe();
    });

    it('should set current page', () => {
      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      const pages: Page[] = ['discover', 'library', 'downloads', 'settings'];

      pages.forEach((page) => {
        setCurrentPage(page);
        expect(state?.currentPage).toBe(page);
      });

      unsubscribe();
    });

    it('should navigate between pages', () => {
      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      setCurrentPage('discover');
      expect(state?.currentPage).toBe('discover');

      setCurrentPage('library');
      expect(state?.currentPage).toBe('library');

      setCurrentPage('downloads');
      expect(state?.currentPage).toBe('downloads');

      setCurrentPage('settings');
      expect(state?.currentPage).toBe('settings');

      unsubscribe();
    });
  });

  describe('modal management', () => {
    it('should initialize with closed modal', () => {
      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      expect(state?.modal.isOpen).toBe(false);
      unsubscribe();
    });

    it('should open modal with details', () => {
      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      openModal('details', 'Movie Details', 'This is a movie', { movieId: '123' });

      expect(state?.modal.isOpen).toBe(true);
      expect(state?.modal.type).toBe('details');
      expect(state?.modal.title).toBe('Movie Details');
      expect(state?.modal.content).toBe('This is a movie');
      expect(state?.modal.data).toEqual({ movieId: '123' });

      unsubscribe();
    });

    it('should close modal', () => {
      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      openModal('details', 'Test');
      expect(state?.modal.isOpen).toBe(true);

      closeModal();
      expect(state?.modal.isOpen).toBe(false);

      unsubscribe();
    });

    it('should replace modal when opening new one', () => {
      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      openModal('details', 'First Modal');
      expect(state?.modal.title).toBe('First Modal');

      openModal('confirmation', 'Second Modal');
      expect(state?.modal.title).toBe('Second Modal');
      expect(state?.modal.type).toBe('confirmation');

      unsubscribe();
    });
  });

  describe('toast notifications', () => {
    it('should add toast notification', () => {
      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      const id = addToast('Success!', 'success');

      expect(state?.toasts).toHaveLength(1);
      expect(state?.toasts[0].message).toBe('Success!');
      expect(state?.toasts[0].type).toBe('success');
      expect(state?.toasts[0].id).toBe(id);

      unsubscribe();
    });

    it('should add multiple toasts', () => {
      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      addToast('First', 'success');
      addToast('Second', 'error');
      addToast('Third', 'info');

      expect(state?.toasts).toHaveLength(3);
      expect(state?.toasts[0].message).toBe('First');
      expect(state?.toasts[1].message).toBe('Second');
      expect(state?.toasts[2].message).toBe('Third');

      unsubscribe();
    });

    it('should remove specific toast', () => {
      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      const id1 = addToast('First', 'success');
      const id2 = addToast('Second', 'error');

      removeToast(id1);

      expect(state?.toasts).toHaveLength(1);
      expect(state?.toasts[0].id).toBe(id2);

      unsubscribe();
    });

    it('should clear all toasts', () => {
      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      addToast('First', 'success');
      addToast('Second', 'error');
      addToast('Third', 'info');

      clearToasts();

      expect(state?.toasts).toHaveLength(0);

      unsubscribe();
    });

    it('should auto-dismiss toast after duration', () => {
      vi.useFakeTimers();

      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      addToast('Auto-dismiss', 'success', 3000);

      expect(state?.toasts).toHaveLength(1);

      vi.advanceTimersByTime(3000);

      expect(state?.toasts).toHaveLength(0);

      vi.useRealTimers();
      unsubscribe();
    });

    it('should not auto-dismiss toast without duration', () => {
      vi.useFakeTimers();

      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      addToast('No auto-dismiss', 'info');

      vi.advanceTimersByTime(5000);

      expect(state?.toasts).toHaveLength(1);

      vi.useRealTimers();
      unsubscribe();
    });
  });

  describe('sidebar management', () => {
    it('should initialize with expanded sidebar', () => {
      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      expect(state?.sidebarCollapsed).toBe(false);
      unsubscribe();
    });

    it('should toggle sidebar', () => {
      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      expect(state?.sidebarCollapsed).toBe(false);

      toggleSidebar();
      expect(state?.sidebarCollapsed).toBe(true);

      toggleSidebar();
      expect(state?.sidebarCollapsed).toBe(false);

      unsubscribe();
    });

    it('should set sidebar collapsed state', () => {
      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      setSidebarCollapsed(true);
      expect(state?.sidebarCollapsed).toBe(true);

      setSidebarCollapsed(false);
      expect(state?.sidebarCollapsed).toBe(false);

      unsubscribe();
    });
  });

  describe('derived stores', () => {
    it('isModalOpen should reflect modal state', () => {
      let isOpenValue = false;
      const unsubscribe = isModalOpen.subscribe((value) => {
        isOpenValue = value;
      });

      expect(isOpenValue).toBe(false);

      openModal('details', 'Test');
      expect(isOpenValue).toBe(true);

      closeModal();
      expect(isOpenValue).toBe(false);

      unsubscribe();
    });

    it('hasToasts should be true when toasts exist', () => {
      let hasToastsValue = false;
      const unsubscribe = hasToasts.subscribe((value) => {
        hasToastsValue = value;
      });

      expect(hasToastsValue).toBe(false);

      addToast('Test', 'success');
      expect(hasToastsValue).toBe(true);

      clearToasts();
      expect(hasToastsValue).toBe(false);

      unsubscribe();
    });

    it('toastCount should track number of toasts', () => {
      let countValue = 0;
      const unsubscribe = toastCount.subscribe((value) => {
        countValue = value;
      });

      expect(countValue).toBe(0);

      const id1 = addToast('First', 'success');
      expect(countValue).toBe(1);

      addToast('Second', 'error');
      expect(countValue).toBe(2);

      removeToast(id1);
      expect(countValue).toBe(1);

      clearToasts();
      expect(countValue).toBe(0);

      unsubscribe();
    });
  });

  describe('state reset', () => {
    it('should reset UI to initial state', () => {
      let state: UiState | undefined;
      const unsubscribe = uiStore.subscribe((s) => {
        state = s;
      });

      setCurrentPage('settings');
      openModal('details', 'Test');
      addToast('Test', 'success');
      setSidebarCollapsed(true);

      resetUi();

      expect(state?.currentPage).toBe('discover');
      expect(state?.modal.isOpen).toBe(false);
      expect(state?.toasts).toHaveLength(0);
      expect(state?.sidebarCollapsed).toBe(false);

      unsubscribe();
    });
  });
});
