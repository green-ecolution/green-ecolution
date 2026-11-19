import { beforeEach, describe, expect, it, vi } from 'vitest'
import { isRedirect } from '@tanstack/react-router'
import { isValidElement } from 'react'
import { QueryClient } from '@tanstack/react-query'
import type { ReachableOrganizationResponse, UserResponse } from '@green-ecolution/backend-client'
import { userApi } from '@/api/backendApi'
import { infoQueries, userQueries } from '@/api/queries'
import { ForbiddenError } from '@/lib/router'
import { setOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'

const bypass = vi.hoisted(() => ({ value: false }))
vi.mock('@/lib/auth/runtimeConfig', () => ({ readAuthBypass: () => bypass.value }))

const { Route: Entry } = await import('./_protected/settings/organization/index')
const { Route: Layout } = await import('./_protected/settings/organization/$orgId/route')
const { default: EntityNotFound } = await import('@/components/layout/EntityNotFound')

const REACHABLE: ReachableOrganizationResponse[] = [
  { id: 'amt', name: 'Grünflächenamt', parentId: 'root' },
  { id: 'nord', name: 'Stadtgärtnerei Nord', parentId: 'amt' },
]

const data = vi.hoisted(
  (): {
    me: UserResponse
    reachable: ReachableOrganizationResponse[]
    features: { name: string; enabled: boolean }[]
  } => ({ me: {} as UserResponse, reachable: [], features: [] }),
)

// A real client: the routes must notice an invalidated or outdated cache entry.
let queryClient: QueryClient
const seedQueryClient = () => {
  queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  queryClient.setQueryData(userQueries.me().queryKey, data.me)
  queryClient.setQueryData(userQueries.myOrganizations().queryKey, data.reachable)
  queryClient.setQueryData(infoQueries.services().queryKey, {
    items: data.features,
  } as never)
}

type Hook = (options: unknown) => Promise<unknown>

const entryRedirect = async (): Promise<{ to?: string; params?: { orgId?: string } }> => {
  try {
    seedQueryClient()
    await (Entry.options.beforeLoad as Hook)({ context: { queryClient } })
  } catch (error) {
    if (!isRedirect(error)) throw error
    return error.options as { to?: string; params?: { orgId?: string } }
  }
  throw new Error('expected a redirect')
}
const entryTarget = async () => (await entryRedirect()).params?.orgId

const meWith = (permissions: string[]) =>
  ({
    id: 'u1',
    organization: { id: 'amt', name: 'Grünflächenamt', parentId: 'root' },
    roles: [{ id: 'r1', name: 'Rolle', organizationId: 'amt', permissions }],
  }) as UserResponse

describe('/settings/organization (entry)', () => {
  beforeEach(() => {
    bypass.value = false
    setOrganizationScope('u1', null)
    data.reachable = REACHABLE
    data.features = [{ name: 'routing', enabled: true }]
    data.me = meWith(['organization:read'])
  })

  it('redirects to the own organization', async () => {
    await expect(entryTarget()).resolves.toBe('amt')
  })

  it('redirects to the scope organization', async () => {
    setOrganizationScope('u1', 'nord')
    await expect(entryTarget()).resolves.toBe('nord')
  })

  it('is forbidden when no organization resolves', async () => {
    data.reachable = []
    await expect(entryTarget()).rejects.toBeInstanceOf(ForbiddenError)
  })

  // Right after a delete the cached list may still hold the deleted organization.
  it('does not resolve an organization that is gone from a freshly fetched list', async () => {
    data.me = { ...meWith(['organization:read']), organization: null }
    data.reachable = [{ id: 'abgerissen', name: 'Abgerissen', parentId: 'root' }, ...REACHABLE]
    vi.spyOn(userApi, 'listMyOrganizations').mockResolvedValue(REACHABLE)
    seedQueryClient()
    await queryClient.invalidateQueries({ queryKey: userQueries.myOrganizations().queryKey })
    await expect(
      (Entry.options.beforeLoad as Hook)({ context: { queryClient } }).catch((error: unknown) =>
        isRedirect(error) ? error.options.params : error,
      ),
    ).resolves.toEqual({ orgId: 'amt' })
  })

  it('opens the overview with organization:read', async () => {
    await expect(entryRedirect()).resolves.toMatchObject({
      to: '/settings/organization/$orgId',
      params: { orgId: 'amt' },
    })
  })

  it('opens the stations for a user who holds only start_point:read', async () => {
    data.me = meWith(['start_point:read'])
    await expect(entryRedirect()).resolves.toMatchObject({
      to: '/settings/organization/$orgId/stations',
      params: { orgId: 'amt' },
    })
  })

  it('is forbidden when no section is visible, e.g. stations with routing off', async () => {
    data.me = meWith(['start_point:read'])
    data.features = [{ name: 'routing', enabled: false }]
    await expect(entryTarget()).rejects.toBeInstanceOf(ForbiddenError)
  })

  it('resolves the topmost reachable organization and its overview in the auth bypass', async () => {
    bypass.value = true
    data.me = {} as UserResponse
    data.features = []
    await expect(entryRedirect()).resolves.toMatchObject({
      to: '/settings/organization/$orgId',
      params: { orgId: 'amt' },
    })
  })
})

interface LoaderResult {
  crumb: { title: string }
}

const load = (orgId: string) =>
  (Layout.options.loader as (options: unknown) => Promise<LoaderResult>)({
    context: { queryClient },
    params: { orgId },
  })

const errorViewOf = (error: unknown) =>
  (Layout.options.errorComponent as (props: unknown) => unknown)({
    error,
    reset: () => undefined,
  })

const loadError = async (orgId: string): Promise<unknown> => {
  try {
    await load(orgId)
  } catch (error) {
    return error
  }
  throw new Error('expected the loader to fail')
}

describe('/settings/organization/$orgId (layout)', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
    vi.spyOn(console, 'error').mockImplementation(() => undefined)
    data.reachable = REACHABLE
    seedQueryClient()
  })

  it('names the crumb after a reachable organization', async () => {
    await expect(load('nord')).resolves.toEqual({ crumb: { title: 'Stadtgärtnerei Nord' } })
  })

  // A just created organization is missing from the cached list until it is refetched.
  it('looks again in a freshly fetched list before giving up', async () => {
    vi.spyOn(userApi, 'listMyOrganizations').mockResolvedValue([
      ...REACHABLE,
      { id: 'west', name: 'Betriebshof West', parentId: 'amt' },
    ])
    await expect(load('west')).resolves.toEqual({ crumb: { title: 'Betriebshof West' } })
  })

  it('shows "organization not found" when the fresh list lacks it too', async () => {
    vi.spyOn(userApi, 'listMyOrganizations').mockResolvedValue(REACHABLE)
    const element = errorViewOf(await loadError('fremd'))
    expect(isValidElement(element) && element.type).toBe(EntityNotFound)
  })

  it('does not report a failed request as "not found"', async () => {
    vi.spyOn(userApi, 'listMyOrganizations').mockRejectedValue(new Error('Network down'))
    const element = errorViewOf(await loadError('fremd'))
    expect(isValidElement(element) && element.type).not.toBe(EntityNotFound)
  })
})
