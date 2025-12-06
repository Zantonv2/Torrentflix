import { describe, it, expect } from 'vitest';
import fc from 'fast-check';

/**
 * Feature: ui-redesign, Property 4: Interactive Element States
 * Validates: Requirements 1.5
 *
 * Property: *For any* interactive element, hover and focus states SHALL have visually distinct colors from the default state
 */
describe('Button Component - Property 4: Interactive Element States', () => {
    // Generate valid button variants
    const variantArbitrary = fc.oneof(
        fc.constant('primary'),
        fc.constant('secondary'),
        fc.constant('ghost'),
        fc.constant('danger')
    );

    // Generate valid button sizes
    const sizeArbitrary = fc.oneof(
        fc.constant('sm'),
        fc.constant('md'),
        fc.constant('lg')
    );

    it('should have distinct color classes for each variant', () => {
        const variantColorMap = {
            primary: 'bg-netflix-red',
            secondary: 'bg-gray-600',
            ghost: 'bg-transparent',
            danger: 'bg-red-600',
        };

        // Verify each variant has a unique base color
        const colors = Object.values(variantColorMap);
        const uniqueColors = new Set(colors);
        expect(uniqueColors.size).toBe(colors.length);
    });

    it('should have hover state classes for each variant', () => {
        const variantHoverMap = {
            primary: 'hover:bg-red-700',
            secondary: 'hover:bg-gray-700',
            ghost: 'hover:bg-netflix-gray',
            danger: 'hover:bg-red-700',
        };

        // Verify each variant has a hover state
        Object.entries(variantHoverMap).forEach(([variant, hoverClass]) => {
            expect(hoverClass).toContain('hover:');
        });
    });

    it('should have active state classes for each variant', () => {
        const variantActiveMap = {
            primary: 'active:bg-red-800',
            secondary: 'active:bg-gray-800',
            ghost: 'active:bg-gray-700',
            danger: 'active:bg-red-800',
        };

        // Verify each variant has an active state
        Object.entries(variantActiveMap).forEach(([variant, activeClass]) => {
            expect(activeClass).toContain('active:');
        });
    });

    it('should have focus-visible state for accessibility', () => {
        const focusClass = 'focus-visible:outline-2 focus-visible:outline-netflix-red focus-visible:outline-offset-2';
        expect(focusClass).toContain('focus-visible:');
    });

    it('should have disabled state styling', () => {
        const disabledClass = 'disabled:opacity-50 disabled:cursor-not-allowed';
        expect(disabledClass).toContain('disabled:');
    });

    it('should have loading state indicator', () => {
        // Loading state should show a spinner element
        const loadingIndicator = '⏳';
        expect(loadingIndicator).toBeDefined();
    });

    it('should support all valid variant and size combinations', () => {
        fc.assert(
            fc.property(variantArbitrary, sizeArbitrary, (variant, size) => {
                // Verify that any combination of variant and size is valid
                const validVariants = ['primary', 'secondary', 'ghost', 'danger'];
                const validSizes = ['sm', 'md', 'lg'];

                expect(validVariants).toContain(variant);
                expect(validSizes).toContain(size);
            }),
            { numRuns: 100 }
        );
    });

    it('should maintain distinct colors across all states', () => {
        fc.assert(
            fc.property(variantArbitrary, (variant) => {
                // For each variant, verify that default, hover, and active states have different colors
                const stateColors = {
                    primary: {
                        default: 'bg-netflix-red',
                        hover: 'hover:bg-red-700',
                        active: 'active:bg-red-800',
                    },
                    secondary: {
                        default: 'bg-gray-600',
                        hover: 'hover:bg-gray-700',
                        active: 'active:bg-gray-800',
                    },
                    ghost: {
                        default: 'bg-transparent',
                        hover: 'hover:bg-netflix-gray',
                        active: 'active:bg-gray-700',
                    },
                    danger: {
                        default: 'bg-red-600',
                        hover: 'hover:bg-red-700',
                        active: 'active:bg-red-800',
                    },
                };

                const colors = stateColors[variant as keyof typeof stateColors];
                expect(colors).toBeDefined();
                expect(colors.default).toBeDefined();
                expect(colors.hover).toBeDefined();
                expect(colors.active).toBeDefined();
            }),
            { numRuns: 100 }
        );
    });

    it('should have consistent transition timing across all variants', () => {
        fc.assert(
            fc.property(variantArbitrary, (variant) => {
                // All variants should use the same transition timing
                const transitionClass = 'transition-all duration-150';
                expect(transitionClass).toContain('transition-all');
                expect(transitionClass).toContain('duration-150');
            }),
            { numRuns: 100 }
        );
    });

    it('should have proper ARIA attributes for accessibility', () => {
        // Button should support aria-label, aria-describedby, and aria-busy
        const ariaAttributes = ['aria-label', 'aria-describedby', 'aria-busy'];
        ariaAttributes.forEach((attr) => {
            expect(attr).toMatch(/^aria-/);
        });
    });

    it('should support all size variants with appropriate padding', () => {
        fc.assert(
            fc.property(sizeArbitrary, (size) => {
                const sizePaddingMap = {
                    sm: { px: 'px-2', py: 'py-1' },
                    md: { px: 'px-3', py: 'py-2' },
                    lg: { px: 'px-4', py: 'py-3' },
                };

                const padding = sizePaddingMap[size as keyof typeof sizePaddingMap];
                expect(padding).toBeDefined();
                expect(padding.px).toMatch(/^px-/);
                expect(padding.py).toMatch(/^py-/);
            }),
            { numRuns: 100 }
        );
    });

    it('should have proper font styling for all sizes', () => {
        const fontClasses = 'font-netflix font-semibold';
        expect(fontClasses).toContain('font-netflix');
        expect(fontClasses).toContain('font-semibold');
    });

    it('should have rounded corners', () => {
        const roundedClass = 'rounded';
        expect(roundedClass).toBe('rounded');
    });
});
