/**
 * Property-based tests for API key masking
 * Feature: settings-ui, Property 5: API Key Masking
 * Validates: Requirements 1.4
 */

import { describe, it, expect } from 'vitest';
import * as fc from 'fast-check';
import { maskSensitiveValue, getLastFourCharacters, isMasked } from './masking';

describe('API Key Masking Property-Based Tests', () => {
  /**
   * Property 5: API Key Masking
   * For any API key stored in settings, when displayed in the UI,
   * the system SHALL show only the last 4 characters and mask the rest with asterisks.
   * Validates: Requirements 1.4
   */

  describe('maskSensitiveValue', () => {
    it('Property: For any string longer than 4 characters, only the last 4 characters should be visible', () => {
      // Generator for strings longer than 4 characters
      const apiKeyGenerator = fc.string({ minLength: 5, maxLength: 100 });

      fc.assert(
        fc.property(apiKeyGenerator, (apiKey) => {
          const masked = maskSensitiveValue(apiKey);
          
          // The masked value should end with the last 4 characters of the original
          const last4 = apiKey.slice(-4);
          expect(masked.endsWith(last4)).toBe(true);
          
          // The masked value should have the same length as the original
          expect(masked.length).toBe(apiKey.length);
          
          // All characters except the last 4 should be asterisks
          const maskedPart = masked.slice(0, -4);
          expect(maskedPart).toBe('*'.repeat(apiKey.length - 4));
          
          // The result should be marked as masked
          expect(isMasked(masked)).toBe(true);
        }),
        { numRuns: 100 }
      );
    });

    it('Property: For any string of 4 or fewer characters, all characters should be masked', () => {
      // Generator for strings of 1-4 characters
      const shortKeyGenerator = fc.string({ minLength: 1, maxLength: 4 });

      fc.assert(
        fc.property(shortKeyGenerator, (apiKey) => {
          const masked = maskSensitiveValue(apiKey);
          
          // All characters should be asterisks
          expect(masked).toBe('*'.repeat(apiKey.length));
          
          // The masked value should have the same length as the original
          expect(masked.length).toBe(apiKey.length);
          
          // The result should be marked as masked
          expect(isMasked(masked)).toBe(true);
        }),
        { numRuns: 100 }
      );
    });

    it('Property: For empty or null/undefined values, should return empty string', () => {
      // Test with empty string
      expect(maskSensitiveValue('')).toBe('');
      expect(maskSensitiveValue(null)).toBe('');
      expect(maskSensitiveValue(undefined)).toBe('');
    });

    it('Property: Masking should be idempotent for the visible part', () => {
      // Generator for API keys
      const apiKeyGenerator = fc.string({ minLength: 5, maxLength: 100 });

      fc.assert(
        fc.property(apiKeyGenerator, (apiKey) => {
          const masked = maskSensitiveValue(apiKey);
          const last4Original = apiKey.slice(-4);
          const last4Masked = masked.slice(-4);
          
          // The last 4 characters should be identical
          expect(last4Masked).toBe(last4Original);
        }),
        { numRuns: 100 }
      );
    });

    it('Property: For any alphanumeric API key, masking preserves length', () => {
      // Generator for realistic API keys (alphanumeric)
      const realisticApiKeyGenerator = fc
        .array(
          fc.constantFrom(
            ...'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_'.split('')
          ),
          { minLength: 10, maxLength: 64 }
        )
        .map((chars) => chars.join(''));

      fc.assert(
        fc.property(realisticApiKeyGenerator, (apiKey) => {
          const masked = maskSensitiveValue(apiKey);
          expect(masked.length).toBe(apiKey.length);
        }),
        { numRuns: 100 }
      );
    });

    it('Property: For any string with special characters, masking works correctly', () => {
      // Generator for strings with special characters
      const specialCharKeyGenerator = fc.string({ minLength: 5, maxLength: 50 });

      fc.assert(
        fc.property(specialCharKeyGenerator, (apiKey) => {
          const masked = maskSensitiveValue(apiKey);
          
          // Should preserve the last 4 characters exactly
          expect(masked.slice(-4)).toBe(apiKey.slice(-4));
          
          // Should mask the rest with asterisks
          expect(masked.slice(0, -4)).toBe('*'.repeat(apiKey.length - 4));
        }),
        { numRuns: 100 }
      );
    });

    it('Property: Masking different keys produces different masked values (for keys > 4 chars)', () => {
      // Generator for pairs of different API keys
      const differentKeysGenerator = fc
        .tuple(
          fc.string({ minLength: 5, maxLength: 50 }),
          fc.string({ minLength: 5, maxLength: 50 })
        )
        .filter(([key1, key2]) => key1 !== key2);

      fc.assert(
        fc.property(differentKeysGenerator, ([key1, key2]) => {
          const masked1 = maskSensitiveValue(key1);
          const masked2 = maskSensitiveValue(key2);
          
          // If the keys are different, the masked values should be different
          // (unless they happen to have the same length and last 4 characters)
          if (key1.length !== key2.length || key1.slice(-4) !== key2.slice(-4)) {
            expect(masked1).not.toBe(masked2);
          }
        }),
        { numRuns: 100 }
      );
    });
  });

  describe('getLastFourCharacters', () => {
    it('Property: For any string, returns the last 4 characters or the full string if shorter', () => {
      const stringGenerator = fc.string({ minLength: 0, maxLength: 100 });

      fc.assert(
        fc.property(stringGenerator, (str) => {
          const last4 = getLastFourCharacters(str);
          
          if (str.length <= 4) {
            expect(last4).toBe(str);
          } else {
            expect(last4).toBe(str.slice(-4));
            expect(last4.length).toBe(4);
          }
        }),
        { numRuns: 100 }
      );
    });

    it('Property: For empty or null/undefined values, returns empty string', () => {
      expect(getLastFourCharacters('')).toBe('');
      expect(getLastFourCharacters(null)).toBe('');
      expect(getLastFourCharacters(undefined)).toBe('');
    });
  });

  describe('isMasked', () => {
    it('Property: Any masked value should be detected as masked', () => {
      const apiKeyGenerator = fc.string({ minLength: 1, maxLength: 100 });

      fc.assert(
        fc.property(apiKeyGenerator, (apiKey) => {
          const masked = maskSensitiveValue(apiKey);
          
          // If the original key is not empty, the masked version should be detected as masked
          if (apiKey.length > 0) {
            expect(isMasked(masked)).toBe(true);
          }
        }),
        { numRuns: 100 }
      );
    });

    it('Property: Unmasked strings without asterisks should not be detected as masked', () => {
      // Generator for strings without asterisks
      const unmaskedGenerator = fc
        .array(
          fc.constantFrom(
            ...'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_'.split('')
          ),
          { minLength: 1, maxLength: 50 }
        )
        .map((chars) => chars.join(''));

      fc.assert(
        fc.property(unmaskedGenerator, (str) => {
          expect(isMasked(str)).toBe(false);
        }),
        { numRuns: 100 }
      );
    });
  });

  describe('Edge Cases', () => {
    it('handles exactly 4 character keys', () => {
      const key = 'abcd';
      const masked = maskSensitiveValue(key);
      expect(masked).toBe('****');
      expect(masked.length).toBe(4);
    });

    it('handles exactly 5 character keys', () => {
      const key = 'abcde';
      const masked = maskSensitiveValue(key);
      expect(masked).toBe('*bcde');
      expect(masked.slice(-4)).toBe('bcde');
    });

    it('handles single character keys', () => {
      const key = 'a';
      const masked = maskSensitiveValue(key);
      expect(masked).toBe('*');
    });

    it('handles very long keys', () => {
      const key = 'a'.repeat(1000);
      const masked = maskSensitiveValue(key);
      expect(masked.length).toBe(1000);
      expect(masked.slice(-4)).toBe('aaaa');
      expect(masked.slice(0, -4)).toBe('*'.repeat(996));
    });

    it('handles keys with unicode characters', () => {
      const key = '🔑🔐🔒🔓🗝️test';
      const masked = maskSensitiveValue(key);
      expect(masked.slice(-4)).toBe('test');
      expect(isMasked(masked)).toBe(true);
    });

    it('handles keys with only special characters', () => {
      const key = '!@#$%^&*()';
      const masked = maskSensitiveValue(key);
      expect(masked.slice(-4)).toBe('&*()');
      expect(masked.slice(0, -4)).toBe('******');
    });
  });
});
