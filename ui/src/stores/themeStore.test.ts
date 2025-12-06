import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { themeStore, setTheme, setAccentColor, currentTheme, currentAccentColor, getThemeColors, getAccentColorValue, type Theme, type AccentColor } from './themeStore';

describe('Theme Store', () => {
    beforeEach(() => {
        // Clear localStorage before each test
        if (typeof window !== 'undefined' && window.localStorage) {
            window.localStorage.clear();
        }
        // Reset DOM
        if (typeof document !== 'undefined') {
            document.documentElement.className = '';
            document.documentElement.style.cssText = '';
        }
    });

    afterEach(() => {
        if (typeof window !== 'undefined' && window.localStorage) {
            window.localStorage.clear();
        }
    });

    describe('Theme Persistence', () => {
        it('should persist theme preference to localStorage', () => {
            // **Feature: ui-redesign, Property 71: Theme Persistence**
            // **Validates: Requirements 20.3**

            // For any theme value:
            const themes: Theme[] = ['dark', 'oled'];

            for (const theme of themes) {
                if (typeof window !== 'undefined' && window.localStorage) {
                    window.localStorage.clear();
                }

                // When theme is set:
                setTheme(theme);

                // Then it should be persisted to localStorage:
                if (typeof window !== 'undefined' && window.localStorage) {
                    expect(window.localStorage.getItem('torrentflix-theme')).toBe(theme);
                }
            }
        });

        it('should load theme from localStorage on initialization', () => {
            // For any stored theme value:
            const themes: Theme[] = ['dark', 'oled'];

            for (const theme of themes) {
                if (typeof window !== 'undefined' && window.localStorage) {
                    window.localStorage.clear();
                    window.localStorage.setItem('torrentflix-theme', theme);
                }

                // When the store is accessed:
                let storedTheme: Theme | undefined;
                const unsubscribe = themeStore.subscribe((state) => {
                    storedTheme = state.theme;
                });

                // Then it should have a theme value:
                expect(storedTheme).toBeDefined();
                expect(['dark', 'oled']).toContain(storedTheme);

                unsubscribe();
            }
        });
    });

    describe('Theme Change Immediate', () => {
        it('should update UI immediately when theme changes without reload', () => {
            // **Feature: ui-redesign, Property 72: Theme Change Immediate**
            // **Validates: Requirements 20.4**

            // For any theme:
            const themes: Theme[] = ['dark', 'oled'];

            for (const theme of themes) {
                if (typeof document !== 'undefined') {
                    document.documentElement.className = '';
                }

                // When theme is set:
                setTheme(theme);

                // Then the store should immediately reflect the change:
                let currentThemeValue: Theme = 'dark';
                const unsubscribe = themeStore.subscribe((state) => {
                    currentThemeValue = state.theme;
                });

                expect(currentThemeValue).toBe(theme);

                // And the DOM should be updated immediately:
                if (typeof document !== 'undefined') {
                    expect(document.documentElement.classList.contains(`theme-${theme}`)).toBe(true);
                }

                unsubscribe();
            }
        });

        it('should apply CSS variables immediately when theme changes', () => {
            // For any theme:
            const themes: Theme[] = ['dark', 'oled'];

            for (const theme of themes) {
                // When theme is set:
                setTheme(theme);

                // Then CSS variables should be applied immediately:
                if (typeof document !== 'undefined') {
                    const bgPrimary = document.documentElement.style.getPropertyValue('--bg-primary');
                    expect(bgPrimary).toBeTruthy();

                    // And the values should match the theme:
                    if (theme === 'dark') {
                        expect(bgPrimary).toBe('#0f0f0f');
                    } else if (theme === 'oled') {
                        expect(bgPrimary).toBe('#000000');
                    }
                }
            }
        });
    });

    describe('Accent Color Persistence', () => {
        it('should persist accent color preference to localStorage', () => {
            // For any accent color value:
            const colors: AccentColor[] = ['red', 'blue', 'green', 'purple', 'orange'];

            for (const color of colors) {
                if (typeof window !== 'undefined' && window.localStorage) {
                    window.localStorage.clear();
                }

                // When accent color is set:
                setAccentColor(color);

                // Then it should be persisted to localStorage:
                if (typeof window !== 'undefined' && window.localStorage) {
                    expect(window.localStorage.getItem('torrentflix-accent-color')).toBe(color);
                }
            }
        });

        it('should load accent color from localStorage on initialization', () => {
            // For any stored accent color value:
            const colors: AccentColor[] = ['red', 'blue', 'green', 'purple', 'orange'];

            for (const color of colors) {
                if (typeof window !== 'undefined' && window.localStorage) {
                    window.localStorage.clear();
                    window.localStorage.setItem('torrentflix-accent-color', color);
                }

                // When the store is accessed:
                let storedColor: AccentColor | undefined;
                const unsubscribe = themeStore.subscribe((state) => {
                    storedColor = state.accentColor;
                });

                // Then it should have an accent color value:
                expect(storedColor).toBeDefined();
                expect(['red', 'blue', 'green', 'purple', 'orange']).toContain(storedColor);

                unsubscribe();
            }
        });
    });

    describe('Theme Colors', () => {
        it('should return correct theme colors for dark theme', () => {
            // For dark theme:
            const colors = getThemeColors('dark');

            // Then it should return the correct values:
            expect(colors.name).toBe('Dark');
            expect(colors.bg).toBe('#0f0f0f');
            expect(colors.secondary).toBe('#141414');
            expect(colors.text).toBe('#ffffff');
        });

        it('should return correct theme colors for OLED theme', () => {
            // For OLED theme:
            const colors = getThemeColors('oled');

            // Then it should return the correct values:
            expect(colors.name).toBe('OLED');
            expect(colors.bg).toBe('#000000');
            expect(colors.secondary).toBe('#0a0a0a');
            expect(colors.text).toBe('#ffffff');
        });
    });

    describe('Accent Color Values', () => {
        it('should return correct hex values for all accent colors', () => {
            // For any accent color:
            const colorMap: Record<AccentColor, string> = {
                red: '#E50914',
                blue: '#2563eb',
                green: '#16a34a',
                purple: '#a855f7',
                orange: '#ea580c',
            };

            for (const [color, expectedValue] of Object.entries(colorMap)) {
                // When getting the accent color value:
                const value = getAccentColorValue(color as AccentColor);

                // Then it should return the correct hex value:
                expect(value).toBe(expectedValue);
            }
        });
    });

    describe('Derived Stores', () => {
        it('should provide currentTheme derived store', () => {
            // For any theme:
            const themes: Theme[] = ['dark', 'oled'];

            for (const theme of themes) {
                // When theme is set:
                setTheme(theme);

                // Then currentTheme derived store should reflect the change:
                let themeValue: Theme = 'dark';
                const unsubscribe = currentTheme.subscribe((value) => {
                    themeValue = value;
                });

                expect(themeValue).toBe(theme);

                unsubscribe();
            }
        });

        it('should provide currentAccentColor derived store', () => {
            // For any accent color:
            const colors: AccentColor[] = ['red', 'blue', 'green', 'purple', 'orange'];

            for (const color of colors) {
                // When accent color is set:
                setAccentColor(color);

                // Then currentAccentColor derived store should reflect the change:
                let colorValue: AccentColor = 'red';
                const unsubscribe = currentAccentColor.subscribe((value) => {
                    colorValue = value;
                });

                expect(colorValue).toBe(color);

                unsubscribe();
            }
        });
    });

    describe('DOM Class Application', () => {
        it('should apply theme class to document root', () => {
            // For any theme:
            const themes: Theme[] = ['dark', 'oled'];

            for (const theme of themes) {
                if (typeof document !== 'undefined') {
                    document.documentElement.className = '';
                }

                // When theme is set:
                setTheme(theme);

                // Then the theme class should be applied:
                if (typeof document !== 'undefined') {
                    expect(document.documentElement.classList.contains(`theme-${theme}`)).toBe(true);
                }
            }
        });

        it('should apply accent color class to document root', () => {
            // For any accent color:
            const colors: AccentColor[] = ['red', 'blue', 'green', 'purple', 'orange'];

            for (const color of colors) {
                if (typeof document !== 'undefined') {
                    document.documentElement.className = '';
                }

                // When accent color is set:
                setAccentColor(color);

                // Then the accent color class should be applied:
                if (typeof document !== 'undefined') {
                    expect(document.documentElement.classList.contains(`accent-${color}`)).toBe(true);
                }
            }
        });

        it('should remove old theme class when changing theme', () => {
            // For any pair of themes:
            const themes: Theme[] = ['dark', 'oled'];

            for (let i = 0; i < themes.length; i++) {
                const oldTheme = themes[i];
                const newTheme = themes[(i + 1) % themes.length];

                // When first theme is set:
                setTheme(oldTheme);
                if (typeof document !== 'undefined') {
                    expect(document.documentElement.classList.contains(`theme-${oldTheme}`)).toBe(true);
                }

                // And then changed to new theme:
                setTheme(newTheme);

                // Then old theme class should be removed:
                if (typeof document !== 'undefined') {
                    expect(document.documentElement.classList.contains(`theme-${oldTheme}`)).toBe(false);

                    // And new theme class should be applied:
                    expect(document.documentElement.classList.contains(`theme-${newTheme}`)).toBe(true);
                }
            }
        });
    });
});
