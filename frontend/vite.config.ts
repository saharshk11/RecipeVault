import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

const BACKEND = "http://localhost:3000"

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  server: {
    proxy: {
      "/auth": BACKEND,
      "/recipes": BACKEND,
      "/parse": BACKEND,
      "/health": BACKEND
    }
  }
});
