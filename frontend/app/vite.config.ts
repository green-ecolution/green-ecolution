import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import path from 'path'
import tanstackRouter from '@tanstack/router-plugin/vite'

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
  ],
  server: {
    host: true,
    proxy: {
      '/api-local': {
        target: 'http://localhost:3000',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api-local/, '/api'),
        ws: true,
      },
      '/api-stage': {
        target: 'https://app.stage.green-ecolution.de',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api-stage/, '/api'),
        ws: true,
      },
    },
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
