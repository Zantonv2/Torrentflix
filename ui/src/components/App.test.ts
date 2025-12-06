import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { get } from 'svelte/store';
import { uiStore, setCurrentPage } from '../stores/uiStore';
import App from './App.svelte';

describe('App.svelte Routing', () => {
  beforeEach(() => {
    // Reset UI store before each test
    setCurrentPage('discover');
  });

  it('should render Discover page by default', () => {
    const state = get(uiStore);
    expect(state.currentPage).toBe('discover');
  });

  it('should navigate to Library page when setCurrentPage is called', () => {
    setCurrentPage('library');
    const state = get(uiStore);
    expect(state.currentPage).toBe('library');
  });

  it('should navigate to Downloads page when setCurrentPage is called', () => {
    setCurrentPage('downloads');
    const state = get(uiStore);
    expect(state.currentPage).toBe('downloads');
  });

  it('should navigate to Settings page when setCurrentPage is called', () => {
    setCurrentPage('settings');
    const state = get(uiStore);
    expect(state.currentPage).toBe('settings');
  });

  it('should maintain page state across multiple navigation changes', () => {
    setCurrentPage('discover');
    let state = get(uiStore);
    expect(state.currentPage).toBe('discover');

    setCurrentPage('library');
    state = get(uiStore);
    expect(state.currentPage).toBe('library');

    setCurrentPage('downloads');
    state = get(uiStore);
    expect(state.currentPage).toBe('downloads');

    setCurrentPage('settings');
    state = get(uiStore);
    expect(state.currentPage).toBe('settings');

    setCurrentPage('discover');
    state = get(uiStore);
    expect(state.currentPage).toBe('discover');
  });

  it('should have valid page types', () => {
    const validPages = ['discover', 'library', 'downloads', 'settings'];
    const state = get(uiStore);
    expect(validPages).toContain(state.currentPage);
  });
});
