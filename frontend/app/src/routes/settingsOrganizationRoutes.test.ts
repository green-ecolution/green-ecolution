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

const data = vi.hoisted((): { me: UserResponse; reachable: ReachableOrganizationResponse[] } => ({
  me: {} as UserResponse,
  reachable: [],
}))

const queryClient = {
  ensureQueryData: ({ queryKey }: { queryKey: unknown[] }) =>
    Promise.resolve(queryKey.join('/') === 'users/me/organizations' ? data.reachable : data.me),
} as unknown as QueryClient

type Hook = (options: unknown) => Promise<unknown>

const entryTarget = async (): Promise<string | undefined> => {
  try {
    await (Entry.options.beforeLoad as Hook)({ context: { queryClient } })
  } catch (error) {
    if (!isRedirect(error)) throw error
    return (error.options as { params?: { orgId?: string } }).params?.orgId
  }
  throw new Error('expected a redirect')
}

describe('/settings/organization (entry)', () => {
  beforeEach(() => {
    bypass.value = false
    setOrganizationScope('u1', null)
    data.reachable = REACHABLE
    data.me = {
      id: 'u1',
      organization: { id: 'amt', name: 'Grünflächenamt' },
      roles: [
        { id: 'r1', name: 'Admin', organizationId: 'amt', permissions: ['organization:read'] },
      ],
    } as UserResponse
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

  it('resolves the topmost reachable organization in the auth bypass', async () => {
    bypass.value = true
    data.me = {} as UserResponse
    await expect(entryTarget()).resolves.toBe('amt')
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
