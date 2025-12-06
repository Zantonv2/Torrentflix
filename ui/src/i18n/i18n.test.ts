import { describe, it, expect } from 'vitest';
import { initializeI18n } from './config';

describe('i18n Initialization', () => {
  it('should initialize i18n without errors', () => {
    expect(() => {
      initializeI18n();
    }).not.toThrow();
  });

  it('should be idempotent - calling multiple times should be safe', () => {
    expect(() => {
      initializeI18n();
      initializeI18n();
      initializeI18n();
    }).not.toThrow();
  });
});

describe('Translation Loading', () => {
  it('should have translation files available', async () => {
    const ruTranslations = await import('./locales/ru.json');
    expect(ruTranslations).toBeDefined();
    expect(Object.keys(ruTranslations.default).length).toBeGreaterThan(0);
  });

  it('should have English translations available', async () => {
    const enTranslations = await import('./locales/en.json');
    expect(enTranslations).toBeDefined();
    expect(Object.keys(enTranslations.default).length).toBeGreaterThan(0);
  });
});

describe('Language Support', () => {
  it('should support Russian locale', async () => {
    const ruTranslations = await import('./locales/ru.json');
    expect(ruTranslations.default).toBeDefined();
  });

  it('should support English locale', async () => {
    const enTranslations = await import('./locales/en.json');
    expect(enTranslations.default).toBeDefined();
  });
});
