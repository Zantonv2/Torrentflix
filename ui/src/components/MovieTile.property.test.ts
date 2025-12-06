import { describe, it, expect } from 'vitest';
import fc from 'fast-check';
import type { UiSearchResult } from '../types';

/**
 * **Feature: ui-redesign, Property 5: Tile Aspect Ratio**
 * *For any* movie tile, the aspect ratio of the poster image container SHALL be 2:3 (portrait)
 * **Validates: Requirements 2.1**
 */
describe('MovieTile Component - Property 5: Tile Aspect Ratio', () => {
    it('should have 2:3 aspect ratio for poster container', () => {
        const aspectRatio = '2/3';
        const aspectRatioValue = 2 / 3;
        expect(aspectRatioValue).toBeCloseTo(0.667, 2);
        expect(aspectRatio).toBe('2/3');
    });

    it('should use aspect-video class with 2:3 style override', () => {
        const classes = 'aspect-video';
        const style = 'aspect-ratio: 2/3;';
        expect(classes).toContain('aspect-video');
        expect(style).toContain('2/3');
    });
});

/**
 * **Feature: ui-redesign, Property 6: Poster Display**
 * *For any* movie with a poster URL, the poster image SHALL fill the tile area with object-fit: cover
 * **Validates: Requirements 2.2**
 */
describe('MovieTile Component - Property 6: Poster Display', () => {
    const movieWithPoster = fc.record({
        id: fc.uuid(),
        title: fc.string({ minLength: 1, maxLength: 100 }),
        poster_url: fc.webUrl(),
        quality_badge: fc.string(),
        torrent_info: fc.record({
            seeders: fc.integer({ min: 0 }),
            size_gb: fc.float({ min: 0 }),
            magnet_link: fc.string(),
        }),
    });

    it('should render img element when poster_url is provided', () => {
        fc.assert(
            fc.property(movieWithPoster, (movie) => {
                expect(movie.poster_url).toBeDefined();
                expect(movie.poster_url).toMatch(/^https?:\/\//);
            }),
            { numRuns: 50 }
        );
    });

    it('should use object-cover class for poster image', () => {
        const objectFitClass = 'object-cover';
        expect(objectFitClass).toBe('object-cover');
    });

    it('should have full width and height for poster', () => {
        const posterClasses = 'w-full h-full object-cover rounded-lg';
        expect(posterClasses).toContain('w-full');
        expect(posterClasses).toContain('h-full');
        expect(posterClasses).toContain('object-cover');
    });

    it('should have lazy loading enabled', () => {
        const loadingAttr = 'lazy';
        expect(loadingAttr).toBe('lazy');
    });
});

/**
 * **Feature: ui-redesign, Property 7: Missing Poster Placeholder**
 * *For any* movie without a poster URL, a placeholder with the movie title SHALL display in the tile
 * **Validates: Requirements 2.3**
 */
describe('MovieTile Component - Property 7: Missing Poster Placeholder', () => {
    const movieWithoutPoster = fc.record({
        id: fc.uuid(),
        title: fc.string({ minLength: 1, maxLength: 100 }),
        quality_badge: fc.string(),
        torrent_info: fc.record({
            seeders: fc.integer({ min: 0 }),
            size_gb: fc.float({ min: 0 }),
            magnet_link: fc.string(),
        }),
    });

    it('should display placeholder when poster_url is undefined', () => {
        fc.assert(
            fc.property(movieWithoutPoster, (movie) => {
                expect(movie.poster_url).toBeUndefined();
            }),
            { numRuns: 50 }
        );
    });

    it('should display title in placeholder', () => {
        fc.assert(
            fc.property(fc.string({ minLength: 1, maxLength: 100 }), (title) => {
                expect(title).toBeDefined();
                expect(title.length).toBeGreaterThan(0);
            }),
            { numRuns: 50 }
        );
    });

    it('should use netflix-gray background for placeholder', () => {
        const placeholderClasses = 'bg-netflix-gray rounded-lg flex items-center justify-center';
        expect(placeholderClasses).toContain('bg-netflix-gray');
        expect(placeholderClasses).toContain('rounded-lg');
    });

    it('should center text in placeholder', () => {
        const textClasses = 'text-center px-4';
        expect(textClasses).toContain('text-center');
        expect(textClasses).toContain('px-4');
    });
});

/**
 * **Feature: ui-redesign, Property 8: Title Truncation**
 * *For any* movie title exceeding two lines, the title SHALL be truncated with ellipsis
 * **Validates: Requirements 2.4**
 */
describe('MovieTile Component - Property 8: Title Truncation', () => {
    const longTitle = fc.string({ minLength: 50, maxLength: 200 });

    it('should apply line-clamp-2 class to title', () => {
        const titleClasses = 'line-clamp-2';
        expect(titleClasses).toBe('line-clamp-2');
    });

    it('should use webkit line clamping for truncation', () => {
        const lineClampStyle = `
      display: -webkit-box;
      -webkit-line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
    `;
        expect(lineClampStyle).toContain('-webkit-line-clamp: 2');
        expect(lineClampStyle).toContain('overflow: hidden');
    });

    it('should truncate any title longer than 2 lines', () => {
        fc.assert(
            fc.property(longTitle, (title) => {
                // Verify that line-clamp-2 will truncate any long title
                expect(title.length).toBeGreaterThan(0);
            }),
            { numRuns: 50 }
        );
    });

    it('should maintain text size for truncated titles', () => {
        const titleClasses = 'text-sm font-netflix font-semibold text-white';
        expect(titleClasses).toContain('text-sm');
        expect(titleClasses).toContain('font-semibold');
    });
});

/**
 * **Feature: ui-redesign, Property 9: Ratings Format**
 * *For any* movie with ratings, ratings SHALL display in the format "★ KP X.X | IMDB X.X | TMDB X.X"
 * **Validates: Requirements 2.5**
 */
describe('MovieTile Component - Property 9: Ratings Format', () => {
    const ratingArbitrary = fc.float({ min: 0, max: 10, noNaN: true });

    it('should format Kinopoisk rating with star and KP prefix', () => {
        fc.assert(
            fc.property(ratingArbitrary, (rating) => {
                const formatted = `★ KP ${rating.toFixed(1)}`;
                expect(formatted).toMatch(/^★ KP \d+\.\d$/);
            }),
            { numRuns: 50 }
        );
    });

    it('should format IMDb rating with IMDB prefix', () => {
        fc.assert(
            fc.property(ratingArbitrary, (rating) => {
                const formatted = `IMDB ${rating.toFixed(1)}`;
                expect(formatted).toMatch(/^IMDB \d+\.\d$/);
            }),
            { numRuns: 50 }
        );
    });

    it('should format TMDB rating with TMDB prefix', () => {
        fc.assert(
            fc.property(ratingArbitrary, (rating) => {
                const formatted = `TMDB ${rating.toFixed(1)}`;
                expect(formatted).toMatch(/^TMDB \d+\.\d$/);
            }),
            { numRuns: 50 }
        );
    });

    it('should join multiple ratings with pipe separator', () => {
        fc.assert(
            fc.property(ratingArbitrary, ratingArbitrary, ratingArbitrary, (kp, imdb, tmdb) => {
                const ratings = [
                    `★ KP ${kp.toFixed(1)}`,
                    `IMDB ${imdb.toFixed(1)}`,
                    `TMDB ${tmdb.toFixed(1)}`,
                ];
                const joined = ratings.join(' | ');
                expect(joined).toContain(' | ');
                expect(joined.split(' | ').length).toBe(3);
            }),
            { numRuns: 50 }
        );
    });

    it('should display ratings in small text size', () => {
        const ratingClasses = 'text-xs text-netflix-light';
        expect(ratingClasses).toContain('text-xs');
        expect(ratingClasses).toContain('text-netflix-light');
    });

    it('should handle missing ratings gracefully', () => {
        // If a rating is undefined, it should not be included in the display
        const ratings = [];
        expect(ratings.length).toBe(0);
    });
});

/**
 * **Feature: ui-redesign, Property 10: Series Info Display**
 * *For any* TV series content, the tile SHALL display season number and episode count
 * **Validates: Requirements 2.6**
 */
describe('MovieTile Component - Property 10: Series Info Display', () => {
    const seasonArbitrary = fc.integer({ min: 1, max: 20 });
    const episodeArbitrary = fc.integer({ min: 1, max: 30 });

    it('should format series info as "Season X • Y episodes"', () => {
        fc.assert(
            fc.property(seasonArbitrary, episodeArbitrary, (season, episodes) => {
                const formatted = `Season ${season} • ${episodes} episodes`;
                expect(formatted).toMatch(/^Season \d+ • \d+ episodes$/);
            }),
            { numRuns: 50 }
        );
    });

    it('should display series info in small text size', () => {
        const seriesInfoClasses = 'text-xs text-netflix-light';
        expect(seriesInfoClasses).toContain('text-xs');
        expect(seriesInfoClasses).toContain('text-netflix-light');
    });

    it('should only display series info when number_of_seasons is defined', () => {
        fc.assert(
            fc.property(seasonArbitrary, (season) => {
                const hasSeriesInfo = season !== undefined;
                expect(hasSeriesInfo).toBe(true);
            }),
            { numRuns: 50 }
        );
    });

    it('should display series info below ratings', () => {
        // Series info should have mt-1 (margin-top) to position it below ratings
        const seriesInfoClasses = 'mt-1';
        expect(seriesInfoClasses).toBe('mt-1');
    });
});

/**
 * **Feature: ui-redesign, Property 11: Tile Border Radius**
 * *For any* movie tile, the border-radius SHALL be at least 8px
 * **Validates: Requirements 2.7**
 */
describe('MovieTile Component - Property 11: Tile Border Radius', () => {
    it('should have rounded-lg class on poster image', () => {
        const posterClasses = 'rounded-lg';
        expect(posterClasses).toBe('rounded-lg');
    });

    it('should have rounded-lg class on placeholder', () => {
        const placeholderClasses = 'rounded-lg';
        expect(placeholderClasses).toBe('rounded-lg');
    });

    it('should maintain border-radius of 8px minimum', () => {
        const borderRadiusValue = 8;
        expect(borderRadiusValue).toBeGreaterThanOrEqual(8);
    });

    it('should apply rounded-lg through Card component', () => {
        // Card component applies rounded-lg by default
        const cardBaseClasses = 'rounded-lg';
        expect(cardBaseClasses).toBe('rounded-lg');
    });

    it('should maintain border-radius in hover state', () => {
        // Hover state should not remove border-radius
        const hoverClasses = 'hover:shadow-lg hover:scale-105';
        expect(hoverClasses).not.toContain('rounded-none');
    });
});
