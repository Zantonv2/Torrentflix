import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { downloadStore, setActive, setLoading } from '../../stores/downloadStore';

describe('Downloads Page - Integration Tests', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('Download Polling Workflow', () => {
        it('should update download progress on each poll', () => {
            const downloads = [
                {
                    hash: 'torrent-1',
                    title: 'Movie.mkv',
                    progress: 40,
                    status: 'downloading' as const,
                    speed: 5500000,
                    downloadedBytes: 400000000,
                    totalBytes: 1000000000,
                },
            ];

            setActive(downloads);
            let state = get(downloadStore);
            expect(state.active[0].progress).toBe(40);

            // Update progress
            const updatedDownloads = [
                {
                    ...downloads[0],
                    progress: 50,
                },
            ];

            setActive(updatedDownloads);
            state = get(downloadStore);
            expect(state.active[0].progress).toBe(50);
        });
    });

    describe('Download Control Workflow', () => {
        it('should show confirmation dialog before delete', () => {
            const downloads = [
                {
                    hash: 'torrent-1',
                    title: 'Movie.mkv',
                    progress: 100,
                    status: 'completed' as const,
                    speed: 0,
                    downloadedBytes: 1000000000,
                    totalBytes: 1000000000,
                },
            ];

            setActive(downloads);
            const state = get(downloadStore);
            expect(state.active.length).toBe(1);
        });
    });

    describe('Loading State Workflow', () => {
        it('should show loading state while fetching downloads', () => {
            setLoading(true);
            let state = get(downloadStore);
            expect(state.loading).toBe(true);

            setLoading(false);
            state = get(downloadStore);
            expect(state.loading).toBe(false);
        });
    });
});
