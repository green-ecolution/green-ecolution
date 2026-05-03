import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import path from 'path'
import tanstackRouter from '@tanstack/router-plugin/vite'
import basicSsl from '@vitejs/plugin-basic-ssl'
import { VitePWA } from 'vite-plugin-pwa'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'

//
// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    tanstackRouter({
      target: 'react',
      autoCodeSplitting: true,
      quoteStyle: 'single',
    }),
    tailwindcss(),
    react({
      babel: {
        plugins: ['babel-plugin-react-compiler'],
      },
    }),
    ...(!process.env.USE_TRAEFIK ? [basicSsl()] : []),
    wasm(),
    topLevelAwait(),
    VitePWA({
      registerType: 'prompt',
      includeAssets: ['images/favicons/favicon.svg', 'images/favicons/apple-touch-icon.png'],
      manifest: false,
      workbox: {
        globPatterns: ['**/*.{js,css,html,ico,png,svg,woff2}'],
        maximumFileSizeToCacheInBytes: 3 * 1024 * 1024,
      },
      devOptions: {
        enabled: false,
      },
    }),
  ],
  server: {
    host: true,
    allowedHosts: true,
    proxy: {
      ...(process.env.USE_TRAEFIK
        ? {
            '/api': {
              target: `http://localhost:3030`,
              changeOrigin: true,
              ws: true,
            },
          }
        : {
            '/api-local': {
              target: `http://localhost:3020`,
              changeOrigin: true,
              rewrite: (path) => path.replace(/^\/api-local/, ''),
              ws: true,
            },
          }),
      '/api-stage': {
        target: 'https://app.stage.green-ecolution.de',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api-stage/, '/api'),
        ws: true,
      },
    },
  },
  define: {
    // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing -- empty string must fall back to npm_package_version
    __APP_VERSION__: JSON.stringify(process.env.APP_VERSION || process.env.npm_package_version),
    __APP_BUILD_TIME__: JSON.stringify(new Date().toISOString().slice(0, 10).replace(/-/g, '')),
    __APP_CITY__: JSON.stringify(process.env.VITE_APP_CITY ?? 'Stadt Flensburg'),
  },
  build: {
    target: 'esnext',
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
})
