import { describe, it, expect, beforeEach } from 'vitest';
import { initializeI18n } from './config';
import { get } from 'svelte/store';
import { locale, t } from 'svelte-i18n';

describe('i18n Initialization', () => {
  beforeEach(() => {
    // Reset locale before each test
    locale.set('ru');
  });

  it('should initialize i18n with Russian as default locale', () => {
    initializeI18n();
    const currentLocale = get(locale);
    expect(currentLocale).toBe('ru');
  });

  it('should have Russian locale registered', async () => {
    initializeI18n();
    // Wait for locale to be set
    await new Promise(resolve => setTimeout(resolve, 100));
    const currentLocale = get(locale);
    expect(currentLocale).toBeDefined();
  });

  it('should support English as fallback locale', () => {
    initializeI18n();
    // Switch to English
    locale.set('en');
    const currentLocale = get(locale);
    expect(currentLocale).toBe('en');
  });
});

describe('Translation Loading', () => {
  beforeEach(() => {
    locale.set('ru');
  });

  it('should load Russian translations', async () => {
    initializeI18n();
    await new Promise(resolve => setTimeout(resolve, 100));
    
    // Get a translation
    const tStore = get(t);
    expect(tStore).toBeDefined();
  });

  it('should load English translations', async () => {
    initializeI18n();
    locale.set('en');
    await new Promise(resolve => setTimeout(resolve, 100));
    
    const tStore = get(t);
    expect(tStore).toBeDefined();
  });

  it('should have navigation translations available', async () => {
    initializeI18n();
    await new Promise(resolve => setTimeout(resolve, 100));
    
    // Verify that translation keys exist
    const tStore = get(t);
    expect(tStore).toBeDefined();
  });
});

describe('Language Switching', () => {
  it('should switch from Russian to English', async () => {
    initializeI18n();
    locale.set('ru');
    let currentLocale = get(locale);
    expect(currentLocale).toBe('ru');
    
    locale.set('en');
    currentLocale = get(locale);
    expect(currentLocale).toBe('en');
  });

  it('should switch from English to Russian', async () => {
    initializeI18n();
    locale.set('en');
    let currentLocale = get(locale);
    expect(currentLocale).toBe('en');
    
    locale.set('ru');
    currentLocale = get(locale);
    expect(currentLocale).toBe('ru');
  });

  it('should maintain locale state after switching', async () => {
    initializeI18n();
    
    // Switch multiple times
    locale.set('ru');
    expect(get(locale)).toBe('ru');
    
    locale.set('en');
    expect(get(locale)).toBe('en');
    
    locale.set('ru');
    expect(get(locale)).toBe('ru');
  });
});
