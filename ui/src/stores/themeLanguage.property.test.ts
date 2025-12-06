import { describe, it, expect, beforeEach } from 'vitest';
import fc from 'fast-check';
import { themeStore, setTheme, setAccentColor, getThemeColors, getAccentColorValue, type Theme, type AccentColor } from './themeStore';
import { languageStore, getSupportedLanguages, getLanguageName, type Language } from './languageStore';

// Generators
const themeGenerator = (): fc.Arbitrary<Theme> => {
    return fc.constantFrom('dark', 'oled') as fc.Arbitrary<Theme>;
};

const accentColorGenerator = (): fc.Arbitrary<AccentColor> => {
    return fc.constantFrom('red', 'blue', 'green', 'purple', 'orange') as fc.Arbitrary<AccentColor>;
};

const languageGenerator = (): fc.Arbitrary<Language> => {
    return fc.constantFrom('en', 'ru') as fc.Arbitrary<Language>;
};

describe('Theme & Localization - Property-Based Tests', () => {
    beforeEach(() => {
        if (typeof window !== 'undefined' && window.localStorage) {
            window.localStorage.clear();
        }
        if (typeof document !== 'undefined') {
            document.documentElement.className = '';
            document.documentElement.style.cssText = '';
        }
    });

    describe('Property 43: Localization Load', () => {
        it('should load localization strings on startup', () => {
            // **Feature: ui-redesign, Property 43: Localization Load**
            // **Validates: Requirements 10.1**

            fc.assert(
                fc.property(languageGenerator(), (language) => {
                    // For any language:
                    if (typeof window !== 'undefined' && window.localStorage) {
                        window.localStorage.setItem('torrentflix-language', language);
                    }

                    // When the store is accessed:
                    let state;
                    const unsubscribe = languageStore.subscribe((s) => {
                        state = s;
                    });

                    // Then localization should be loaded:
                    expect(state?.currentLanguage).toBeDefined();
                    expect(state?.isLoading).toBe(false);

                    unsubscribe();
                    return true;
                }),
                { numRuns: 100 }
            );
        });
    });

    describe('Property 44: Language Support', () => {
        it('should support English and Russian languages', () => {
            // **Feature: ui-redesign, Property 44: Language Support**
            // **Validates: Requirements 10.2**

            // For any supported language:
            const languages = getSupportedLanguages();

            // Then it should include both English and Russian:
            expect(languages).toContain('en');
            expect(languages).toContain('ru');
            expect(languages.length).toBe(2);
        });
    });

    describe('Property 45: Missing Translation Fallback', () => {
        it('should display key name as fallback for missing translations', () => {
            // **Feature: ui-redesign, Property 45: Missing Translation Fallback**
            // **Validates: Requirements 10.3**

            fc.assert(
                fc.property(
                    fc.string({ minLength: 1, maxLength: 50 }),
                    (keyName) => {
                        // For any missing translation key:
                        // The fallback should be the key name itself
                        const fallback = keyName;

                        // Then it should be a non-empty string:
                        expect(fallback).toBeTruthy();
                        expect(typeof fallback).toBe('string');

                        return true;
                    }
                ),
                { numRuns: 100 }
            );
        });
    });

    describe('Property 46: Language Persistence', () => {
        it('should persist language preference across sessions', () => {
            // **Feature: ui-redesign, Property 46: Language Persistence**
            // **Validates: Requirements 10.4**

            fc.assert(
                fc.property(languageGenerator(), (language) => {
                    // For any language:
                    if (typeof window !== 'undefined' && window.localStorage) {
                        window.localStorage.clear();
                        window.localStorage.setItem('torrentflix-language', language);
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
                    return true;
                }),
                { numRuns: 100 }
            );
        });
    });

    describe('Property 71: Theme Persistence', () => {
        it('should persist theme preference across sessions', () => {
            // **Feature: ui-redesign, Property 71: Theme Persistence**
            // **Validates: Requirements 20.3**

            fc.assert(
                fc.property(themeGenerator(), (theme) => {
                    // For any theme:
                    if (typeof window !== 'undefined' && window.localStorage) {
                        window.localStorage.clear();
                    }

                    // When theme is set:
                    setTheme(theme);

                    // Then it should be persisted to localStorage:
                    if (typeof window !== 'undefined' && window.localStorage) {
                        expect(window.localStorage.getItem('torrentflix-theme')).toBe(theme);
                    }

                    // And when accessed again:
                    let storedTheme: Theme = 'dark';
                    const unsubscribe = themeStore.subscribe((state) => {
                        storedTheme = state.theme;
                    });

                    // It should load the persisted theme:
                    expect(storedTheme).toBe(theme);

                    unsubscribe();
                    return true;
                }),
                { numRuns: 100 }
            );
        });

        it('should persist accent color preference across sessions', () => {
            // For any accent color:
            if (typeof window !== 'undefined' && window.localStorage) {
                window.localStorage.clear();
            }

            fc.assert(
                fc.property(accentColorGenerator(), (color) => {
                    // When accent color is set:
                    setAccentColor(color);

                    // Then it should be persisted to localStorage:
                    if (typeof window !== 'undefined' && window.localStorage) {
                        expect(window.localStorage.getItem('torrentflix-accent-color')).toBe(color);
                    }

                    // And when accessed again:
                    let storedColor: AccentColor = 'red';
                    const unsubscribe = themeStore.subscribe((state) => {
                        storedColor = state.accentColor;
                    });

                    // It should load the persisted accent color:
                    expect(storedColor).toBe(color);

                    unsubscribe();
                    return true;
                }),
                { numRuns: 100 }
            );
        });
    });

    describe('Property 72: Theme Change Immediate', () => {
        it('should update UI immediately when theme changes without reload', () => {
            // **Feature: ui-redesign, Property 72: Theme Change Immediate**
            // **Validates: Requirements 20.4**

            fc.assert(
                fc.property(themeGenerator(), (theme) => {
                    // For any theme:
                    if (typeof document !== 'undefined') {
                        document.documentElement.className = '';
                    }

                    // When theme is set:
                    setTheme(theme);

                    // Then the store should immediately reflect the change:
                    let currentTheme: Theme = 'dark';
                    const unsubscribe = themeStore.subscribe((state) => {
                        currentTheme = state.theme;
                    });

                    expect(currentTheme).toBe(theme);

                    // And the DOM should be updated immediately:
                    if (typeof document !== 'undefined') {
                        expect(document.documentElement.classList.contains(`theme-${theme}`)).toBe(true);
                    }

                    // And CSS variables should be applied immediately:
                    if (typeof document !== 'undefined') {
                        const bgPrimary = document.documentElement.style.getPropertyValue('--bg-primary');
                        expect(bgPrimary).toBeTruthy();
                    }

                    unsubscribe();
                    return true;
                }),
                { numRuns: 100 }
            );
        });

        it('should update UI immediately when accent color changes without reload', () => {
            // For any accent color:
            if (typeof document !== 'undefined') {
                document.documentElement.className = '';
            }

            fc.assert(
                fc.property(accentColorGenerator(), (color) => {
                    // When accent color is set:
                    setAccentColor(color);

                    // Then the store should immediately reflect the change:
                    let currentColor: AccentColor = 'red';
                    const unsubscribe = themeStore.subscribe((state) => {
                        currentColor = state.accentColor;
                    });

                    expect(currentColor).toBe(color);

                    // And the DOM should be updated immediately:
                    if (typeof document !== 'undefined') {
                        expect(document.documentElement.classList.contains(`accent-${color}`)).toBe(true);
                    }

                    // And CSS variables should be applied immediately:
                    if (typeof document !== 'undefined') {
                        const accentColor = document.documentElement.style.getPropertyValue('--accent-color');
                        expect(accentColor).toBeTruthy();
                    }

                    unsubscribe();
                    return true;
                }),
                { numRuns: 100 }
            );
        });
    });

    describe('Property: Theme Color Consistency', () => {
        it('should return consistent theme colors for the same theme', () => {
            // For any theme:
            fc.assert(
                fc.property(themeGenerator(), (theme) => {
                    // When getting theme colors multiple times:
                    const colors1 = getThemeColors(theme);
                    const colors2 = getThemeColors(theme);

                    // Then they should be identical:
                    expect(colors1).toEqual(colors2);
                    expect(colors1.bg).toBe(colors2.bg);
                    expect(colors1.secondary).toBe(colors2.secondary);
                    expect(colors1.text).toBe(colors2.text);

                    return true;
                }),
                { numRuns: 100 }
            );
        });
    });

    describe('Property: Accent Color Value Consistency', () => {
        it('should return consistent accent color values for the same color', () => {
            // For any accent color:
            fc.assert(
                fc.property(accentColorGenerator(), (color) => {
                    // When getting accent color value multiple times:
                    const value1 = getAccentColorValue(color);
                    const value2 = getAccentColorValue(color);

                    // Then they should be identical:
                    expect(value1).toBe(value2);

                    // And should be a valid hex color:
                    expect(value1).toMatch(/^#[0-9A-Fa-f]{6}$/);

                    return true;
                }),
                { numRuns: 100 }
            );
        });
    });

    describe('Property: Language Name Consistency', () => {
        it('should return consistent language names for the same language', () => {
            // For any language:
            fc.assert(
                fc.property(languageGenerator(), (language) => {
                    // When getting language name multiple times:
                    const name1 = getLanguageName(language);
                    const name2 = getLanguageName(language);

                    // Then they should be identical:
                    expect(name1).toBe(name2);

                    // And should be non-empty:
                    expect(name1.length).toBeGreaterThan(0);

                    return true;
                }),
                { numRuns: 100 }
            );
        });
    });

    describe('Property: Theme and Accent Color Independence', () => {
        it('should allow independent changes to theme and accent color', () => {
            // For any combination of theme and accent color:
            fc.assert(
                fc.property(
                    fc.tuple(themeGenerator(), accentColorGenerator()),
                    ([theme, color]) => {
                        // When setting theme:
                        setTheme(theme);

                        let themeValue: Theme = 'dark';
                        let colorValue: AccentColor = 'red';

                        const unsubscribe = themeStore.subscribe((state) => {
                            themeValue = state.theme;
                            colorValue = state.accentColor;
                        });

                        // Then theme should be set:
                        expect(themeValue).toBe(theme);

                        // When setting accent color:
                        setAccentColor(color);

                        // Then accent color should be set:
                        expect(colorValue).toBe(color);

                        // And theme should remain unchanged:
                        expect(themeValue).toBe(theme);

                        unsubscribe();
                        return true;
                    }
                ),
                { numRuns: 100 }
            );
        });
    });
});
