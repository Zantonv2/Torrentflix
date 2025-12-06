import { init, register } from 'svelte-i18n';

// Register locale loaders
register('ru', () => import('./locales/ru.json'));
register('en', () => import('./locales/en.json'));

let isInitialized = false;

// Initialize i18n - can be called multiple times safely
export function initializeI18n() {
  if (isInitialized) return;

  try {
    init({
      fallbackLocale: 'en',
      initialLocale: 'ru',
    });
    isInitialized = true;
  } catch (err) {
    console.warn('Failed to initialize i18n:', err);
  }
}
