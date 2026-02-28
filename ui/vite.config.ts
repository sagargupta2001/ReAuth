import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig, loadEnv } from 'vite'
import tsconfigPaths from 'vite-tsconfig-paths'

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')
  const proxyTarget = env.VITE_API_PROXY_TARGET || 'http://localhost:3000'

  return {
    plugins: [react(), tsconfigPaths(), tailwindcss()],
    server: {
      proxy: {
        '/api': {
          target: proxyTarget,
          changeOrigin: true,
        },
      },
    },
    test: {
      globals: true,
      environment: 'jsdom',
      setupFiles: './src/shared/lib/test/setup.ts',
      include: ['src/**/*.{test,spec}.{ts,tsx}'],
      coverage: {
        all: true,
        provider: 'v8',
        reporter: ['text', 'json', 'json-summary', 'html'],
        include: ['src/**/*'],
        exclude: [
          'node_modules/',
          'src/shared/lib/test/**',
          'src/**/*.test.{ts,tsx}',
          'src/**/*.d.ts',
          'src/vite-env.d.ts',
          'src/app/main.tsx',
          '.types.ts'
        ],
      },
    },
  }
})
