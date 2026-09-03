import { vi } from 'vitest'

// vite-plugin-pwa only provides this virtual module for the real Vite build;
// tests alias it here so the generated route tree (which pulls in the whole
// app, including the PWA update hook) can be imported under vitest.
// eslint-disable-next-line react-x/no-unnecessary-use-prefix -- name must match the real hook it stands in for
export function useRegisterSW() {
  return { needRefresh: [false, vi.fn()], updateServiceWorker: vi.fn() }
}
