import { beforeEach, describe, expect, it, vi } from 'vitest'
import { isRedirect } from '@tanstack/react-router'
import { isValidElement } from 'react'
import type { QueryClient } from '@tanstack/react-query'
import type { ReachableOrganizationResponse, UserResponse } from '@green-ecolution/backend-client'
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

const queryClient = {
  ensureQueryData: ({ queryKey }: { queryKey: unknown[] }) => {
    const key = queryKey.join('/')
    if (key === 'users/me/organizations') return Promise.resolve(data.reachable)
    if (key === 'info/services') return Promise.resolve({ items: data.features })
    return Promise.resolve(data.me)
  },
} as unknown as QueryClient

type Hook = (options: unknown) => Promise<unknown>

const entryRedirect = async (): Promise<{ to?: string; params?: { orgId?: string } }> => {
  try {
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

describe('/settings/organization/$orgId (layout)', () => {
  beforeEach(() => {
    data.reachable = REACHABLE
  })

  it('names the crumb after a reachable organization', async () => {
    await expect(load('nord')).resolves.toEqual({ crumb: { title: 'Stadtgärtnerei Nord' } })
  })

  it('fails for an organization that is not reachable', async () => {
    await expect(load('fremd')).rejects.toThrow()
  })

  it('renders the failure as "organization not found"', () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined)
    const element = (Layout.options.errorComponent as (props: unknown) => unknown)({
      error: new Error('not reachable'),
      reset: () => undefined,
    })
    expect(isValidElement(element) && element.type).toBe(EntityNotFound)
  })
})
