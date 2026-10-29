import { beforeEach, describe, expect, it, vi } from 'vitest'
import { renderHook } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import type { UserResponse } from '@green-ecolution/backend-client'
import { userQueries } from '@/api/queries'
import { setOrganizationScope } from './organizationScopeStore'

const bypass = vi.hoisted(() => ({ value: false }))
vi.mock('@/lib/auth/runtimeConfig', () => ({ readAuthBypass: () => bypass.value }))

const { isAdminMode, isRootMember, useAdminMode } = await import('./adminMode')

const ROOT = '01980000-0000-7000-8000-000000000001'
const TENANT = '01980000-0000-7000-8000-000000000002'

const rootMember = { id: 'u1', organization: { id: ROOT, name: 'Green Ecolution' } } as UserResponse
const tenantMember = {
  id: 'u2',
  organization: { id: TENANT, name: 'TBZ', parentId: ROOT },
} as UserResponse
const orphan = { id: 'u3', organization: null } as UserResponse

const clientWith = (me: UserResponse | undefined) => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { staleTime: Infinity, retry: false } },
  })
  if (me) queryClient.setQueryData(userQueries.me().queryKey, me)
  return queryClient
}

const renderAdminMode = (me: UserResponse | undefined) => {
  const queryClient = clientWith(me)
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  )
  return renderHook(() => useAdminMode(), { wrapper })
}

describe('isRootMember', () => {
  it('is true only for a member of the organization without a parent', () => {
    expect(isRootMember(rootMember)).toBe(true)
    expect(isRootMember(tenantMember)).toBe(false)
    expect(isRootMember(orphan)).toBe(false)
    expect(isRootMember(undefined)).toBe(false)
  })
})

describe('useAdminMode', () => {
  beforeEach(() => {
    bypass.value = false
    localStorage.clear()
    setOrganizationScope('u1', null)
  })

  it('is on for a root member without a scope', () => {
    expect(renderAdminMode(rootMember).result.current).toBe(true)
  })

  it('is off once a scope is set', () => {
    setOrganizationScope('u1', TENANT)
    expect(renderAdminMode(rootMember).result.current).toBe(false)
  })

  it('is off for a tenant member', () => {
    expect(renderAdminMode(tenantMember).result.current).toBe(false)
  })

  it('is off while the user has not loaded', () => {
    expect(renderAdminMode(undefined).result.current).toBe(false)
  })

  it('is off in auth-bypass mode even with a root member cached', () => {
    bypass.value = true
    expect(renderAdminMode(rootMember).result.current).toBe(false)
  })
})

describe('isAdminMode', () => {
  beforeEach(() => {
    bypass.value = false
    localStorage.clear()
    setOrganizationScope('u1', null)
  })

  it('resolves the user through the query client', async () => {
    await expect(isAdminMode(clientWith(rootMember))).resolves.toBe(true)
    await expect(isAdminMode(clientWith(tenantMember))).resolves.toBe(false)
  })

  it('is false with a scope or in auth-bypass mode without touching the client', async () => {
    setOrganizationScope('u1', TENANT)
    await expect(isAdminMode(clientWith(rootMember))).resolves.toBe(false)
    setOrganizationScope('u1', null)
    bypass.value = true
    await expect(isAdminMode(clientWith(rootMember))).resolves.toBe(false)
  })
})
