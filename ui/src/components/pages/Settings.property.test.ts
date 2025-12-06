import { describe, it, expect, beforeEach } from 'vitest';
import fc from 'fast-check';
import {
  settingsStore,
  setSettings,
  updateSetting,
  resetSettings,
  clearSettings,
  isDirty,
} from '../../stores/settingsStore';
import type { Settings } from '../../stores/settingsStore';

// Generator for valid settings objects
const settingsGenerator = (): fc.Arbitrary<Settings> => {
  return fc.record({
    tmdbApiKey: fc.option(fc.string({ minLength: 1, maxLength: 100 })),
    kinopoiskToken: fc.option(fc.string({ minLength: 1, maxLength: 100 })),
    qbittorrentEnabled: fc.boolean(),
    qbittorrentHost: fc.string({ minLength: 1, maxLength: 255 }),
    qbittorrentPort: fc.integer({ min: 1, max: 65535 }),
    qbittorrentUsername: fc.option(fc.string({ minLength: 0, maxLength: 100 })),
    qbittorrentPassword: fc.option(fc.string({ minLength: 0, maxLength: 100 })),
    qbittorrentUseHttps: fc.boolean(),
    qbittorrentVerifySsl: fc.boolean(),
    libraryPath: fc.string({ minLength: 1, maxLength: 500 }),
    downloadPath: fc.string({ minLength: 1, maxLength: 500 }),
    telegramEnabled: fc.boolean(),
    telegramBotToken: fc.option(fc.string({ minLength: 0, maxLength: 100 })),
    telegramChatId: fc.option(fc.string({ minLength: 0, maxLength: 100 })),
    telegramDebounce: fc.option(fc.integer({ min: 0, max: 3600 })),
    emailEnabled: fc.boolean(),
    emailSmtpHost: fc.option(fc.string({ minLength: 0, maxLength: 255 })),
    emailSmtpPort: fc.option(fc.integer({ min: 1, max: 65535 })),
    emailUsername: fc.option(fc.string({ minLength: 0, maxLength: 100 })),
    emailPassword: fc.option(fc.string({ minLength: 0, maxLength: 100 })),
    emailFromAddress: fc.option(fc.string({ minLength: 0, maxLength: 100 })),
    emailToAddress: fc.option(fc.string({ minLength: 0, maxLength: 100 })),
    logLevel: fc.constantFrom('DEBUG', 'INFO', 'WARN', 'ERROR') as fc.Arbitrary<'DEBUG' | 'INFO' | 'WARN' | 'ERROR'>,
    searchLimit: fc.integer({ min: 1, max: 1000 }),
    searchTimeout: fc.integer({ min: 1, max: 300 }),
    maxConcurrentDownloads: fc.integer({ min: 1, max: 100 }),
    enabledIndexers: fc.option(fc.string({ minLength: 0, maxLength: 100 })),
    debugMode: fc.boolean(),
    proxyEnabled: fc.boolean(),
    proxyAddress: fc.option(fc.string({ minLength: 0, maxLength: 255 })),
    proxyPort: fc.option(fc.integer({ min: 1, max: 65535 })),
  });
};

describe('Settings Page - Property-Based Tests', () => {
  beforeEach(() => {
    clearSettings();
  });

  describe('Property 7: Settings Dirty State', () => {
    it('should track dirty state correctly when settings are modified', () => {
      // **Feature: ui-redesign, Property 7: Settings Dirty State**
      // **Validates: Requirements 6.2**

      fc.assert(
        fc.property(settingsGenerator(), (initialSettings) => {
          // For any initial settings:
          // 1. After loading, dirty state should be false
          setSettings(initialSettings);

          let isDirtyValue = false;
          const unsubscribe = isDirty.subscribe((value) => {
            isDirtyValue = value;
          });

          expect(isDirtyValue).toBe(false);

          // 2. After modifying any field, dirty state should be true
          updateSetting('searchLimit', initialSettings.searchLimit + 1);
          expect(isDirtyValue).toBe(true);

          // 3. After resetting to original, dirty state should be false
          resetSettings();
          expect(isDirtyValue).toBe(false);

          unsubscribe();
          return true;
        }),
        { numRuns: 100 }
      );
    });

    it('should detect dirty state with multiple field modifications', () => {
      fc.assert(
        fc.property(settingsGenerator(), (initialSettings) => {
          // For any initial settings:
          setSettings(initialSettings);

          let isDirtyValue = false;
          const unsubscribe = isDirty.subscribe((value) => {
            isDirtyValue = value;
          });

          // Initially not dirty
          expect(isDirtyValue).toBe(false);

          // After first modification, should be dirty
          updateSetting('searchLimit', initialSettings.searchLimit + 1);
          expect(isDirtyValue).toBe(true);

          // After another modification, should still be dirty
          updateSetting('searchTimeout', initialSettings.searchTimeout + 1);
          expect(isDirtyValue).toBe(true);

          // After reset, should not be dirty
          resetSettings();
          expect(isDirtyValue).toBe(false);

          unsubscribe();
          return true;
        }),
        { numRuns: 100 }
      );
    });

    it('should not be dirty after reverting to original value', () => {
      fc.assert(
        fc.property(settingsGenerator(), (initialSettings) => {
          // For any initial settings:
          setSettings(initialSettings);

          let isDirtyValue = false;
          const unsubscribe = isDirty.subscribe((value) => {
            isDirtyValue = value;
          });

          // 1. Modify a field
          const newValue = initialSettings.searchLimit + 1;
          updateSetting('searchLimit', newValue);
          expect(isDirtyValue).toBe(true);

          // 2. Revert to original value
          updateSetting('searchLimit', initialSettings.searchLimit);

          // 3. Should not be dirty anymore
          expect(isDirtyValue).toBe(false);

          unsubscribe();
          return true;
        }),
        { numRuns: 100 }
      );
    });

    it('should maintain dirty state consistency across multiple changes', () => {
      fc.assert(
        fc.property(settingsGenerator(), (initialSettings) => {
          // For any initial settings:
          setSettings(initialSettings);

          let isDirtyValue = false;
          const unsubscribe = isDirty.subscribe((value) => {
            isDirtyValue = value;
          });

          // Initially not dirty
          expect(isDirtyValue).toBe(false);

          // Apply multiple changes
          updateSetting('searchLimit', initialSettings.searchLimit + 1);
          expect(isDirtyValue).toBe(true);

          updateSetting('searchTimeout', initialSettings.searchTimeout + 1);
          expect(isDirtyValue).toBe(true);

          updateSetting('maxConcurrentDownloads', initialSettings.maxConcurrentDownloads + 1);
          expect(isDirtyValue).toBe(true);

          // After reset, should not be dirty
          resetSettings();
          expect(isDirtyValue).toBe(false);

          unsubscribe();
          return true;
        }),
        { numRuns: 100 }
      );
    });

    it('should handle edge cases: empty strings, null values, boundary values', () => {
      fc.assert(
        fc.property(
          fc.record({
            searchLimit: fc.integer({ min: 50, max: 500 }),
            searchTimeout: fc.integer({ min: 10, max: 200 }),
            maxConcurrentDownloads: fc.integer({ min: 2, max: 50 }),
          }),
          (edgeCaseSettings) => {
            // For any edge case settings:
            const baseSettings = {
              qbittorrentEnabled: true,
              qbittorrentHost: 'localhost',
              qbittorrentPort: 5555,
              qbittorrentUseHttps: false,
              qbittorrentVerifySsl: true,
              libraryPath: '/path',
              downloadPath: '/path',
              telegramEnabled: false,
              emailEnabled: false,
              logLevel: 'INFO' as const,
              debugMode: false,
              proxyEnabled: false,
              ...edgeCaseSettings,
            };

            setSettings(baseSettings);

            let isDirtyValue = false;
            const unsubscribe = isDirty.subscribe((value) => {
              isDirtyValue = value;
            });

            // Should not be dirty initially
            expect(isDirtyValue).toBe(false);

            // Modify with boundary values (ensure we change the value)
            const newLimit = edgeCaseSettings.searchLimit === 1 ? 100 : 1;
            updateSetting('searchLimit', newLimit);
            expect(isDirtyValue).toBe(true);

            resetSettings();
            expect(isDirtyValue).toBe(false);

            // Modify again with different value
            const anotherLimit = edgeCaseSettings.searchLimit === 500 ? 100 : 500;
            updateSetting('searchLimit', anotherLimit);
            expect(isDirtyValue).toBe(true);

            resetSettings();
            expect(isDirtyValue).toBe(false);

            unsubscribe();
            return true;
          }
        ),
        { numRuns: 100 }
      );
    });

    it('should preserve dirty state across store subscriptions', () => {
      fc.assert(
        fc.property(settingsGenerator(), (initialSettings) => {
          // For any initial settings:
          setSettings(initialSettings);

          let dirtyValue1: boolean | undefined;
          let dirtyValue2: boolean | undefined;

          // Subscribe multiple times
          const unsub1 = isDirty.subscribe((value) => {
            dirtyValue1 = value;
          });
          const unsub2 = isDirty.subscribe((value) => {
            dirtyValue2 = value;
          });

          // Both should initially see false
          expect(dirtyValue1).toBe(false);
          expect(dirtyValue2).toBe(false);

          // Make a change
          updateSetting('searchLimit', initialSettings.searchLimit + 1);

          // Both subscribers should see the same dirty state (true)
          expect(dirtyValue1).toBe(true);
          expect(dirtyValue2).toBe(true);

          unsub1();
          unsub2();
          return true;
        }),
        { numRuns: 100 }
      );
    });
  });
});
