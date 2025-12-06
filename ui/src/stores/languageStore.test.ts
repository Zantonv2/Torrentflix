import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { languageStore, setLanguage, currentLanguage, getSupportedLanguages, getLanguageName, type Language } from './languageStore';

describe('Language Store', () => {
    beforeEach(() => {
        // Clear localStorage before each test
        if (typeof window !== 'undefined' && window.localStorage) {
            window.localStorage.clear();
        }
    });

    afterEach(() => {
        if (typeof window !== 'undefined' && window.localStorage) {
            window.localStorage.clear();
        }
    });

    describe('Language Persistence', () => {
        it('should persist language preference to localStorage', async () => {
            // **Feature: ui-redesign, Property 46: Language Persistence**
            // **Validates: Requirements 10.4**

            // For any supported language:
            const languages: Language[] = ['en', 'ru'];

            for (const lang of languages) {
                if (typeof window !== 'undefined' && window.localStorage) {
                    window.localStorage.clear();
                }

                // When language is set:
                await setLanguage(lang);

                // Then it should be persisted to localStorage:
                if (typeof window !== 'undefined' && window.localStorage) {
                    expect(window.localStorage.getItem('torrentflix-language')).toBe(lang);
                }
            }
        });

        it('should load language from localStorage on initialization', () => {
            // For any stored language value:
            const languages: Language[] = ['en', 'ru'];

            for (const lang of languages) {
                if (typeof window !== 'undefined' && window.localStorage) {
                    window.localStorage.clear();
                    window.localStorage.setItem('torrentflix-language', lang);
                }

                // When the store is accessed:
                let storedLanguage: Language | undefined;
                const unsubscribe = languageStore.subscribe((state) => {
                    storedLanguage = state.currentLanguage;
                });

                // Then it should have a language value:
                expect(storedLanguage).toBeDefined();
                expect(['en', 'ru']).toContain(storedLanguage);

                unsubscribe();
            }
        });
    });

    describe('Supported Languages', () => {
        it('should support English and Russian languages', () => {
            // **Feature: ui-redesign, Property 44: Language Support**
            // **Validates: Requirements 10.2**

            // When getting supported languages:
            const languages = getSupportedLanguages();

            // Then it should include English and Russian:
            expect(languages).toContain('en');
            expect(languages).toContain('ru');
            expect(languages.length).toBe(2);
        });
    });

    describe('Language Display Names', () => {
        it('should return correct display names for all languages', () => {
            // For any supported language:
            const languages: Language[] = ['en', 'ru'];

            for (const lang of languages) {
                // When getting the language name:
                const name = getLanguageName(lang);

                // Then it should return a non-empty string:
                expect(name).toBeTruthy();
                expect(typeof name).toBe('string');
                expect(name.length).toBeGreaterThan(0);
            }
        });

        it('should return English for en language', () => {
            // When getting the name for English:
            const name = getLanguageName('en');

            // Then it should return 'English':
            expect(name).toBe('English');
        });

        it('should return Russian for ru language', () => {
            // When getting the name for Russian:
            const name = getLanguageName('ru');

            // Then it should return 'Русский':
            expect(name).toBe('Русский');
        });
    });

    describe('Derived Stores', () => {
        it('should provide currentLanguage derived store', async () => {
            // For any language:
            const languages: Language[] = ['en', 'ru'];

            for (const lang of languages) {
                // When language is set:
                await setLanguage(lang);

                // Then currentLanguage derived store should reflect the change:
                let languageValue: Language = 'en';
                const unsubscribe = currentLanguage.subscribe((value) => {
                    languageValue = value;
                });

                expect(languageValue).toBe(lang);

                unsubscribe();
            }
        });
    });

    describe('Language Loading State', () => {
        it('should track loading state when changing language', async () => {
            // For any language:
            const languages: Language[] = ['en', 'ru'];

            for (const lang of languages) {
                // When language is set:
                const promise = setLanguage(lang);

                // Then loading state should eventually be false:
                await promise;

                let isLoading = false;
                const unsubscribe = languageStore.subscribe((state) => {
                    isLoading = state.isLoading;
                });

                expect(isLoading).toBe(false);

                unsubscribe();
            }
        });
    });

    describe('Language Store State', () => {
        it('should initialize with default language', () => {
            // When the store is first accessed:
            let state;
            const unsubscribe = languageStore.subscribe((s) => {
                state = s;
            });

            // Then it should have a valid language:
            expect(state?.currentLanguage).toBeDefined();
            expect(['en', 'ru']).toContain(state?.currentLanguage);

            // And loading should be false:
            expect(state?.isLoading).toBe(false);

            unsubscribe();
        });

        it('should update language in store when setLanguage is called', async () => {
            // For any language:
            const languages: Language[] = ['en', 'ru'];

            for (const lang of languages) {
                // When language is set:
                await setLanguage(lang);

                // Then the store should reflect the new language:
                let state;
                const unsubscribe = languageStore.subscribe((s) => {
                    state = s;
                });

                expect(state?.currentLanguage).toBe(lang);

                unsubscribe();
            }
        });
    });

    describe('Language Switching', () => {
        it('should handle switching between all supported languages', async () => {
            // For any pair of languages:
            const languages: Language[] = ['en', 'ru'];

            for (const lang1 of languages) {
                for (const lang2 of languages) {
                    if (lang1 === lang2) continue;

                    // When switching from lang1 to lang2:
                    await setLanguage(lang1);
                    await setLanguage(lang2);

                    // Then the store should reflect lang2:
                    let currentLang: Language = 'en';
                    const unsubscribe = languageStore.subscribe((state) => {
                        currentLang = state.currentLanguage;
                    });

                    expect(currentLang).toBe(lang2);

                    unsubscribe();
                }
            }
        });
    });
});
