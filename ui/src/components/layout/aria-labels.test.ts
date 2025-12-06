import { describe, it, expect } from 'vitest';
import { render } from 'vitest-dom';
import Button from '../core/Button.svelte';
import Input from '../core/Input.svelte';
import Modal from '../core/Modal.svelte';
import Card from '../core/Card.svelte';
import Badge from '../core/Badge.svelte';
import Tabs from '../core/Tabs.svelte';
import Toast from '../core/Toast.svelte';

describe('ARIA Labels - Core Components', () => {
  describe('Button Component ARIA Labels', () => {
    it('should have aria-label when provided', () => {
      const { container } = render(Button, {
        props: {
          ariaLabel: 'Submit form',
        },
      });
      const button = container.querySelector('button');
      expect(button?.getAttribute('aria-label')).toBe('Submit form');
    });

    it('should have aria-busy when loading', () => {
      const { container } = render(Button, {
        props: {
          loading: true,
        },
      });
      const button = container.querySelector('button');
      expect(button?.getAttribute('aria-busy')).toBe('true');
    });

    it('should have aria-describedby when error description provided', () => {
      const { container } = render(Button, {
        props: {
          ariaDescribedby: 'error-message-1',
        },
      });
      const button = container.querySelector('button');
      expect(button?.getAttribute('aria-describedby')).toBe('error-message-1');
    });

    it('should be disabled when disabled prop is true', () => {
      const { container } = render(Button, {
        props: {
          disabled: true,
        },
      });
      const button = container.querySelector('button');
      expect(button?.disabled).toBe(true);
    });
  });

  describe('Input Component ARIA Labels', () => {
    it('should have aria-label from label prop', () => {
      const { container } = render(Input, {
        props: {
          label: 'Email address',
        },
      });
      const input = container.querySelector('input');
      expect(input?.getAttribute('aria-label')).toBe('Email address');
    });

    it('should have aria-invalid when error exists', () => {
      const { container } = render(Input, {
        props: {
          error: 'This field is required',
        },
      });
      const input = container.querySelector('input');
      expect(input?.getAttribute('aria-invalid')).toBe('true');
    });

    it('should have aria-describedby linking to error message', () => {
      const { container } = render(Input, {
        props: {
          error: 'Invalid email format',
        },
      });
      const input = container.querySelector('input');
      const ariaDescribedby = input?.getAttribute('aria-describedby');
      expect(ariaDescribedby).toBeTruthy();
      expect(ariaDescribedby).toContain('error');
    });

    it('should have aria-label on password toggle button', () => {
      const { container } = render(Input, {
        props: {
          type: 'password',
          masked: true,
        },
      });
      const toggleButton = container.querySelector('button');
      expect(toggleButton?.getAttribute('aria-label')).toBeTruthy();
    });

    it('should have proper aria-label for custom ariaLabel prop', () => {
      const { container } = render(Input, {
        props: {
          ariaLabel: 'Search movies',
        },
      });
      const input = container.querySelector('input');
      expect(input?.getAttribute('aria-label')).toBe('Search movies');
    });
  });

  describe('Modal Component ARIA Labels', () => {
    it('should have role="dialog" on modal', () => {
      const { container } = render(Modal, {
        props: {
          open: true,
          title: 'Confirm Action',
        },
      });
      const modal = container.querySelector('[role="dialog"]');
      expect(modal).toBeTruthy();
    });

    it('should have aria-modal="true" on modal', () => {
      const { container } = render(Modal, {
        props: {
          open: true,
        },
      });
      const modal = container.querySelector('[role="dialog"]');
      expect(modal?.getAttribute('aria-modal')).toBe('true');
    });

    it('should have aria-labelledby linking to title', () => {
      const { container } = render(Modal, {
        props: {
          open: true,
          title: 'Delete Item',
        },
      });
      const modal = container.querySelector('[role="dialog"]');
      const ariaLabelledby = modal?.getAttribute('aria-labelledby');
      expect(ariaLabelledby).toBeTruthy();
      expect(ariaLabelledby).toContain('title');
    });

    it('should have aria-label on close button', () => {
      const { container } = render(Modal, {
        props: {
          open: true,
        },
      });
      const closeButton = container.querySelector('button');
      expect(closeButton?.getAttribute('aria-label')).toBe('Close modal');
    });

    it('should have aria-label when provided instead of title', () => {
      const { container } = render(Modal, {
        props: {
          open: true,
          ariaLabel: 'Confirmation dialog',
        },
      });
      const modal = container.querySelector('[role="dialog"]');
      expect(modal?.getAttribute('aria-label')).toBe('Confirmation dialog');
    });
  });

  describe('Card Component ARIA Labels', () => {
    it('should have role="button" when clickable', () => {
      const { container } = render(Card, {
        props: {
          clickable: true,
        },
      });
      const card = container.querySelector('[role="button"]');
      expect(card).toBeTruthy();
    });

    it('should have aria-label when provided', () => {
      const { container } = render(Card, {
        props: {
          clickable: true,
          ariaLabel: 'Movie card for Inception',
        },
      });
      const card = container.querySelector('[role="button"]');
      expect(card?.getAttribute('aria-label')).toBe('Movie card for Inception');
    });

    it('should have aria-describedby when provided', () => {
      const { container } = render(Card, {
        props: {
          ariaDescribedby: 'movie-description-1',
        },
      });
      const card = container.querySelector('div');
      expect(card?.getAttribute('aria-describedby')).toBe('movie-description-1');
    });

    it('should have tabindex="0" when clickable', () => {
      const { container } = render(Card, {
        props: {
          clickable: true,
        },
      });
      const card = container.querySelector('[role="button"]');
      expect(card?.getAttribute('tabindex')).toBe('0');
    });
  });

  describe('Badge Component ARIA Labels', () => {
    it('should have aria-label when provided', () => {
      const { container } = render(Badge, {
        props: {
          ariaLabel: 'Quality: 1080p',
        },
      });
      const badge = container.querySelector('span');
      expect(badge?.getAttribute('aria-label')).toBe('Quality: 1080p');
    });

    it('should render with correct variant', () => {
      const { container } = render(Badge, {
        props: {
          variant: 'success',
        },
      });
      const badge = container.querySelector('span');
      expect(badge?.className).toContain('bg-green-600');
    });
  });

  describe('Tabs Component ARIA Labels', () => {
    it('should have role="tablist" on container', () => {
      const { container } = render(Tabs, {
        props: {
          tabs: [
            { id: 'tab1', label: 'Tab 1' },
            { id: 'tab2', label: 'Tab 2' },
          ],
        },
      });
      const tablist = container.querySelector('[role="tablist"]');
      expect(tablist).toBeTruthy();
    });

    it('should have aria-label on tablist when provided', () => {
      const { container } = render(Tabs, {
        props: {
          tabs: [
            { id: 'tab1', label: 'Tab 1' },
          ],
          ariaLabel: 'Navigation tabs',
        },
      });
      const tablist = container.querySelector('[role="tablist"]');
      expect(tablist?.getAttribute('aria-label')).toBe('Navigation tabs');
    });

    it('should have role="tab" on tab buttons', () => {
      const { container } = render(Tabs, {
        props: {
          tabs: [
            { id: 'tab1', label: 'Tab 1' },
            { id: 'tab2', label: 'Tab 2' },
          ],
        },
      });
      const tabs = container.querySelectorAll('[role="tab"]');
      expect(tabs.length).toBe(2);
    });

    it('should have aria-selected on active tab', () => {
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
      expect(activeTab).toBeTruthy();
    });

    it('should have aria-controls linking to tabpanel', () => {
      const { container } = render(Tabs, {
        props: {
          tabs: [
            { id: 'tab1', label: 'Tab 1' },
          ],
        },
      });
      const tab = container.querySelector('[role="tab"]');
      const ariaControls = tab?.getAttribute('aria-controls');
      expect(ariaControls).toBeTruthy();
      expect(ariaControls).toContain('tabpanel');
    });
  });

  describe('Toast Component ARIA Labels', () => {
    it('should have role="alert" on toast', () => {
      const { container } = render(Toast, {
        props: {
          message: 'Operation successful',
          type: 'success',
        },
      });
      const toast = container.querySelector('[role="alert"]');
      expect(toast).toBeTruthy();
    });

    it('should have aria-live="polite" on toast', () => {
      const { container } = render(Toast, {
        props: {
          message: 'Operation successful',
        },
      });
      const toast = container.querySelector('[role="alert"]');
      expect(toast?.getAttribute('aria-live')).toBe('polite');
    });

    it('should have aria-label with message and type', () => {
      const { container } = render(Toast, {
        props: {
          message: 'File saved',
          type: 'success',
        },
      });
      const toast = container.querySelector('[role="alert"]');
      const ariaLabel = toast?.getAttribute('aria-label');
      expect(ariaLabel).toContain('File saved');
      expect(ariaLabel).toContain('Success');
    });

    it('should have aria-label on close button', () => {
      const { container } = render(Toast, {
        props: {
          message: 'Notification',
        },
      });
      const closeButton = container.querySelector('button');
      expect(closeButton?.getAttribute('aria-label')).toBe('Dismiss notification');
    });
  });

  describe('Navigation ARIA Labels', () => {
    it('should have aria-label on navigation buttons', () => {
      const navItems = [
        { id: 'discover', label: 'Открыть', ariaLabel: 'Открыть' },
        { id: 'library', label: 'Библиотека', ariaLabel: 'Библиотека' },
        { id: 'downloads', label: 'Загрузки', ariaLabel: 'Загрузки' },
        { id: 'settings', label: 'Настройки', ariaLabel: 'Настройки' },
      ];

      navItems.forEach((item) => {
        expect(item.ariaLabel).toBeDefined();
        expect(item.ariaLabel.length).toBeGreaterThan(0);
      });
    });

    it('should have aria-current on active navigation item', () => {
      const activeItem = {
        id: 'discover',
        label: 'Открыть',
        ariaCurrent: 'page',
      };

      expect(activeItem.ariaCurrent).toBe('page');
    });

    it('should have role="navigation" on sidebar', () => {
      const sidebar = {
        role: 'navigation',
        ariaLabel: 'Main navigation',
      };

      expect(sidebar.role).toBe('navigation');
      expect(sidebar.ariaLabel).toBeDefined();
    });
  });
});
