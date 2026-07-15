import { describe, it, expect, vi, afterEach } from 'vitest'

afterEach(() => {
  vi.unstubAllEnvs()
  vi.resetModules()
})

describe('getUserManager', () => {
  it('requests offline_access so PWA sessions survive long suspends', async () => {
    vi.stubEnv('VITE_AUTH_BYPASS', '')
    vi.stubEnv('VITE_OIDC_AUTHORITY', 'https://auth.example.com/realms/test')
    vi.stubEnv('VITE_OIDC_CLIENT_ID', 'frontend')
    const { getUserManager } = await import('./userManager')
    expect(getUserManager().settings.scope).toContain('offline_access')
  })
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
