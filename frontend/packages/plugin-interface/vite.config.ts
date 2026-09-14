import { resolve } from 'path'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import dts from 'vite-plugin-dts'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(),
    dts({
      tsconfigPath: resolve(__dirname, 'tsconfig.build.json'),
      insertTypesEntry: true,
    }),
  ],
  build: {
    lib: {
      entry: resolve(__dirname, 'lib/main.ts'),
      formats: ['es'],
    },
    rollupOptions: {
      // Subpaths have to stay external too, not just the bare specifiers: a
      // bundled `react/jsx-runtime` resolves to react's CJS build, whose
      // `require("react")` survives into the ESM output and throws in the
      // browser on the first import of this package.
      external: [/^react($|\/)/, /^react-dom($|\/)/],
    },
  },
})
