import { describe, it, expect, vi, afterEach } from 'vitest'

afterEach(() => {
  vi.unstubAllEnvs()
  vi.resetModules()
})

describe('isAuthBypass', () => {
  it('is true when VITE_AUTH_BYPASS === "true"', async () => {
    vi.stubEnv('VITE_AUTH_BYPASS', 'true')
    const { isAuthBypass } = await import('./userManager')
    expect(isAuthBypass()).toBe(true)
  })

  it('is false otherwise', async () => {
    vi.stubEnv('VITE_AUTH_BYPASS', '')
    const { isAuthBypass } = await import('./userManager')
    expect(isAuthBypass()).toBe(false)
  })
})
