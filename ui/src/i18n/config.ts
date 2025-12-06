import { init, register, locale } from 'svelte-i18n';

// Register locale loaders
register('ru', () => import('./locales/ru.json'));
register('en', () => import('./locales/en.json'));

let isInitialized = false;

// Get initial locale from localStorage or use default
function getInitialLocale(): string {
  if (typeof window === 'undefined') return 'en';
  const stored = localStorage.getItem('torrentflix-language');
  return stored || 'en';
}

// Initialize i18n - can be called multiple times safely
export function initializeI18n() {
  if (isInitialized) return;

  try {
    const initialLocale = getInitialLocale();

    init({
      fallbackLocale: 'en',
      initialLocale: initialLocale,
    });

    // Set the locale
    locale.set(initialLocale);

    isInitialized = true;
  } catch (err) {
    console.warn('Failed to initialize i18n:', err);
  }
}
