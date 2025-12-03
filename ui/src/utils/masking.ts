/**
 * Utility functions for masking sensitive data in the UI
 */

/**
 * Masks a sensitive string by showing only the last 4 characters
 * @param value - The string to mask
 * @returns Masked string with asterisks, or empty string if value is empty
 * 
 * Examples:
 * - maskSensitiveValue("abc123def456") -> "********def456"
 * - maskSensitiveValue("key") -> "***"
 * - maskSensitiveValue("") -> ""
 */
export function maskSensitiveValue(value: string | undefined | null): string {
  if (!value) return '';
  if (value.length <= 4) return '*'.repeat(value.length);
  return '*'.repeat(value.length - 4) + value.slice(-4);
}

/**
 * Gets the last 4 characters of a string for verification purposes
 * @param value - The string to extract from
 * @returns Last 4 characters, or the full string if shorter than 4 chars
 */
export function getLastFourCharacters(value: string | undefined | null): string {
  if (!value) return '';
  return value.slice(-4);
}

/**
 * Checks if a value is masked (contains only asterisks and possibly some visible chars)
 * @param value - The string to check
 * @returns true if the string appears to be masked
 */
export function isMasked(value: string): boolean {
  return value.includes('*');
}
