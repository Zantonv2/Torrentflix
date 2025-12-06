import { writable, derived } from 'svelte/store';

export type Theme = 'dark' | 'oled';
export type AccentColor = 'red' | 'blue' | 'green' | 'purple' | 'orange';

export interface ThemeState {
    theme: Theme;
    accentColor: AccentColor;
}

const THEME_STORAGE_KEY = 'torrentflix-theme';
const ACCENT_COLOR_STORAGE_KEY = 'torrentflix-accent-color';

// Default theme state
const defaultTheme: Theme = 'dark';
const defaultAccentColor: AccentColor = 'red';

// Load from localStorage or use defaults
function loadThemeFromStorage(): Theme {
    if (typeof window === 'undefined') return defaultTheme;
    const stored = localStorage.getItem(THEME_STORAGE_KEY);
    return (stored as Theme) || defaultTheme;
}

function loadAccentColorFromStorage(): AccentColor {
    if (typeof window === 'undefined') return defaultAccentColor;
    const stored = localStorage.getItem(ACCENT_COLOR_STORAGE_KEY);
    return (stored as AccentColor) || defaultAccentColor;
}

// Create the theme store
export const themeStore = writable<ThemeState>({
    theme: loadThemeFromStorage(),
    accentColor: loadAccentColorFromStorage(),
});

// Helper functions
export const setTheme = (theme: Theme) => {
    themeStore.update((state) => ({ ...state, theme }));
    if (typeof window !== 'undefined') {
        localStorage.setItem(THEME_STORAGE_KEY, theme);
    }
    applyThemeToDOM(theme);
};

export const setAccentColor = (accentColor: AccentColor) => {
    themeStore.update((state) => ({ ...state, accentColor }));
    if (typeof window !== 'undefined') {
        localStorage.setItem(ACCENT_COLOR_STORAGE_KEY, accentColor);
    }
    applyAccentColorToDOM(accentColor);
};

// Apply theme to DOM
function applyThemeToDOM(theme: Theme) {
    if (typeof document === 'undefined') return;

    const root = document.documentElement;

    // Remove all theme classes
    root.classList.remove('theme-dark', 'theme-oled');

    // Add the new theme class
    root.classList.add(`theme-${theme}`);

    // Update CSS variables for theme
    if (theme === 'dark') {
        root.style.setProperty('--bg-primary', '#0f0f0f');
        root.style.setProperty('--bg-secondary', '#141414');
        root.style.setProperty('--bg-tertiary', '#1a1a1a');
        root.style.setProperty('--text-primary', '#ffffff');
        root.style.setProperty('--text-secondary', '#b3b3b3');
    } else if (theme === 'oled') {
        root.style.setProperty('--bg-primary', '#000000');
        root.style.setProperty('--bg-secondary', '#0a0a0a');
        root.style.setProperty('--bg-tertiary', '#141414');
        root.style.setProperty('--text-primary', '#ffffff');
        root.style.setProperty('--text-secondary', '#b3b3b3');
    }
}

// Apply accent color to DOM
function applyAccentColorToDOM(accentColor: AccentColor) {
    if (typeof document === 'undefined') return;

    const root = document.documentElement;

    // Remove all accent color classes
    root.classList.remove('accent-red', 'accent-blue', 'accent-green', 'accent-purple', 'accent-orange');

    // Add the new accent color class
    root.classList.add(`accent-${accentColor}`);

    // Update CSS variables for accent color
    const accentColors: Record<AccentColor, string> = {
        red: '#E50914',
        blue: '#2563eb',
        green: '#16a34a',
        purple: '#a855f7',
        orange: '#ea580c',
    };

    root.style.setProperty('--accent-color', accentColors[accentColor]);
}

// Initialize theme on store creation
if (typeof window !== 'undefined') {
    const initialTheme = loadThemeFromStorage();
    const initialAccentColor = loadAccentColorFromStorage();
    applyThemeToDOM(initialTheme);
    applyAccentColorToDOM(initialAccentColor);
}

// Derived stores
export const currentTheme = derived(themeStore, ($themeStore) => $themeStore.theme);
export const currentAccentColor = derived(themeStore, ($themeStore) => $themeStore.accentColor);

// Get theme colors for display
export const getThemeColors = (theme: Theme) => {
    if (theme === 'dark') {
        return {
            name: 'Dark',
            bg: '#0f0f0f',
            secondary: '#141414',
            text: '#ffffff',
        };
    } else {
        return {
            name: 'OLED',
            bg: '#000000',
            secondary: '#0a0a0a',
            text: '#ffffff',
        };
    }
};

export const getAccentColorValue = (color: AccentColor): string => {
    const colors: Record<AccentColor, string> = {
        red: '#E50914',
        blue: '#2563eb',
        green: '#16a34a',
        purple: '#a855f7',
        orange: '#ea580c',
    };
    return colors[color];
};
