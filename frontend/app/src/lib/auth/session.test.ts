import { describe, it, expect, vi, afterEach } from 'vitest'
import { DemoAuthSession } from './session'
import { DEMO_ACCESS_TOKEN } from './demoUser'

afterEach(() => {
  vi.unstubAllEnvs()
  vi.resetModules()
  vi.restoreAllMocks()
})

describe('OidcAuthSession', () => {
  it('exchanges the auth code once even when the callback runs twice', async () => {
    vi.stubEnv('VITE_AUTH_BYPASS', '')
    vi.stubEnv('VITE_OIDC_AUTHORITY', 'https://auth.example.com/realms/test')
    vi.stubEnv('VITE_OIDC_CLIENT_ID', 'frontend')

    const { UserManager } = await import('oidc-client-ts')
    const spy = vi
      .spyOn(UserManager.prototype, 'signinCallback')
      .mockResolvedValue({ state: { returnTo: '/map' } } as never)

    const { OidcAuthSession } = await import('./session')
    const session = new OidcAuthSession()

    const [first, second] = await Promise.all([session.signinCallback(), session.signinCallback()])

    expect(spy).toHaveBeenCalledTimes(1)
    expect(first).toBe('/map')
    expect(second).toBe('/map')
  })
})

describe('DemoAuthSession', () => {
  it('is always authenticated and returns the demo token', async () => {
    const s = new DemoAuthSession()
    expect(await s.isAuthenticated()).toBe(true)
    expect(await s.getAccessToken()).toBe(DEMO_ACCESS_TOKEN)
  })

  it('signin/signout are no-ops that resolve', async () => {
    const s = new DemoAuthSession()
    await expect(s.signinRedirect({ returnTo: '/x' })).resolves.toBeUndefined()
    await expect(s.signoutRedirect()).resolves.toBeUndefined()
  })

  it('signinCallback returns the default route', async () => {
    const s = new DemoAuthSession()
    expect(await s.signinCallback()).toBe('/dashboard')
  })
})
