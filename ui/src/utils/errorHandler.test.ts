import { describe, it, expect, beforeEach, vi } from 'vitest';
import { safeInvoke, handleError, handleSuccess, createRetryHandler } from './errorHandler';
import * as uiStore from '../stores/uiStore';

// Mock the uiStore
vi.mock('../stores/uiStore', () => ({
  addToast: vi.fn(),
}));

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('Error Handler Utility', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('safeInvoke', () => {
    it('should return success result on successful invocation', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockResolvedValueOnce({ data: 'test' });

      const result = await safeInvoke('test_command', { arg: 'value' });

      expect(result.success).toBe(true);
      expect(result.data).toEqual({ data: 'test' });
      expect(result.error).toBeUndefined();
    });

    it('should return error result on failed invocation', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      const testError = new Error('Test error message');
      vi.mocked(invoke).mockRejectedValueOnce(testError);

      const result = await safeInvoke('test_command', { arg: 'value' });

      expect(result.success).toBe(false);
      expect(result.error).toBe('Test error message');
      expect(result.data).toBeUndefined();
    });

    it('should preserve previous state on error', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      const previousState = { items: [1, 2, 3] };
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Network error'));

      const result = await safeInvoke('test_command', {}, previousState);

      expect(result.success).toBe(false);
      expect(result.previousState).toEqual(previousState);
    });

    it('should handle string error messages', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockRejectedValueOnce('String error');

      const result = await safeInvoke('test_command');

      expect(result.success).toBe(false);
      expect(result.error).toBe('String error');
    });
  });

  describe('handleError', () => {
    it('should display timeout error message', () => {
      handleError('Request timeout', { command: 'test_command' });

      expect(uiStore.addToast).toHaveBeenCalledWith(
        'Запрос истёк. Повторить?',
        'error',
        5000
      );
    });

    it('should display connection error message', () => {
      handleError('Connection refused', { command: 'test_command' });

      expect(uiStore.addToast).toHaveBeenCalledWith(
        'Ошибка подключения. Повторить?',
        'error',
        5000
      );
    });

    it('should display not found error message', () => {
      handleError('Resource not found', { command: 'test_command' });

      expect(uiStore.addToast).toHaveBeenCalledWith(
        'Не найдено',
        'error',
        5000
      );
    });

    it('should display custom user message if provided', () => {
      handleError('Some error', {
        command: 'test_command',
        userMessage: 'Custom error message',
      });

      expect(uiStore.addToast).toHaveBeenCalledWith(
        'Custom error message',
        'error',
        5000
      );
    });

    it('should display generic error message', () => {
      handleError('Unknown error', { command: 'test_command' });

      expect(uiStore.addToast).toHaveBeenCalledWith(
        'Ошибка: Unknown error',
        'error',
        5000
      );
    });
  });

  describe('handleSuccess', () => {
    it('should display success message', () => {
      handleSuccess('Operation completed');

      expect(uiStore.addToast).toHaveBeenCalledWith(
        'Operation completed',
        'success',
        3000
      );
    });

    it('should display default success message', () => {
      handleSuccess();

      expect(uiStore.addToast).toHaveBeenCalledWith(
        'Успешно',
        'success',
        3000
      );
    });
  });

  describe('createRetryHandler', () => {
    it('should execute operation successfully', async () => {
      const operation = vi.fn().mockResolvedValueOnce(undefined);
      const retryHandler = createRetryHandler(operation);

      await retryHandler();

      expect(operation).toHaveBeenCalledTimes(1);
    });

    it('should retry on failure', async () => {
      const operation = vi
        .fn()
        .mockRejectedValueOnce(new Error('First attempt'))
        .mockResolvedValueOnce(undefined);

      const retryHandler = createRetryHandler(operation, 3);

      await retryHandler();
      await retryHandler();

      expect(operation).toHaveBeenCalledTimes(2);
    });

    it('should respect max retries limit', async () => {
      const operation = vi.fn().mockRejectedValue(new Error('Always fails'));
      const retryHandler = createRetryHandler(operation, 2);

      await retryHandler();
      await retryHandler();
      await retryHandler();

      expect(uiStore.addToast).toHaveBeenCalledWith(
        'Максимальное количество попыток достигнуто',
        'error',
        5000
      );
    });

    it('should reset retry count on success', async () => {
      const operation = vi
        .fn()
        .mockResolvedValueOnce(undefined)
        .mockRejectedValueOnce(new Error('Fail'))
        .mockResolvedValueOnce(undefined);

      const retryHandler = createRetryHandler(operation, 3);

      await retryHandler(); // Success
      await retryHandler(); // Fail
      await retryHandler(); // Success

      expect(operation).toHaveBeenCalledTimes(3);
    });
  });

  describe('Error Recovery', () => {
    it('should preserve state when error occurs during data fetch', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      const previousData = { items: ['item1', 'item2'] };

      vi.mocked(invoke).mockRejectedValueOnce(new Error('Network error'));

      const result = await safeInvoke('fetch_data', {}, previousData);

      expect(result.previousState).toEqual(previousData);
      expect(result.success).toBe(false);
    });

    it('should handle multiple consecutive errors', async () => {
      const { invoke } = await import('@tauri-apps/api/core');

      vi.mocked(invoke).mockRejectedValue(new Error('Persistent error'));

      const result1 = await safeInvoke('command1');
      const result2 = await safeInvoke('command2');

      expect(result1.success).toBe(false);
      expect(result2.success).toBe(false);
      expect(uiStore.addToast).not.toHaveBeenCalled(); // handleError not called in safeInvoke
    });
  });

  describe('Error Message Localization', () => {
    it('should use Russian error messages', () => {
      handleError('timeout', { command: 'test' });

      expect(uiStore.addToast).toHaveBeenCalledWith(
        expect.stringContaining('Запрос'),
        'error',
        5000
      );
    });

    it('should use Russian success messages', () => {
      handleSuccess();

      expect(uiStore.addToast).toHaveBeenCalledWith(
        'Успешно',
        'success',
        3000
      );
    });
  });
});
