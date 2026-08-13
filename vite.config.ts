import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from 'node:path';
import tailwindcss from '@tailwindcss/vite';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  clearScreen: false,
  build: {
    target: 'es2022',
    rollupOptions: {
      input: {
        // Main app entry
        main: path.resolve(__dirname, 'index.html'),
        // Hidden webview entry for youtubei.js extraction engine
        // Built to dist/extractor.html (loaded by Rust via WebviewUrl::App("extractor.html"))
        extractor: path.resolve(__dirname, 'extractor.html'),
      },
      output: {
        // Place extractor entry at dist root so the HTML loads cleanly
        entryFileNames: 'assets/[name]-[hash].js',
      },
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
