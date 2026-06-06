import { describe, it, expect } from 'vitest'
import { DemoAuthSession } from './session'
import { DEMO_ACCESS_TOKEN } from './demoUser'

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
