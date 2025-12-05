import { addToast } from '../stores/uiStore';

/**
 * Error handler utility for Tauri command invocations
 * Handles error recovery, state preservation, and user-friendly messages
 */

export interface ErrorContext {
  command: string;
  previousState?: any;
  userMessage?: string;
}

export interface ErrorResult<T> {
  success: boolean;
  data?: T;
  error?: string;
  previousState?: any;
}

/**
 * Wraps a Tauri invoke call with error handling
 * Preserves previous state on error and displays user-friendly messages
 */
export async function safeInvoke<T>(
  command: string,
  args?: Record<string, any>,
  previousState?: any
): Promise<ErrorResult<T>> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const result = await invoke<T>(command, args);
    return {
      success: true,
      data: result,
    };
  } catch (err) {
    const errorMessage = err instanceof Error ? err.message : String(err);
    console.error(`[${command}] Error:`, err);

    return {
      success: false,
      error: errorMessage,
      previousState,
    };
  }
}

/**
 * Handles error display and recovery
 * Shows toast notification and optionally restores previous state
 */
export function handleError(
  error: string,
  context: Partial<ErrorContext> = {}
): void {
  const { command = 'Unknown', userMessage } = context;

  // Determine error type and show appropriate message
  if (error.includes('timeout') || error.includes('Timeout')) {
    addToast('Запрос истёк. Повторить?', 'error', 5000);
  } else if (error.includes('connection') || error.includes('Connection')) {
    addToast('Ошибка подключения. Повторить?', 'error', 5000);
  } else if (error.includes('not found') || error.includes('Not found')) {
    addToast('Не найдено', 'error', 5000);
  } else if (userMessage) {
    addToast(userMessage, 'error', 5000);
  } else {
    addToast(`Ошибка: ${error}`, 'error', 5000);
  }

  console.error(`[${command}] Error handled:`, error);
}

/**
 * Handles successful operation
 * Shows success toast notification
 */
export function handleSuccess(message: string = 'Успешно'): void {
  addToast(message, 'success', 3000);
}

/**
 * Retry helper - returns a function that can be called to retry an operation
 */
export function createRetryHandler(
  operation: () => Promise<void>,
  maxRetries: number = 3
): () => Promise<void> {
  let retryCount = 0;

  return async () => {
    if (retryCount >= maxRetries) {
      addToast('Максимальное количество попыток достигнуто', 'error', 5000);
      return;
    }

    retryCount++;
    try {
      await operation();
      retryCount = 0; // Reset on success
    } catch (err) {
      console.error(`Retry attempt ${retryCount}/${maxRetries} failed:`, err);
      if (retryCount < maxRetries) {
        addToast(`Попытка ${retryCount}/${maxRetries}...`, 'info', 2000);
      }
    }
  };
}
