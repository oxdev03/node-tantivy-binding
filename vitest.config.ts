import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    environment: 'node',
    testTimeout: 120000, // 2 minutes
    hookTimeout: 30000,
    teardownTimeout: 10000,
    globals: true,
    include: ['__test__/**/*.{test,spec}.{js,ts}'],
    benchmark: {
      include: ['benchmark/**/*.{bench,benchmark}.{js,ts}'],
    },
  },
})
