import { describe, it, expect, vi, afterEach } from 'vitest'
import { startAnalytics } from './analytics'

const SCRIPT_URL = 'https://analytics.example/a.js?id=tenant-42'

function fakeRouter() {
  const listeners: (() => void)[] = []
  return {
    subscribe: (_event: 'onResolved', listener: () => void) => {
      listeners.push(listener)
      return vi.fn()
    },
    resolve: () => listeners.forEach((l) => l()),
  }
}

function setRuntimeEnv(env: Record<string, string> | undefined) {
  Object.defineProperty(window, '_env_', { value: env, configurable: true, writable: true })
}

function injectedScript() {
  return document.head.querySelector<HTMLScriptElement>(`script[src="${SCRIPT_URL}"]`)
}

/** Stands in for a.js: defines window.a, reports nothing on its own. */
function loadScript() {
  const pageView = vi.fn()
  window.a = { pageView, trackEvent: vi.fn() }
  injectedScript()?.dispatchEvent(new Event('load'))
  return pageView
}

afterEach(() => {
  setRuntimeEnv(undefined)
  delete window.a
  injectedScript()?.remove()
})

describe('startAnalytics', () => {
  it('loads nothing without a configured script url', () => {
    setRuntimeEnv({})
    const router = fakeRouter()

    startAnalytics(router)

    expect(injectedScript()).toBeNull()
    expect(() => router.resolve()).not.toThrow()
  })

  it('injects the script with the tenant id intact', () => {
    setRuntimeEnv({ VITE_ANALYTICS_SCRIPT_URL: SCRIPT_URL })

    startAnalytics(fakeRouter())

    expect(injectedScript()?.src).toBe(SCRIPT_URL)
  })

  it('reports a view for every resolved navigation', () => {
    setRuntimeEnv({ VITE_ANALYTICS_SCRIPT_URL: SCRIPT_URL })
    const router = fakeRouter()
    startAnalytics(router)
    const pageView = loadScript()

    router.resolve()
    router.resolve()

    expect(pageView).toHaveBeenCalledTimes(2)
  })

  it('still reports the initial view that resolved before the script loaded', () => {
    setRuntimeEnv({ VITE_ANALYTICS_SCRIPT_URL: SCRIPT_URL })
    const router = fakeRouter()
    startAnalytics(router)

    router.resolve()
    const pageView = loadScript()

    expect(pageView).toHaveBeenCalledTimes(1)
  })

  it('collapses views held before the load into a single report', () => {
    setRuntimeEnv({ VITE_ANALYTICS_SCRIPT_URL: SCRIPT_URL })
    const router = fakeRouter()
    startAnalytics(router)

    router.resolve()
    router.resolve()
    const pageView = loadScript()

    expect(pageView).toHaveBeenCalledTimes(1)
  })

  it('does not report anything when nothing navigated before the load', () => {
    setRuntimeEnv({ VITE_ANALYTICS_SCRIPT_URL: SCRIPT_URL })
    startAnalytics(fakeRouter())

    const pageView = loadScript()

    expect(pageView).not.toHaveBeenCalled()
  })
})
