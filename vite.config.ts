import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  base: './',
  server: {
    port: 5173,
    host: '0.0.0.0',
    strictPort: true
  },
  optimizeDeps: {
    exclude: ['flowbite-svelte']
  },
  ssr: {
    noExternal: ['flowbite-svelte']
  }
});
