import { sveltekit } from '@sveltejs/kit/vite'
import { defineConfig } from 'vitest/config'
import viteCompression from 'vite-plugin-compression'

export default defineConfig({
  plugins: [
    sveltekit(),
    viteCompression(),
  ],
  test: {
    include: ['src/**/*.{test,spec}.{js,ts}'],
  },
  build: {
    target: 'esnext',
  }
})
