import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import LibraryCollections from './LibraryCollections.svelte';
import * as tauri from '@tauri-apps/api/core';

// Mock Tauri
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('LibraryCollections', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('7.5.1 Collection creation', () => {
    it('should create a new collection with name and description', async () => {
      const mockInvoke = vi.mocked(tauri.invoke);
      mockInvoke.mockResolvedValueOnce([]); // get_collections
      mockInvoke.mockResolvedValueOnce(123); // create_collection

      render(LibraryCollections);

      // Wait for initial load
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_collections');
      });

      // Click "+ New" button
      const newButton = screen.getByLabelText('Create new collection');
      await fireEvent.click(newButton);

      // Fill form
      const nameInput = screen.getByPlaceholderText('например, Избранное, К просмотру');
      const descInput = screen.getByPlaceholderText('Добавьте описание...');

      await fireEvent.change(nameInput, { target: { value: 'My Collection' } });
      await fireEvent.change(descInput, { target: { value: 'Test description' } });

      // Submit form
      const createButton = screen.getByText('Создать');
      await fireEvent.click(createButton);

      // Verify invoke was called with correct params
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('create_collection', {
          name: 'My Collection',
          description: 'Test description',
        });
      });
    });

    it('should prevent collection creation with empty name', async () => {
      const mockInvoke = vi.mocked(tauri.invoke);
      mockInvoke.mockResolvedValueOnce([]); // get_collections

      render(LibraryCollections);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_collections');
      });

      // Click "+ New" button
      const newButton = screen.getByLabelText('Create new collection');
      await fireEvent.click(newButton);

      // Try to submit without filling name
      const createButton = screen.getByText('Создать');
      expect(createButton).toBeDisabled();
    });

    it('should display error message on creation failure', async () => {
      const mockInvoke = vi.mocked(tauri.invoke);
      mockInvoke.mockResolvedValueOnce([]); // get_collections
      mockInvoke.mockRejectedValueOnce(new Error('Network error')); // create_collection fails

      render(LibraryCollections);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_collections');
      });

      // Click "+ New" button
      const newButton = screen.getByLabelText('Create new collection');
      await fireEvent.click(newButton);

      // Fill form
      const nameInput = screen.getByPlaceholderText('например, Избранное, К просмотру');
      await fireEvent.change(nameInput, { target: { value: 'Test' } });

      // Submit form
      const createButton = screen.getByText('Создать');
      await fireEvent.click(createButton);

      // Verify error is displayed
      await waitFor(() => {
        expect(screen.getByText(/Ошибка создания коллекции/)).toBeInTheDocument();
      });
    });
  });

  describe('7.5.2 Smart collection creation', () => {
    it('should create a smart collection with criteria', async () => {
      const mockInvoke = vi.mocked(tauri.invoke);
      mockInvoke.mockResolvedValueOnce([]); // get_collections
      mockInvoke.mockResolvedValueOnce(456); // create_smart_collection

      render(LibraryCollections);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_collections');
      });

      // Click "🧠 Smart" button
      const smartButton = screen.getByLabelText('Create smart collection');
      await fireEvent.click(smartButton);

      // Fill form
      const nameInput = screen.getByPlaceholderText('например, 4K фильмы, Непросмотренные');
      const patternInput = screen.getByPlaceholderText('например, Marvel');

      await fireEvent.change(nameInput, { target: { value: 'Marvel Movies' } });
      await fireEvent.change(patternInput, { target: { value: 'Marvel' } });

      // Submit form
      const createButton = screen.getByText('Создать');
      await fireEvent.click(createButton);

      // Verify invoke was called with correct params
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('create_smart_collection', {
          name: 'Marvel Movies',
          criteria: expect.objectContaining({
            title_pattern: 'Marvel',
          }),
        });
      });
    });

    it('should create smart collection with watch status filter', async () => {
      const mockInvoke = vi.mocked(tauri.invoke);
      mockInvoke.mockResolvedValueOnce([]); // get_collections
      mockInvoke.mockResolvedValueOnce(789); // create_smart_collection

      render(LibraryCollections);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_collections');
      });

      // Click "🧠 Smart" button
      const smartButton = screen.getByLabelText('Create smart collection');
      await fireEvent.click(smartButton);

      // Fill form
      const nameInput = screen.getByPlaceholderText('например, 4K фильмы, Непросмотренные');
      const statusSelect = screen.getByDisplayValue('Любой');

      await fireEvent.change(nameInput, { target: { value: 'Unwatched' } });
      await fireEvent.change(statusSelect, { target: { value: 'not_watched' } });

      // Submit form
      const createButton = screen.getByText('Создать');
      await fireEvent.click(createButton);

      // Verify invoke was called with correct params
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('create_smart_collection', {
          name: 'Unwatched',
          criteria: expect.objectContaining({
            watch_status: 'not_watched',
          }),
        });
      });
    });
  });

  describe('7.5.3 Adding items to collection', () => {
    it('should add selected items to collection', async () => {
      const mockInvoke = vi.mocked(tauri.invoke);
      const mockCollections = [
        { id: '1', name: 'Test Collection', description: '', item_count: 0, is_smart: false },
      ];
      const mockItems = [
        { id: '1', title: 'Movie 1', year: 2020 },
        { id: '2', title: 'Movie 2', year: 2021 },
      ];

      mockInvoke.mockResolvedValueOnce(mockCollections); // get_collections
      mockInvoke.mockResolvedValueOnce([]); // get_collection_items
      mockInvoke.mockResolvedValueOnce(mockItems); // query_library
      mockInvoke.mockResolvedValueOnce(undefined); // add_to_collection

      render(LibraryCollections);

      // Wait for initial load
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_collections');
      });

      // Select collection
      const collectionButton = screen.getByText('Test Collection');
      await fireEvent.click(collectionButton);

      // Click "Add" button
      const addButton = screen.getByLabelText('Add items to collection');
      await fireEvent.click(addButton);

      // Wait for items to load
      await waitFor(() => {
        expect(screen.getByText('Movie 1')).toBeInTheDocument();
      });

      // Select items
      const checkboxes = screen.getAllByRole('checkbox');
      await fireEvent.click(checkboxes[0]);
      await fireEvent.click(checkboxes[1]);

      // Submit
      const submitButton = screen.getByText(/Добавить \(2\)/);
      await fireEvent.click(submitButton);

      // Verify invoke was called
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('add_to_collection', {
          collection_id: '1',
          media_ids: expect.arrayContaining(['1', '2']),
        });
      });
    });

    it('should prevent adding items without selection', async () => {
      const mockInvoke = vi.mocked(tauri.invoke);
      const mockCollections = [
        { id: '1', name: 'Test Collection', description: '', item_count: 0, is_smart: false },
      ];

      mockInvoke.mockResolvedValueOnce(mockCollections); // get_collections
      mockInvoke.mockResolvedValueOnce([]); // get_collection_items

      render(LibraryCollections);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_collections');
      });

      // Select collection
      const collectionButton = screen.getByText('Test Collection');
      await fireEvent.click(collectionButton);

      // Click "Add" button
      const addButton = screen.getByLabelText('Add items to collection');
      await fireEvent.click(addButton);

      // Verify submit button is disabled
      const submitButton = screen.getByText(/Добавить \(0\)/);
      expect(submitButton).toBeDisabled();
    });

    it('should display error when adding items fails', async () => {
      const mockInvoke = vi.mocked(tauri.invoke);
      const mockCollections = [
        { id: '1', name: 'Test Collection', description: '', item_count: 0, is_smart: false },
      ];
      const mockItems = [{ id: '1', title: 'Movie 1', year: 2020 }];

      mockInvoke.mockResolvedValueOnce(mockCollections); // get_collections
      mockInvoke.mockResolvedValueOnce([]); // get_collection_items
      mockInvoke.mockResolvedValueOnce(mockItems); // query_library
      mockInvoke.mockRejectedValueOnce(new Error('Add failed')); // add_to_collection fails

      render(LibraryCollections);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_collections');
      });

      // Select collection
      const collectionButton = screen.getByText('Test Collection');
      await fireEvent.click(collectionButton);

      // Click "Add" button
      const addButton = screen.getByLabelText('Add items to collection');
      await fireEvent.click(addButton);

      // Wait for items to load
      await waitFor(() => {
        expect(screen.getByText('Movie 1')).toBeInTheDocument();
      });

      // Select item
      const checkbox = screen.getByRole('checkbox');
      await fireEvent.click(checkbox);

      // Submit
      const submitButton = screen.getByText(/Добавить \(1\)/);
      await fireEvent.click(submitButton);

      // Verify error is displayed
      await waitFor(() => {
        expect(screen.getByText(/Ошибка добавления элементов/)).toBeInTheDocument();
      });
    });
  });

  describe('Collection list display', () => {
    it('should display collections with item count', async () => {
      const mockInvoke = vi.mocked(tauri.invoke);
      const mockCollections = [
        { id: '1', name: 'Favorites', description: 'My favorites', item_count: 5, is_smart: false },
        { id: '2', name: 'Marvel', description: '', item_count: 10, is_smart: true },
      ];

      mockInvoke.mockResolvedValueOnce(mockCollections); // get_collections

      render(LibraryCollections);

      await waitFor(() => {
        expect(screen.getByText('Favorites')).toBeInTheDocument();
        expect(screen.getByText('Marvel')).toBeInTheDocument();
        expect(screen.getByText('5 элементов')).toBeInTheDocument();
        expect(screen.getByText('10 элементов')).toBeInTheDocument();
      });
    });

    it('should mark smart collections with icon', async () => {
      const mockInvoke = vi.mocked(tauri.invoke);
      const mockCollections = [
        { id: '1', name: 'Smart Collection', description: '', item_count: 0, is_smart: true },
      ];

      mockInvoke.mockResolvedValueOnce(mockCollections); // get_collections

      render(LibraryCollections);

      await waitFor(() => {
        expect(screen.getByText('🧠')).toBeInTheDocument();
      });
    });
  });
});
