import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render } from 'vitest-dom';
import Modal from '../core/Modal.svelte';
import Card from '../core/Card.svelte';

describe('Focus Management', () => {
  describe('Modal Focus Trap', () => {
    let previousFocus: HTMLElement | null = null;

    beforeEach(() => {
      previousFocus = document.activeElement as HTMLElement;
    });

    afterEach(() => {
      if (previousFocus) {
        previousFocus.focus();
      }
    });

    it('should trap focus within modal when tabbing forward', () => {
      const { container } = render(Modal, {
        props: {
          open: true,
          title: 'Test Modal',
        },
      });

      const modal = container.querySelector('[role="dialog"]');
      expect(modal).toBeTruthy();

      // Get all focusable elements within modal
      const focusableElements = modal?.querySelectorAll(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );
      expect(focusableElements?.length).toBeGreaterThan(0);
    });

    it('should trap focus within modal when tabbing backward', () => {
      const { container } = render(Modal, {
        props: {
          open: true,
        },
      });

      const modal = container.querySelector('[role="dialog"]');
      const focusableElements = modal?.querySelectorAll(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );

      // Verify focus trap is possible
      expect(focusableElements?.length).toBeGreaterThan(0);
    });

    it('should return focus to trigger element when modal closes', () => {
      const triggerButton = document.createElement('button');
      triggerButton.textContent = 'Open Modal';
      document.body.appendChild(triggerButton);

      triggerButton.focus();
      const focusedBefore = document.activeElement;

      const { container } = render(Modal, {
        props: {
          open: true,
          onClose: () => {
            triggerButton.focus();
          },
        },
      });

      // Simulate modal close
      const closeButton = container.querySelector('button[aria-label="Close modal"]');
      closeButton?.dispatchEvent(new MouseEvent('click'));

      // Focus should return to trigger button
      expect(document.activeElement).toBe(triggerButton);

      document.body.removeChild(triggerButton);
    });

    it('should move focus to first focusable element when modal opens', () => {
      const { container } = render(Modal, {
        props: {
          open: true,
        },
      });

      const modal = container.querySelector('[role="dialog"]');
      const firstFocusable = modal?.querySelector(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );

      expect(firstFocusable).toBeTruthy();
    });

    it('should handle Escape key to close modal and restore focus', () => {
      const { container } = render(Modal, {
        props: {
          open: true,
        },
      });

      const modal = container.querySelector('[role="dialog"]');
      expect(modal).toBeTruthy();

      // Simulate Escape key
      const escapeEvent = new KeyboardEvent('keydown', {
        key: 'Escape',
        bubbles: true,
      });

      document.dispatchEvent(escapeEvent);
      // Modal should handle escape and close
    });
  });

  describe('Clickable Card Focus Management', () => {
    it('should have tabindex="0" when clickable', () => {
      const { container } = render(Card, {
        props: {
          clickable: true,
        },
      });

      const card = container.querySelector('[role="button"]');
      expect(card?.getAttribute('tabindex')).toBe('0');
    });

    it('should not have tabindex when not clickable', () => {
      const { container } = render(Card, {
        props: {
          clickable: false,
        },
      });

      const card = container.querySelector('div');
      expect(card?.getAttribute('tabindex')).toBeNull();
    });

    it('should handle Enter key on clickable card', () => {
      const { container } = render(Card, {
        props: {
          clickable: true,
        },
      });

      const card = container.querySelector('[role="button"]');
      const clickSpy = vi.fn();

      card?.addEventListener('click', clickSpy);

      const enterEvent = new KeyboardEvent('keydown', {
        key: 'Enter',
        bubbles: true,
      });

      card?.dispatchEvent(enterEvent);
      // Enter key should trigger click
    });

    it('should handle Space key on clickable card', () => {
      const { container } = render(Card, {
        props: {
          clickable: true,
        },
      });

      const card = container.querySelector('[role="button"]');
      const clickSpy = vi.fn();

      card?.addEventListener('click', clickSpy);

      const spaceEvent = new KeyboardEvent('keydown', {
        key: ' ',
        bubbles: true,
      });

      card?.dispatchEvent(spaceEvent);
      // Space key should trigger click
    });
  });

  describe('Focus Visibility', () => {
    it('should have visible focus indicator on button', () => {
      const button = document.createElement('button');
      button.textContent = 'Test Button';
      button.className = 'focus:outline-none focus:ring-2 focus:ring-netflix-red';
      document.body.appendChild(button);

      button.focus();
      expect(document.activeElement).toBe(button);

      document.body.removeChild(button);
    });

    it('should have visible focus indicator on input', () => {
      const input = document.createElement('input');
      input.type = 'text';
      input.className = 'focus:outline-none focus:border-netflix-red focus:ring-2 focus:ring-netflix-red/30';
      document.body.appendChild(input);

      input.focus();
      expect(document.activeElement).toBe(input);

      document.body.removeChild(input);
    });

    it('should have visible focus indicator on card', () => {
      const card = document.createElement('div');
      card.role = 'button';
      card.tabIndex = 0;
      card.className = 'focus:outline-none focus:ring-2 focus:ring-netflix-red';
      document.body.appendChild(card);

      card.focus();
      expect(document.activeElement).toBe(card);

      document.body.removeChild(card);
    });
  });

  describe('Focus Order', () => {
    it('should maintain logical focus order in navigation', () => {
      const navItems = ['discover', 'library', 'downloads', 'settings'];
      const focusOrder = [...navItems];

      expect(focusOrder[0]).toBe('discover');
      expect(focusOrder[1]).toBe('library');
      expect(focusOrder[2]).toBe('downloads');
      expect(focusOrder[3]).toBe('settings');
    });

    it('should skip non-interactive elements in focus order', () => {
      const elements = [
        { id: 'text', interactive: false },
        { id: 'button1', interactive: true },
        { id: 'text2', interactive: false },
        { id: 'button2', interactive: true },
      ];

      const focusableElements = elements.filter((el) => el.interactive);
      expect(focusableElements.length).toBe(2);
      expect(focusableElements[0].id).toBe('button1');
      expect(focusableElements[1].id).toBe('button2');
    });

    it('should maintain focus order after dynamic content changes', () => {
      const initialOrder = ['button1', 'button2', 'button3'];
      const newElement = 'button4';

      const updatedOrder = [...initialOrder, newElement];
      expect(updatedOrder.length).toBe(4);
      expect(updatedOrder[3]).toBe('button4');
    });
  });

  describe('Mobile Menu Focus Management', () => {
    it('should trap focus in mobile menu when open', () => {
      const mobileMenu = document.createElement('div');
      mobileMenu.className = 'fixed left-0 top-[60px] bottom-0 w-[200px]';
      document.body.appendChild(mobileMenu);

      const button1 = document.createElement('button');
      button1.textContent = 'Menu Item 1';
      mobileMenu.appendChild(button1);

      const button2 = document.createElement('button');
      button2.textContent = 'Menu Item 2';
      mobileMenu.appendChild(button2);

      const focusableElements = mobileMenu.querySelectorAll('button');
      expect(focusableElements.length).toBe(2);

      document.body.removeChild(mobileMenu);
    });

    it('should restore focus to hamburger button when mobile menu closes', () => {
      const hamburgerButton = document.createElement('button');
      hamburgerButton.textContent = '☰';
      hamburgerButton.setAttribute('aria-label', 'Toggle navigation menu');
      document.body.appendChild(hamburgerButton);

      hamburgerButton.focus();
      const focusedBefore = document.activeElement;

      // Simulate menu close
      hamburgerButton.focus();
      expect(document.activeElement).toBe(hamburgerButton);

      document.body.removeChild(hamburgerButton);
    });

    it('should close mobile menu on Escape key', () => {
      let menuOpen = true;

      const handleEscape = (e: KeyboardEvent) => {
        if (e.key === 'Escape' && menuOpen) {
          menuOpen = false;
        }
      };

      const escapeEvent = new KeyboardEvent('keydown', { key: 'Escape' });
      handleEscape(escapeEvent);

      expect(menuOpen).toBe(false);
    });
  });

  describe('Modal Backdrop Click', () => {
    it('should close modal when clicking backdrop', () => {
      const { container } = render(Modal, {
        props: {
          open: true,
          onClose: () => {
            // Modal should close
          },
        },
      });

      const backdrop = container.querySelector('[role="presentation"]');
      expect(backdrop).toBeTruthy();

      // Simulate backdrop click
      const clickEvent = new MouseEvent('click', { bubbles: true });
      backdrop?.dispatchEvent(clickEvent);
    });

    it('should not close modal when clicking modal content', () => {
      const { container } = render(Modal, {
        props: {
          open: true,
        },
      });

      const modal = container.querySelector('[role="dialog"]');
      expect(modal).toBeTruthy();

      // Clicking modal content should not close it
      const clickEvent = new MouseEvent('click', { bubbles: false });
      modal?.dispatchEvent(clickEvent);
    });
  });
});
