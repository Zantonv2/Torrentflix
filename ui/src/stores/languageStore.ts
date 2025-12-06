import { writable, derived } from 'svelte/store';
import { locale, waitLocale } from 'svelte-i18n';

export type Language = 'en' | 'ru';

export interface LanguageState {
    currentLanguage: Language;
    isLoading: boolean;
}

const LANGUAGE_STORAGE_KEY = 'torrentflix-language';

// Default language
const defaultLanguage: Language = 'en';

// Load from localStorage or use defaults
function loadLanguageFromStorage(): Language {
    if (typeof window === 'undefined') return defaultLanguage;
    const stored = localStorage.getItem(LANGUAGE_STORAGE_KEY);
    return (stored as Language) || defaultLanguage;
}

// Create the language store
export const languageStore = writable<LanguageState>({
    currentLanguage: loadLanguageFromStorage(),
    isLoading: false,
});

// Helper functions
export const setLanguage = async (language: Language) => {
    languageStore.update((state) => ({ ...state, isLoading: true }));

    try {
        // Update svelte-i18n locale
        locale.set(language);

        // Wait for locale to be loaded
        await waitLocale();

        // Update store
        languageStore.update((state) => ({
            ...state,
            currentLanguage: language,
            isLoading: false,
        }));

        // Persist to localStorage
        if (typeof window !== 'undefined') {
            localStorage.setItem(LANGUAGE_STORAGE_KEY, language);
        }
    } catch (err) {
        console.error('Failed to set language:', err);
        languageStore.update((state) => ({ ...state, isLoading: false }));
    }
};

// Derived stores
export const currentLanguage = derived(languageStore, ($languageStore) => $languageStore.currentLanguage);
export const isLanguageLoading = derived(languageStore, ($languageStore) => $languageStore.isLoading);

// Get language display name
export const getLanguageName = (lang: Language): string => {
    const names: Record<Language, string> = {
        en: 'English',
        ru: 'Русский',
    };
    return names[lang];
};

// Get all supported languages
export const getSupportedLanguages = (): Language[] => {
    return ['en', 'ru'];
};
