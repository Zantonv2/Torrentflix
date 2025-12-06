import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [svelte()],
  build: {
    target: 'es2020',
    outDir: 'dist',
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true,
      },
    },
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          // Split vendor libraries into separate chunk
          if (id.includes('node_modules')) {
            if (id.includes('svelte')) {
              return 'svelte-vendor';
            }
            if (id.includes('tauri')) {
              return 'tauri-vendor';
            }
            return 'vendor';
          }
          // Split pages into separate chunks for lazy loading
          if (id.includes('components/pages/')) {
            const match = id.match(/components\/pages\/([^/]+)\.svelte/);
            if (match) {
              return `page-${match[1].toLowerCase()}`;
            }
          }
          // Split layout components
          if (id.includes('components/layout/')) {
            return 'layout';
          }
          // Split core components
          if (id.includes('components/core/')) {
            return 'core-components';
          }
        },
      },
    },
  },
  server: {
    port: 5173,
    host: "127.0.0.1",
    strictPort: true,
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./vitest.setup.ts'],
    exclude: ['node_modules', 'dist', '.idea', '.git', '.cache'],
  },
});
