import '@testing-library/jest-dom/vitest'
import { cleanup } from '@testing-library/react'
import { afterEach } from 'vitest'

// Without vitest's `globals`, testing-library registers no auto cleanup and
// renders would pile up across tests.
afterEach(cleanup)

// jsdom has no ResizeObserver, and the view installs one to report its height
// to the host.
class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}

globalThis.ResizeObserver = ResizeObserverStub as unknown as typeof ResizeObserver
