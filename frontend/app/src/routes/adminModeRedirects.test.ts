import { beforeEach, describe, expect, it, vi } from 'vitest'
import { isRedirect } from '@tanstack/react-router'

const admin = vi.hoisted(() => ({ value: false }))
vi.mock('@/lib/auth/scope/adminMode', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/lib/auth/scope/adminMode')>()),
  isAdminMode: () => Promise.resolve(admin.value),
}))
vi.mock('@/lib/auth/runtimeConfig', () => ({ readAuthBypass: () => false }))
vi.mock('@/lib/auth/session', () => ({ getAuthSession: () => ({}) }))
vi.mock('@/lib/auth/handover', () => ({ startSigninHandover: vi.fn() }))
vi.mock('@/lib/router', () => ({
  prefetch: vi.fn(),
  pendingLoading: () => () => null,
  crumbRoute: (titleKey: string) => ({ loader: () => ({ crumb: { titleKey } }) }),
}))
vi.mock('@/api/queries', () => ({
  userQueries: { me: () => ({ queryKey: ['me'] }), mapView: () => ({ queryKey: ['mv'] }) },
  modulesQuery: () => ({ queryKey: ['modules'] }),
  organizationQueries: {
    list: () => ({ queryKey: ['organizations'] }),
    resourceCounts: () => ({ queryKey: ['organizations', 'resource-counts'] }),
  },
}))
vi.mock('@/components/admin/TenantOverview', () => ({ default: () => null }))

const { Route: Protected } = await import('./_protected')
const { Route: Admin } = await import('./_protected/admin/route')

type BeforeLoad = (o: unknown) => Promise<unknown>

const redirectTarget = async (promise: Promise<unknown>): Promise<string | null> => {
  try {
    await promise
    return null
  } catch (error) {
    if (!isRedirect(error)) throw error
    return (error.options as { to?: string }).to ?? null
  }
}

const runProtected = (pathname: string, preload = false) =>
  redirectTarget(
    (Protected.options.beforeLoad as BeforeLoad)({
      context: {
        auth: { isAuthenticated: () => Promise.resolve(true) },
        queryClient: { ensureQueryData: () => Promise.resolve({ id: 'u1' }) },
      },
      location: { pathname, searchStr: '' },
      preload,
    }),
  )

const runAdmin = () =>
  redirectTarget((Admin.options.beforeLoad as BeforeLoad)({ context: { queryClient: {} } }))

describe('admin mode redirects', () => {
  beforeEach(() => {
    admin.value = false
  })

  it('sends a root member from an operational page to the tenant overview', async () => {
    admin.value = true
    await expect(runProtected('/map')).resolves.toBe('/admin')
    await expect(runProtected('/dashboard')).resolves.toBe('/admin')
  })

  it('lets a root member reach the instance pages', async () => {
    admin.value = true
    await expect(runProtected('/settings/organization')).resolves.toBeNull()
    await expect(runProtected('/admin')).resolves.toBeNull()
  })

  it('matches whole path segments, so /settingsfoo is operational', async () => {
    admin.value = true
    await expect(runProtected('/settingsfoo')).resolves.toBe('/admin')
  })

  it('leaves everyone else on the operational pages', async () => {
    await expect(runProtected('/map')).resolves.toBeNull()
  })

  it('does not redirect while only preloading', async () => {
    admin.value = true
    await expect(runProtected('/map', true)).resolves.toBeNull()
  })

  it('sends a non-admin from the overview to the dashboard', async () => {
    await expect(runAdmin()).resolves.toBe('/dashboard')
  })

  it('lets a root member into the overview', async () => {
    admin.value = true
    await expect(runAdmin()).resolves.toBeNull()
  })
})
