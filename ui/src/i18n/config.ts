import { init, register } from 'svelte-i18n';

// Register locale loaders
register('ru', () => import('./locales/ru.json'));
register('en', () => import('./locales/en.json'));

// Initialize i18n immediately (synchronously) so it's ready before any component renders
init({
  fallbackLocale: 'en',
  initialLocale: 'ru',
});

// Keep this function for backward compatibility, but it's now a no-op
export function initializeI18n() {
  // i18n is already initialized above
}
