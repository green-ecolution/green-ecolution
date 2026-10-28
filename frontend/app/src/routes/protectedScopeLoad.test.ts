import { beforeEach, describe, expect, it, vi } from 'vitest'

const bypass = vi.hoisted(() => ({ value: false }))

vi.mock('@/lib/auth/runtimeConfig', () => ({ readAuthBypass: () => bypass.value }))
vi.mock('@/lib/auth/session', () => ({ getAuthSession: () => ({}) }))
vi.mock('@/lib/auth/handover', () => ({ startSigninHandover: vi.fn() }))
vi.mock('@/lib/router', () => ({ prefetch: vi.fn() }))
vi.mock('@/api/queries', () => ({
  userQueries: { me: () => ({ queryKey: ['me'] }), mapView: () => ({ queryKey: ['mv'] }) },
  modulesQuery: () => ({ queryKey: ['modules'] }),
}))

const { Route } = await import('./_protected')
const { getOrganizationScope, DEMO_USER_ID, setOrganizationScope, loadOrganizationScope } =
  await import('@/lib/auth/scope/organizationScopeStore')

const ORG = '0190a1b2-0000-7000-8000-000000000001'

const run = () =>
  (Route.options.beforeLoad as (o: unknown) => Promise<unknown>)({
    context: {
      auth: { isAuthenticated: () => Promise.resolve(true) },
      queryClient: { ensureQueryData: () => Promise.resolve({ id: 'u1' }) },
    },
    location: { pathname: '/', searchStr: '' },
    preload: false,
  })

describe('_protected scope loading', () => {
  beforeEach(() => {
    localStorage.clear()
    loadOrganizationScope('reset')
  })

  it('restores the stored scope for the demo user in auth-bypass mode', async () => {
    bypass.value = true
    localStorage.setItem(`ge.organizationScope.${DEMO_USER_ID}`, ORG)
    await run()
    expect(getOrganizationScope()).toBe(ORG)
  })

  it('restores the stored scope for the /me user otherwise', async () => {
    bypass.value = false
    localStorage.setItem('ge.organizationScope.u1', ORG)
    await run()
    expect(getOrganizationScope()).toBe(ORG)
    setOrganizationScope('u1', null)
  })
})
