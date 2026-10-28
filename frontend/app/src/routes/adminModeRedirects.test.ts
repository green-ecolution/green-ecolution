import { beforeEach, describe, expect, it, vi } from 'vitest'
import { isRedirect } from '@tanstack/react-router'

const admin = vi.hoisted(() => ({ value: false }))
vi.mock('@/lib/auth/scope/adminMode', () => ({
  isAdminMode: () => Promise.resolve(admin.value),
}))
vi.mock('@/lib/router', () => ({
  prefetch: vi.fn(),
  pendingLoading: () => () => null,
  crumbRoute: (titleKey: string) => ({ loader: () => ({ crumb: { titleKey } }) }),
}))
// The dashboard component pulls in the session; the test only runs beforeLoad.
vi.mock('@/lib/auth/session', () => ({ getAuthSession: () => ({}) }))
vi.mock('@/lib/auth/useCurrentUser', () => ({
  useCurrentUser: () => ({ firstName: '', lastName: '', email: '' }),
}))
vi.mock('@/api/queries', () => ({
  organizationQueries: {
    list: () => ({ queryKey: ['organizations'] }),
    resourceCounts: () => ({ queryKey: ['organizations', 'resource-counts'] }),
  },
}))
vi.mock('@/components/admin/TenantOverview', () => ({ default: () => null }))

const { Route: Dashboard } = await import('./_protected/dashboard')
const { Route: Admin } = await import('./_protected/admin/route')

type BeforeLoad = (o: unknown) => Promise<unknown>
const run = (route: { options: { beforeLoad?: unknown } }) =>
  (route.options.beforeLoad as BeforeLoad)({ context: { queryClient: {} } })

const redirectTarget = async (promise: Promise<unknown>): Promise<string | null> => {
  try {
    await promise
    return null
  } catch (error) {
    if (!isRedirect(error)) throw error
    return (error.options as { to?: string }).to ?? null
  }
}

describe('admin mode redirects', () => {
  beforeEach(() => {
    admin.value = false
  })

  it('sends a root member from the dashboard to the tenant overview', async () => {
    admin.value = true
    await expect(redirectTarget(run(Dashboard))).resolves.toBe('/admin')
  })

  it('leaves everyone else on the dashboard', async () => {
    await expect(redirectTarget(run(Dashboard))).resolves.toBeNull()
  })

  it('sends a non-admin from the overview to the dashboard', async () => {
    await expect(redirectTarget(run(Admin))).resolves.toBe('/dashboard')
  })

  it('lets a root member into the overview', async () => {
    admin.value = true
    await expect(redirectTarget(run(Admin))).resolves.toBeNull()
  })
})
