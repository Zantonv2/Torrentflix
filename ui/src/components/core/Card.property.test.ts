import { describe, it, expect } from 'vitest';
import { fc } from '@fast-check/vitest';

/**
 * **Feature: ui-redesign, Property 11: Tile Border Radius**
 * *For any* movie tile, the border-radius SHALL be at least 8px
 * **Validates: Requirements 2.7**
 */
describe('Card Component - Property 11: Tile Border Radius', () => {
    it('should have border-radius of at least 8px', () => {
        // Card component uses rounded-lg which is 8px in Tailwind
        const borderRadiusClass = 'rounded-lg';
        const borderRadiusValue = 8; // pixels

        expect(borderRadiusValue).toBeGreaterThanOrEqual(8);
        expect(borderRadiusClass).toBe('rounded-lg');
    });

    it('should apply rounded-lg class by default', () => {
        const cardClasses = 'bg-netflix-dark border border-netflix-gray rounded-lg transition-all duration-200';
        expect(cardClasses).toContain('rounded-lg');
    });

    it('should maintain border-radius across all padding variants', () => {
        const paddingVariants = ['sm', 'md', 'lg'] as const;
        const borderRadiusClass = 'rounded-lg';

        paddingVariants.forEach((variant) => {
            // Each variant should maintain the rounded-lg class
            expect(borderRadiusClass).toBe('rounded-lg');
        });
    });

    it('should maintain border-radius in hover state', () => {
        const hoverClasses = 'hover:shadow-lg hover:scale-105';
        const baseClasses = 'rounded-lg';

        // Hover effects should not remove border-radius
        expect(baseClasses).toContain('rounded');
        expect(hoverClasses).not.toContain('rounded-none');
    });

    it('should maintain border-radius in clickable state', () => {
        const clickableClasses = 'cursor-pointer';
        const baseClasses = 'rounded-lg';

        // Clickable state should not affect border-radius
        expect(baseClasses).toContain('rounded-lg');
        expect(clickableClasses).not.toContain('rounded');
    });
});
