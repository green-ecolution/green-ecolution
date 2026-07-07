import { defineConfig, type Plugin } from 'vite'
import react, { reactCompilerPreset } from '@vitejs/plugin-react'
import babel from '@rolldown/plugin-babel'
import tailwindcss from '@tailwindcss/vite'
import path from 'path'
import tanstackRouter from '@tanstack/router-plugin/vite'
import basicSsl from '@vitejs/plugin-basic-ssl'
import { VitePWA } from 'vite-plugin-pwa'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'

const useTraefik = !!process.env.USE_TRAEFIK

// Replace Vite's "Local: localhost:5173" banner with the Traefik URL so the
// dev URL the user should actually open stays visible after Vite starts.
function publicDevUrlBanner(): Plugin {
  return {
    name: 'gec:public-dev-url-banner',
    configureServer(server) {
      const url = process.env.PUBLIC_DEV_URL ?? 'http://localhost:3000'
      const cyan = (s: string) => `\x1b[36m${s}\x1b[0m`
      const bold = (s: string) => `\x1b[1m${s}\x1b[0m`
      const dim = (s: string) => `\x1b[2m${s}\x1b[0m`
      server.printUrls = () => {
        process.stdout.write(
          `\n  ${cyan('➜')}  ${bold('Dev environment')}: ${cyan(url)}\n` +
            `     ${dim('proxied via Traefik (backend :3030, vite :5173)')}\n\n`,
        )
      }
    },
  }
}

//
// https://vitejs.dev/config/
export default defineConfig({
  clearScreen: !useTraefik,
  plugins: [
    tanstackRouter({
      target: 'react',
      autoCodeSplitting: true,
      quoteStyle: 'single',
    }),
    tailwindcss(),
    react(),
    babel({ presets: [reactCompilerPreset()] }),
    ...(!useTraefik ? [basicSsl()] : []),
    ...(useTraefik ? [publicDevUrlBanner()] : []),
    wasm(),
    topLevelAwait(),
    VitePWA({
      registerType: 'prompt',
      includeAssets: ['images/favicons/favicon.svg', 'images/favicons/apple-touch-icon.png'],
      manifest: false,
      workbox: {
        globPatterns: ['**/*.{js,css,html,ico,png,svg,woff2,wasm}'],
        maximumFileSizeToCacheInBytes: 5 * 1024 * 1024,
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
      ...(useTraefik
        ? {
            '/api': {
              target: `http://localhost:3030`,
              changeOrigin: true,
              ws: true,
            },
          }
        : {
            '/api-local': {
              target: `http://localhost:3030`,
              changeOrigin: true,
              rewrite: (path) => path.replace(/^\/api-local/, '/api'),
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
  // `just run-prod` serves the production build behind Traefik on 5173 (the
  // port the dev-services router expects); host/allowedHosts let the docker
  // Traefik reach it via host.docker.internal.
  preview: {
    host: true,
    allowedHosts: true,
    port: 5173,
    strictPort: true,
  },
  define: {
    // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing -- empty string must fall back to npm_package_version
    __APP_VERSION__: JSON.stringify(process.env.APP_VERSION || process.env.npm_package_version),
    __APP_BUILD_TIME__: JSON.stringify(new Date().toISOString().slice(0, 10).replace(/-/g, '')),
    __APP_CITY__: JSON.stringify(process.env.VITE_APP_CITY ?? 'Stadt Flensburg'),
  },
  build: {
    target: 'esnext',
    rollupOptions: {
      output: {
        advancedChunks: {
          groups: [
            { name: 'domain-wasm', test: /[\\/]domain-wasm[\\/]/ },
            { name: 'maplibre', test: /maplibre-gl/ },
          ],
        },
      },
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      'lottie-react': path.resolve(__dirname, 'node_modules/lottie-react/build/index.es.js'),
    },
  },
})
