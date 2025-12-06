import { describe, it, expect } from 'vitest';
import ruTranslations from './locales/ru.json';
import enTranslations from './locales/en.json';

/**
 * **Feature: ui-redesign, Property 8: Russian Localization Completeness**
 * **Validates: Requirements 13.1, 13.2, 13.3, 13.4, 13.5, 13.6, 13.7, 13.8**
 * 
 * For any page or component, all user-facing text (labels, buttons, placeholders, messages, errors, empty states)
 * SHALL be displayed in Russian.
 */
describe('Russian Localization Completeness', () => {
  /**
   * Property: All Russian translation keys have non-empty values
   * For any translation key in the Russian locale, the value SHALL be a non-empty string
   */
  it('should have all Russian translation keys with non-empty values', () => {
    const checkTranslations = (obj: any, path = ''): string[] => {
      const errors: string[] = [];
      
      for (const [key, value] of Object.entries(obj)) {
        const fullPath = path ? `${path}.${key}` : key;
        
        if (typeof value === 'string') {
          // Check that string values are not empty
          if (value.trim().length === 0) {
            errors.push(`Empty translation at ${fullPath}`);
          }
        } else if (typeof value === 'object' && value !== null) {
          // Recursively check nested objects
          errors.push(...checkTranslations(value, fullPath));
        }
      }
      
      return errors;
    };
    
    const errors = checkTranslations(ruTranslations);
    expect(errors).toHaveLength(0);
  });

  /**
   * Property: Russian and English locales have matching key structure
   * For any key in the Russian locale, the same key SHALL exist in the English locale
   */
  it('should have matching key structure between Russian and English', () => {
    const getKeys = (obj: any, path = ''): string[] => {
      const keys: string[] = [];
      
      for (const [key, value] of Object.entries(obj)) {
        const fullPath = path ? `${path}.${key}` : key;
        keys.push(fullPath);
        
        if (typeof value === 'object' && value !== null && typeof value !== 'string') {
          keys.push(...getKeys(value, fullPath));
        }
      }
      
      return keys;
    };
    
    const ruKeys = getKeys(ruTranslations);
    const enKeys = getKeys(enTranslations);
    
    // All Russian keys should exist in English
    const missingInEnglish = ruKeys.filter(key => !enKeys.includes(key));
    expect(missingInEnglish).toHaveLength(0);
    
    // All English keys should exist in Russian
    const missingInRussian = enKeys.filter(key => !ruKeys.includes(key));
    expect(missingInRussian).toHaveLength(0);
  });

  /**
   * Property: All required translation sections exist
   * For any required section (nav, header, discover, library, downloads, settings, common),
   * the section SHALL exist in the Russian locale
   */
  it('should have all required translation sections', () => {
    const requiredSections = [
      'nav',
      'header',
      'discover',
      'library',
      'library_browser',
      'library_collections',
      'library_analytics',
      'library_cleanup',
      'downloads',
      'settings',
      'common',
      'modal',
      'form',
      'toast',
    ];
    
    for (const section of requiredSections) {
      expect(ruTranslations).toHaveProperty(section);
      expect(typeof ruTranslations[section as keyof typeof ruTranslations]).toBe('object');
    }
  });

  /**
   * Property: Navigation translations are complete
   * For any navigation item (discover, library, downloads, settings),
   * the Russian translation SHALL be available
   */
  it('should have complete navigation translations', () => {
    const navItems = ['discover', 'library', 'downloads', 'settings'];
    
    for (const item of navItems) {
      expect(ruTranslations.nav).toHaveProperty(item);
      const translation = ruTranslations.nav[item as keyof typeof ruTranslations.nav];
      expect(typeof translation).toBe('string');
      expect((translation as string).length).toBeGreaterThan(0);
    }
  });

  /**
   * Property: Common UI strings are translated
   * For any common UI string (loading, error, success, cancel, save),
   * the Russian translation SHALL be available
   */
  it('should have common UI strings translated', () => {
    const commonStrings = [
      'loading',
      'error',
      'success',
      'warning',
      'info',
      'cancel',
      'save',
      'delete',
      'edit',
      'close',
      'retry',
      'yes',
      'no',
      'confirm',
    ];
    
    for (const str of commonStrings) {
      expect(ruTranslations.common).toHaveProperty(str);
      const translation = ruTranslations.common[str as keyof typeof ruTranslations.common];
      expect(typeof translation).toBe('string');
      expect((translation as string).length).toBeGreaterThan(0);
    }
  });

  /**
   * Property: Error messages are translated
   * For any error message key (connection_error, backend_error, timeout_error, not_found),
   * the Russian translation SHALL be available
   */
  it('should have error messages translated', () => {
    const errorMessages = [
      'connection_error',
      'backend_error',
      'timeout_error',
      'not_found',
    ];
    
    for (const msg of errorMessages) {
      expect(ruTranslations.common).toHaveProperty(msg);
      const translation = ruTranslations.common[msg as keyof typeof ruTranslations.common];
      expect(typeof translation).toBe('string');
      expect((translation as string).length).toBeGreaterThan(0);
    }
  });

  /**
   * Property: Settings section translations are complete
   * For any settings field (api_metadata, qbittorrent, filesystem, notifications, etc.),
   * the Russian translation SHALL be available
   */
  it('should have complete settings translations', () => {
    const settingsKeys = [
      'title',
      'api_metadata',
      'tmdb_api_key',
      'kinopoisk_token',
      'qbittorrent',
      'filesystem',
      'notifications',
      'application',
      'proxy',
      'save',
      'loading',
      'error',
      'success',
    ];
    
    for (const key of settingsKeys) {
      expect(ruTranslations.settings).toHaveProperty(key);
      const translation = ruTranslations.settings[key as keyof typeof ruTranslations.settings];
      expect(typeof translation).toBe('string');
      expect((translation as string).length).toBeGreaterThan(0);
    }
  });

  /**
   * Property: Downloads section translations are complete
   * For any downloads field (title, active_downloads, pause, resume, delete, etc.),
   * the Russian translation SHALL be available
   */
  it('should have complete downloads translations', () => {
    const downloadsKeys = [
      'title',
      'active_downloads',
      'progress',
      'speed',
      'pause',
      'resume',
      'delete',
      'empty_state',
    ];
    
    for (const key of downloadsKeys) {
      expect(ruTranslations.downloads).toHaveProperty(key);
      const translation = ruTranslations.downloads[key as keyof typeof ruTranslations.downloads];
      expect(typeof translation).toBe('string');
      expect((translation as string).length).toBeGreaterThan(0);
    }
  });

  /**
   * Property: No translation values contain only whitespace
   * For any translation value, it SHALL not consist only of whitespace characters
   */
  it('should not have whitespace-only translation values', () => {
    const checkForWhitespace = (obj: any, path = ''): string[] => {
      const errors: string[] = [];
      
      for (const [key, value] of Object.entries(obj)) {
        const fullPath = path ? `${path}.${key}` : key;
        
        if (typeof value === 'string') {
          if (value.trim().length === 0) {
            errors.push(`Whitespace-only translation at ${fullPath}`);
          }
        } else if (typeof value === 'object' && value !== null) {
          errors.push(...checkForWhitespace(value, fullPath));
        }
      }
      
      return errors;
    };
    
    const errors = checkForWhitespace(ruTranslations);
    expect(errors).toHaveLength(0);
  });

  /**
   * Property: Russian translations are primarily in Russian language
   * For any Russian translation value, it SHALL be a valid Russian translation
   * (may contain technical terms, numbers, and punctuation)
   */
  it('should have valid Russian translations', () => {
    const cyrillicRegex = /[\u0400-\u04FF]/; // Cyrillic Unicode range
    
    const checkRussian = (obj: any, path = ''): string[] => {
      const errors: string[] = [];
      
      for (const [key, value] of Object.entries(obj)) {
        const fullPath = path ? `${path}.${key}` : key;
        
        if (typeof value === 'string') {
          // Most Russian translations should have at least some Cyrillic characters
          // unless they're purely technical (like "TMDB", "API", etc.)
          const hasPlaceholder = value.includes('{') && value.includes('}');
          const isPurelyTechnical = /^[A-Z0-9\s\-.,!?:;()[\]{}«»"']+$/.test(value);
          
          if (!hasPlaceholder && !isPurelyTechnical && value.trim().length > 0) {
            // Check if it has at least one Cyrillic character
            if (!cyrillicRegex.test(value)) {
              errors.push(`No Cyrillic characters in ${fullPath}: "${value}"`);
            }
          }
        } else if (typeof value === 'object' && value !== null) {
          errors.push(...checkRussian(value, fullPath));
        }
      }
      
      return errors;
    };
    
    const errors = checkRussian(ruTranslations);
    expect(errors).toHaveLength(0);
  });
});
