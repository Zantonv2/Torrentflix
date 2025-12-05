import { init, register } from 'svelte-i18n';
import { writable } from 'svelte/store';

// Register locale loaders
register('ru', () => import('./locales/ru.json'));
register('en', () => import('./locales/en.json'));

let isInitialized = false;

// Store to track i18n readiness
export const i18nReady = writable(false);

// Initialize i18n - can be called multiple times safely
export function initializeI18n() {
  if (isInitialized) return;

  try {
    init({
      fallbackLocale: 'en',
      initialLocale: 'ru',
    });
    isInitialized = true;
    i18nReady.set(true);
  } catch (err) {
    console.warn('Failed to initialize i18n:', err);
    // Even if init fails, mark as ready to prevent infinite loops
    i18nReady.set(true);
  }
}
