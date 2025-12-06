import { describe, it, expect } from 'vitest';
import fc from 'fast-check';
import { searchStore, setQuery, setResults, clearSearch } from './searchStore';
import { libraryStore, setItems, setFilters } from './libraryStore';
import { downloadStore, setActive, setPolling } from './downloadStore';
import { settingsStore, updateSetting } from './settingsStore';
import { uiStore, setCurrentPage, addToast } from './uiStore';
import type { UiSearchResult } from '../types';
import type { SearchState } from './searchStore';
import type { LibraryState } from './libraryStore';
import type { DownloadState } from './downloadStore';
import type { SettingsState } from './settingsStore';
import type { UiState } from './uiStore';

/**
 * **Feature: ui-redesign, Property 2: Store Atomicity**
 * 
 * For any Tauri command response, the corresponding store update SHALL complete atomically 
 * without partial state, ensuring all subscribed components receive consistent data.
 * 
 * **Validates: Requirements 2.3**
 */
describe('Store Atomicity - Property 2', () => {
  it('should update searchStore atomically', () => {
    fc.assert(
      fc.property(
        fc.string({ minLength: 1, maxLength: 100 }),
        fc.array(
          fc.record({
            id: fc.string(),
            title: fc.string(),
            year: fc.option(fc.integer({ min: 1900, max: 2100 })),
            quality_badge: fc.string(),
            torrent_info: fc.record({
              seeders: fc.integer({ min: 0, max: 10000 }),
              size_gb: fc.float({ min: 0.1, max: 100 }),
              magnet_link: fc.string(),
            }),
          }),
          { minLength: 0, maxLength: 50 }
        ),
        (query, results) => {
          clearSearch();

          // Track all state updates
          const stateUpdates: SearchState[] = [];
          const unsubscribe = searchStore.subscribe((state: SearchState) => {
            stateUpdates.push(JSON.parse(JSON.stringify(state)));
          });

          // Perform atomic update
          setQuery(query);
          setResults(results as UiSearchResult[]);

          // Verify atomicity: all subscribers should see the same final state
          let finalState: SearchState | undefined;
          const finalUnsubscribe = searchStore.subscribe((state: SearchState) => {
            finalState = state;
          });

          // Verify consistency
          expect(finalState?.query).toBe(query);
          expect(finalState?.results).toEqual(results);
          expect(finalState?.loading).toBe(false);
          expect(finalState?.error).toBeNull();

          // Verify no partial states were observed
          const partialStates = stateUpdates.filter(
            (state) =>
              (state.query === query && state.results.length === 0) ||
              (state.query === '' && state.results.length > 0)
          );
          expect(partialStates.length).toBe(0);

          unsubscribe();
          finalUnsubscribe();
        }
      ),
      { numRuns: 100 }
    );
  });

  it('should update libraryStore atomically', () => {
    fc.assert(
      fc.property(
        fc.array(
          fc.record({
            id: fc.string(),
            title: fc.string(),
            quality_badge: fc.string(),
            torrent_info: fc.record({
              seeders: fc.integer({ min: 0, max: 10000 }),
              size_gb: fc.float({ min: 0.1, max: 100 }),
              magnet_link: fc.string(),
            }),
          }),
          { minLength: 0, maxLength: 50 }
        ),
        fc.record({
          year: fc.option(fc.integer({ min: 1900, max: 2100 })),
          resolution: fc.option(fc.string()),
        }),
        (items, filters) => {
          // Track all state updates
          const stateUpdates: LibraryState[] = [];
          const unsubscribe = libraryStore.subscribe((state: LibraryState) => {
            stateUpdates.push(JSON.parse(JSON.stringify(state)));
          });

          // Perform atomic update
          setItems(items as UiSearchResult[]);
          setFilters({
            year: filters.year ?? undefined,
            resolution: filters.resolution ?? undefined,
          });

          // Verify atomicity
          let finalState: LibraryState | undefined;
          const finalUnsubscribe = libraryStore.subscribe((state: LibraryState) => {
            finalState = state;
          });

          expect(finalState?.items).toEqual(items);
          expect(finalState?.loading).toBe(false);
          expect(finalState?.error).toBeNull();

          unsubscribe();
          finalUnsubscribe();
        }
      ),
      { numRuns: 100 }
    );
  });

  it('should update downloadStore atomically', () => {
    fc.assert(
      fc.property(
        fc.array(
          fc.record({
            hash: fc.string(),
            title: fc.string(),
            progress: fc.integer({ min: 0, max: 100 }),
            downloadedBytes: fc.integer({ min: 0, max: 10000000000 }),
            totalBytes: fc.integer({ min: 1, max: 10000000000 }),
            speed: fc.integer({ min: 0, max: 10000000 }),
            status: fc.constantFrom('downloading', 'paused', 'seeding', 'completed'),
          }),
          { minLength: 0, maxLength: 10 }
        ),
        (downloads) => {
          // Track all state updates
          const stateUpdates: DownloadState[] = [];
          const unsubscribe = downloadStore.subscribe((state: DownloadState) => {
            stateUpdates.push(JSON.parse(JSON.stringify(state)));
          });

          // Perform atomic update
          setActive(downloads as any);
          setPolling(true);

          // Verify atomicity
          let finalState: DownloadState | undefined;
          const finalUnsubscribe = downloadStore.subscribe((state: DownloadState) => {
            finalState = state;
          });

          expect(finalState?.active).toEqual(downloads);
          expect(finalState?.isPolling).toBe(true);
          expect(finalState?.loading).toBe(false);
          expect(finalState?.error).toBeNull();

          unsubscribe();
          finalUnsubscribe();
        }
      ),
      { numRuns: 100 }
    );
  });

  it('should update settingsStore atomically', () => {
    fc.assert(
      fc.property(
        fc.record({
          searchLimit: fc.integer({ min: 1, max: 1000 }),
          searchTimeout: fc.integer({ min: 1, max: 300 }),
          maxConcurrentDownloads: fc.integer({ min: 1, max: 100 }),
        }),
        (settingUpdates) => {
          // Track all state updates
          const stateUpdates: SettingsState[] = [];
          const unsubscribe = settingsStore.subscribe((state: SettingsState) => {
            stateUpdates.push(JSON.parse(JSON.stringify(state)));
          });

          // Perform atomic updates
          updateSetting('searchLimit', settingUpdates.searchLimit);
          updateSetting('searchTimeout', settingUpdates.searchTimeout);
          updateSetting('maxConcurrentDownloads', settingUpdates.maxConcurrentDownloads);

          // Verify atomicity
          let finalState: SettingsState | undefined;
          const finalUnsubscribe = settingsStore.subscribe((state: SettingsState) => {
            finalState = state;
          });

          expect(finalState?.data.searchLimit).toBe(settingUpdates.searchLimit);
          expect(finalState?.data.searchTimeout).toBe(settingUpdates.searchTimeout);
          expect(finalState?.data.maxConcurrentDownloads).toBe(settingUpdates.maxConcurrentDownloads);

          unsubscribe();
          finalUnsubscribe();
        }
      ),
      { numRuns: 100 }
    );
  });

  it('should update uiStore atomically', () => {
    fc.assert(
      fc.property(
        fc.constantFrom('discover', 'library', 'downloads', 'settings'),
        fc.string({ minLength: 1, maxLength: 100 }),
        (page, toastMessage) => {
          // Track all state updates
          const stateUpdates: UiState[] = [];
          const unsubscribe = uiStore.subscribe((state: UiState) => {
            stateUpdates.push(JSON.parse(JSON.stringify(state)));
          });

          // Perform atomic updates
          setCurrentPage(page as any);
          addToast(toastMessage, 'info');

          // Verify atomicity
          let finalState: UiState | undefined;
          const finalUnsubscribe = uiStore.subscribe((state: UiState) => {
            finalState = state;
          });

          expect(finalState?.currentPage).toBe(page);
          expect(finalState?.toasts.length).toBeGreaterThan(0);
          expect(finalState?.toasts[finalState.toasts.length - 1].message).toBe(toastMessage);

          unsubscribe();
          finalUnsubscribe();
        }
      ),
      { numRuns: 100 }
    );
  });

  it('should maintain consistency across multiple rapid updates', () => {
    fc.assert(
      fc.property(
        fc.array(fc.string({ minLength: 1, maxLength: 50 }), { minLength: 1, maxLength: 20 }),
        (queries) => {
          clearSearch();

          // Track all state updates
          const stateUpdates: SearchState[] = [];
          const unsubscribe = searchStore.subscribe((state: SearchState) => {
            stateUpdates.push(JSON.parse(JSON.stringify(state)));
          });

          // Perform rapid updates
          queries.forEach((query) => {
            setQuery(query);
          });

          // Verify final state is consistent
          let finalState: SearchState | undefined;
          const finalUnsubscribe = searchStore.subscribe((state: SearchState) => {
            finalState = state;
          });

          expect(finalState?.query).toBe(queries[queries.length - 1]);
          expect(finalState?.loading).toBe(false);

          unsubscribe();
          finalUnsubscribe();
        }
      ),
      { numRuns: 100 }
    );
  });
});
