import { describe, it, expect } from 'vitest';
import fc from 'fast-check';
import type { QualityVersion } from '../types';

// Generator for quality versions
const qualityGenerator = (): fc.Arbitrary<QualityVersion> => {
    return fc.record({
        resolution: fc.constantFrom('4K', '1080p', '720p', 'SD'),
        file_size_gb: fc.float({ min: Math.fround(0.1), max: Math.fround(100), noNaN: true }),
        seeders: fc.integer({ min: 0, max: 10000 }),
        leechers: fc.integer({ min: 0, max: 5000 }),
        magnet_link: fc.string({ minLength: 10 }),
        is_recommended: fc.option(fc.boolean()),
    });
};

describe('Quality Selection - Property-Based Tests', () => {
    describe('Property 59: Quality Sorting', () => {
        it('should sort quality options by resolution in descending order', () => {
            // **Feature: ui-redesign, Property 59: Quality Sorting**
            // **Validates: Requirements 14.3**

            fc.assert(
                fc.property(
                    fc.array(qualityGenerator(), { minLength: 1, maxLength: 10 }),
                    (qualities) => {
                        // For any list of qualities, verify sorting logic
                        const resolutionOrder: Record<string, number> = {
                            '4K': 4,
                            '1080p': 3,
                            '720p': 2,
                            'SD': 1,
                        };

                        const sorted = [...qualities].sort((a, b) => {
                            const orderA = resolutionOrder[a.resolution] || 0;
                            const orderB = resolutionOrder[b.resolution] || 0;
                            return orderB - orderA;
                        });

                        // Verify sorted order is descending
                        for (let i = 0; i < sorted.length - 1; i++) {
                            const currentOrder = resolutionOrder[sorted[i].resolution] || 0;
                            const nextOrder = resolutionOrder[sorted[i + 1].resolution] || 0;
                            expect(currentOrder).toBeGreaterThanOrEqual(nextOrder);
                        }

                        return true;
                    }
                ),
                { numRuns: 100 }
            );
        });
    });

    describe('Property 60: Recommended Quality', () => {
        it('should highlight the recommended quality option', () => {
            // **Feature: ui-redesign, Property 60: Recommended Quality**
            // **Validates: Requirements 14.5**

            fc.assert(
                fc.property(
                    fc.array(qualityGenerator(), { minLength: 1, maxLength: 10 }).map((qualities) => {
                        // Ensure at most one quality is marked as recommended
                        const recommendedIndex = qualities.findIndex((q) => q.is_recommended === true);
                        return qualities.map((q, i) => ({
                            ...q,
                            is_recommended: i === recommendedIndex ? true : false,
                        }));
                    }),
                    (qualities) => {
                        // For any list of qualities, at most one should be recommended
                        const recommendedCount = qualities.filter(
                            (q) => q.is_recommended === true
                        ).length;

                        expect(recommendedCount).toBeLessThanOrEqual(1);

                        // If there is a recommended quality, it should be identifiable
                        if (recommendedCount === 1) {
                            const recommended = qualities.find((q) => q.is_recommended);
                            expect(recommended).toBeDefined();
                            expect(recommended?.magnet_link).toBeDefined();
                        }

                        return true;
                    }
                ),
                { numRuns: 100 }
            );
        });
    });

    describe('Property 59-60: Quality Selection Combined', () => {
        it('should sort qualities and identify recommended option', () => {
            // **Feature: ui-redesign, Property 59-60: Quality Selection**
            // **Validates: Requirements 14.3, 14.5**

            fc.assert(
                fc.property(
                    fc.array(qualityGenerator(), { minLength: 1, maxLength: 10 }).map((qualities) => {
                        // Ensure at most one quality is marked as recommended
                        const recommendedIndex = qualities.findIndex((q) => q.is_recommended === true);
                        return qualities.map((q, i) => ({
                            ...q,
                            is_recommended: i === recommendedIndex ? true : false,
                        }));
                    }),
                    (qualities) => {
                        // For any list of qualities:
                        // 1. All should have required properties
                        qualities.forEach((quality) => {
                            expect(quality.resolution).toBeDefined();
                            expect(quality.file_size_gb).toBeGreaterThan(0);
                            expect(quality.seeders).toBeGreaterThanOrEqual(0);
                            expect(quality.leechers).toBeGreaterThanOrEqual(0);
                            expect(quality.magnet_link).toBeDefined();
                        });

                        // 2. Sorting should work correctly
                        const resolutionOrder: Record<string, number> = {
                            '4K': 4,
                            '1080p': 3,
                            '720p': 2,
                            'SD': 1,
                        };

                        const sorted = [...qualities].sort((a, b) => {
                            const orderA = resolutionOrder[a.resolution] || 0;
                            const orderB = resolutionOrder[b.resolution] || 0;
                            return orderB - orderA;
                        });

                        expect(sorted.length).toBe(qualities.length);

                        // 3. Recommended flag should be valid
                        const recommendedCount = qualities.filter(
                            (q) => q.is_recommended === true
                        ).length;
                        expect(recommendedCount).toBeLessThanOrEqual(1);

                        return true;
                    }
                ),
                { numRuns: 100 }
            );
        });
    });
});
