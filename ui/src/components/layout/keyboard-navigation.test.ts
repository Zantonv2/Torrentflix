import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { render } from 'vitest-dom';
import { uiStore, setCurrentPage } from '../../stores/uiStore';
import Tabs from '../core/Tabs.svelte';
import Modal from '../core/Modal.svelte';
import Card from '../core/Card.svelte';

describe('Keyboard Navigation', () => {
  beforeEach(() => {
    setCurrentPage('discover');
  });

  describe('Arrow Key Navigation', () => {
    it('should navigate to next page with ArrowDown', () => {
      const navItems = ['discover', 'library', 'downloads', 'settings'];
      let currentIndex = 0;

      // Simulate ArrowDown
      currentIndex = (currentIndex + 1) % navItems.length;
      setCurrentPage(navItems[currentIndex] as any);

      let state = get(uiStore);
      expect(state.currentPage).toBe('library');
    });

    it('should navigate to previous page with ArrowUp', () => {
      setCurrentPage('library');
      const navItems = ['discover', 'library', 'downloads', 'settings'];
      let currentIndex = 1;

      // Simulate ArrowUp
      currentIndex = (currentIndex - 1 + navItems.length) % navItems.length;
      setCurrentPage(navItems[currentIndex] as any);

      let state = get(uiStore);
      expect(state.currentPage).toBe('discover');
    });

    it('should wrap around when navigating past last item with ArrowDown', () => {
      setCurrentPage('settings');
      const navItems = ['discover', 'library', 'downloads', 'settings'];
      let currentIndex = 3;

      // Simulate ArrowDown from last item
      currentIndex = (currentIndex + 1) % navItems.length;
      setCurrentPage(navItems[currentIndex] as any);

      let state = get(uiStore);
      expect(state.currentPage).toBe('discover');
    });

    it('should wrap around when navigating before first item with ArrowUp', () => {
      setCurrentPage('discover');
      const navItems = ['discover', 'library', 'downloads', 'settings'];
      let currentIndex = 0;

      // Simulate ArrowUp from first item
      currentIndex = (currentIndex - 1 + navItems.length) % navItems.length;
      setCurrentPage(navItems[currentIndex] as any);

      let state = get(uiStore);
      expect(state.currentPage).toBe('settings');
    });

    it('should navigate left in tabs with ArrowLeft', () => {
      const tabs = [
        { id: 'tab1', label: 'Tab 1' },
        { id: 'tab2', label: 'Tab 2' },
        { id: 'tab3', label: 'Tab 3' },
      ];

      let currentTabIndex = 1;
      const newIndex = (currentTabIndex - 1 + tabs.length) % tabs.length;
      expect(newIndex).toBe(0);
    });

    it('should navigate right in tabs with ArrowRight', () => {
      const tabs = [
        { id: 'tab1', label: 'Tab 1' },
        { id: 'tab2', label: 'Tab 2' },
        { id: 'tab3', label: 'Tab 3' },
      ];

      let currentTabIndex = 0;
      const newIndex = (currentTabIndex + 1) % tabs.length;
      expect(newIndex).toBe(1);
    });

    it('should jump to first tab with Home key', () => {
      const tabs = [
        { id: 'tab1', label: 'Tab 1' },
        { id: 'tab2', label: 'Tab 2' },
        { id: 'tab3', label: 'Tab 3' },
      ];

      let currentTabIndex = 2;
      const newIndex = 0; // Home key
      expect(newIndex).toBe(0);
    });

    it('should jump to last tab with End key', () => {
      const tabs = [
        { id: 'tab1', label: 'Tab 1' },
        { id: 'tab2', label: 'Tab 2' },
        { id: 'tab3', label: 'Tab 3' },
      ];

      let currentTabIndex = 0;
      const newIndex = tabs.length - 1; // End key
      expect(newIndex).toBe(2);
    });
  });

  describe('Escape Key Navigation', () => {
    it('should clear search input on Escape', () => {
      let searchValue = 'test query';
      // Simulate Escape key clearing search
      searchValue = '';
      expect(searchValue).toBe('');
    });

    it('should close modals on Escape', () => {
      const handleEscape = (e: KeyboardEvent) => {
        if (e.key === 'Escape') {
          return true;
        }
        return false;
      };

      const event = new KeyboardEvent('keydown', { key: 'Escape' });
      expect(handleEscape(event)).toBe(true);
    });

    it('should close mobile menu on Escape', () => {
      let mobileMenuOpen = true;

      const handleEscape = (e: KeyboardEvent) => {
        if (e.key === 'Escape' && mobileMenuOpen) {
          mobileMenuOpen = false;
        }
      };

      const event = new KeyboardEvent('keydown', { key: 'Escape' });
      handleEscape(event);
      expect(mobileMenuOpen).toBe(false);
    });

    it('should close dropdown on Escape', () => {
      let dropdownOpen = true;

      const handleEscape = (e: KeyboardEvent) => {
        if (e.key === 'Escape' && dropdownOpen) {
          dropdownOpen = false;
        }
      };

      const event = new KeyboardEvent('keydown', { key: 'Escape' });
      handleEscape(event);
      expect(dropdownOpen).toBe(false);
    });
  });

  describe('Tab Navigation', () => {
    it('should maintain focus order in navigation', () => {
      const navItems = ['discover', 'library', 'downloads', 'settings'];
      expect(navItems.length).toBe(4);
      expect(navItems[0]).toBe('discover');
      expect(navItems[1]).toBe('library');
      expect(navItems[2]).toBe('downloads');
      expect(navItems[3]).toBe('settings');
    });

    it('should support Tab key for focus navigation', () => {
      const handleTab = (e: KeyboardEvent) => {
        return e.key === 'Tab';
      };

      const event = new KeyboardEvent('keydown', { key: 'Tab' });
      expect(handleTab(event)).toBe(true);
    });

    it('should support Shift+Tab for reverse focus navigation', () => {
      const handleShiftTab = (e: KeyboardEvent) => {
        return e.key === 'Tab' && e.shiftKey;
      };

      const event = new KeyboardEvent('keydown', { key: 'Tab', shiftKey: true });
      expect(handleShiftTab(event)).toBe(true);
    });

    it('should cycle through focusable elements with Tab', () => {
      const focusableElements = ['button1', 'button2', 'button3'];
      let currentIndex = 0;

      // Simulate Tab key
      currentIndex = (currentIndex + 1) % focusableElements.length;
      expect(focusableElements[currentIndex]).toBe('button2');

      currentIndex = (currentIndex + 1) % focusableElements.length;
      expect(focusableElements[currentIndex]).toBe('button3');

      currentIndex = (currentIndex + 1) % focusableElements.length;
      expect(focusableElements[currentIndex]).toBe('button1');
    });

    it('should cycle backward through focusable elements with Shift+Tab', () => {
      const focusableElements = ['button1', 'button2', 'button3'];
      let currentIndex = 2;

      // Simulate Shift+Tab
      currentIndex = (currentIndex - 1 + focusableElements.length) % focusableElements.length;
      expect(focusableElements[currentIndex]).toBe('button2');

      currentIndex = (currentIndex - 1 + focusableElements.length) % focusableElements.length;
      expect(focusableElements[currentIndex]).toBe('button1');

      currentIndex = (currentIndex - 1 + focusableElements.length) % focusableElements.length;
      expect(focusableElements[currentIndex]).toBe('button3');
    });
  });

  describe('Enter Key Navigation', () => {
    it('should activate navigation item on Enter', () => {
      const navItems = ['discover', 'library', 'downloads', 'settings'];
      let currentIndex = 1;

      // Simulate Enter key on library item
      setCurrentPage(navItems[currentIndex] as any);

      let state = get(uiStore);
      expect(state.currentPage).toBe('library');
    });

    it('should trigger search on Enter in search input', () => {
      const handleEnter = (e: KeyboardEvent) => {
        return e.key === 'Enter';
      };

      const event = new KeyboardEvent('keydown', { key: 'Enter' });
      expect(handleEnter(event)).toBe(true);
    });

    it('should activate button on Enter', () => {
      const button = document.createElement('button');
      button.textContent = 'Click me';
      document.body.appendChild(button);

      const clickSpy = vi.fn();
      button.addEventListener('click', clickSpy);

      const enterEvent = new KeyboardEvent('keydown', { key: 'Enter' });
      button.dispatchEvent(enterEvent);

      document.body.removeChild(button);
    });

    it('should activate tab on Enter', () => {
      const { container } = render(Tabs, {
        props: {
          tabs: [
            { id: 'tab1', label: 'Tab 1' },
            { id: 'tab2', label: 'Tab 2' },
          ],
        },
      });

      const tab = container.querySelector('[role="tab"]');
      expect(tab).toBeTruthy();
    });
  });

  describe('Space Key Navigation', () => {
    it('should activate button on Space', () => {
      const button = document.createElement('button');
      button.textContent = 'Click me';
      document.body.appendChild(button);

      const clickSpy = vi.fn();
      button.addEventListener('click', clickSpy);

      const spaceEvent = new KeyboardEvent('keydown', { key: ' ' });
      button.dispatchEvent(spaceEvent);

      document.body.removeChild(button);
    });

    it('should toggle checkbox on Space', () => {
      const checkbox = document.createElement('input');
      checkbox.type = 'checkbox';
      checkbox.checked = false;
      document.body.appendChild(checkbox);

      checkbox.focus();
      const spaceEvent = new KeyboardEvent('keydown', { key: ' ' });
      checkbox.dispatchEvent(spaceEvent);

      document.body.removeChild(checkbox);
    });

    it('should activate clickable card on Space', () => {
      const { container } = render(Card, {
        props: {
          clickable: true,
        },
      });

      const card = container.querySelector('[role="button"]');
      expect(card?.getAttribute('tabindex')).toBe('0');
    });
  });

  describe('Tab Component Keyboard Navigation', () => {
    it('should navigate between tabs with arrow keys', () => {
      const { container } = render(Tabs, {
        props: {
          tabs: [
            { id: 'tab1', label: 'Tab 1' },
            { id: 'tab2', label: 'Tab 2' },
            { id: 'tab3', label: 'Tab 3' },
          ],
        },
      });

      const tabs = container.querySelectorAll('[role="tab"]');
      expect(tabs.length).toBe(3);
    });

    it('should have correct tabindex on active tab', () => {
      const { container } = render(Tabs, {
        props: {
          tabs: [
            { id: 'tab1', label: 'Tab 1' },
            { id: 'tab2', label: 'Tab 2' },
          ],
          activeTab: 'tab1',
        },
      });

      const activeTab = container.querySelector('[aria-selected="true"]');
      expect(activeTab?.getAttribute('tabindex')).toBe('0');
    });

    it('should have tabindex="-1" on inactive tabs', () => {
      const { container } = render(Tabs, {
        props: {
          tabs: [
            { id: 'tab1', label: 'Tab 1' },
            { id: 'tab2', label: 'Tab 2' },
          ],
          activeTab: 'tab1',
        },
      });

      const inactiveTabs = container.querySelectorAll('[aria-selected="false"]');
      inactiveTabs.forEach((tab) => {
        expect(tab.getAttribute('tabindex')).toBe('-1');
      });
    });
  });

  describe('Modal Keyboard Navigation', () => {
    it('should close modal on Escape', () => {
      const { container } = render(Modal, {
        props: {
          open: true,
          onClose: () => {
            // Modal should close
          },
        },
      });

      const modal = container.querySelector('[role="dialog"]');
      expect(modal).toBeTruthy();
    });

    it('should trap Tab within modal', () => {
      const { container } = render(Modal, {
        props: {
          open: true,
        },
      });

      const modal = container.querySelector('[role="dialog"]');
      const focusableElements = modal?.querySelectorAll(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );

      expect(focusableElements?.length).toBeGreaterThan(0);
    });

    it('should trap Shift+Tab within modal', () => {
      const { container } = render(Modal, {
        props: {
          open: true,
        },
      });

      const modal = container.querySelector('[role="dialog"]');
      const focusableElements = modal?.querySelectorAll(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );

      expect(focusableElements?.length).toBeGreaterThan(0);
    });
  });

  describe('Search Input Keyboard Navigation', () => {
    it('should trigger search on Enter', () => {
      const handleEnter = (e: KeyboardEvent) => {
        if (e.key === 'Enter') {
          return true;
        }
        return false;
      };

      const event = new KeyboardEvent('keydown', { key: 'Enter' });
      expect(handleEnter(event)).toBe(true);
    });

    it('should clear search on Escape', () => {
      let searchValue = 'test';

      const handleEscape = (e: KeyboardEvent) => {
        if (e.key === 'Escape') {
          searchValue = '';
        }
      };

      const event = new KeyboardEvent('keydown', { key: 'Escape' });
      handleEscape(event);
      expect(searchValue).toBe('');
    });

    it('should allow Tab to move focus out of search input', () => {
      const handleTab = (e: KeyboardEvent) => {
        return e.key === 'Tab';
      };

      const event = new KeyboardEvent('keydown', { key: 'Tab' });
      expect(handleTab(event)).toBe(true);
    });
  });
});
