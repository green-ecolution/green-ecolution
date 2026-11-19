import { beforeEach, describe, expect, it, vi } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import type { ReachableOrganizationResponse, RoleResponse } from '@green-ecolution/backend-client'
import { userApi } from '@/api/backendApi'
import { userQueries } from '@/api/queries'

const roles = vi.hoisted(() => ({ value: undefined as RoleResponse[] | undefined }))
vi.mock('@/lib/auth/scope/effectiveRoles', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/lib/auth/scope/effectiveRoles')>()),
  useEffectiveRoles: () => roles.value,
}))
vi.mock('@/lib/auth/runtimeConfig', () => ({ readAuthBypass: () => false }))

const { usePermissionsIn, usePermissionsLoadError, usePermissionsResolver } =
  await import('./usePermissionsIn')

const REACHABLE: ReachableOrganizationResponse[] = [
  { id: 'amt', name: 'Grünflächenamt', parentId: 'root' },
  { id: 'nord', name: 'Stadtgärtnerei Nord', parentId: 'amt' },
]

const renderWith = <T,>(hook: () => T, reachable?: ReachableOrganizationResponse[]) => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { staleTime: Infinity, retry: false, enabled: false } },
  })
  if (reachable) queryClient.setQueryData(userQueries.myOrganizations().queryKey, reachable)
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  )
  return renderHook(hook, { wrapper }).result.current
}

describe('usePermissionsResolver', () => {
  beforeEach(() => {
    roles.value = [
      { id: 'r', name: 'Rolle', organizationId: 'amt', permissions: ['role:read'] } as RoleResponse,
    ]
  })

  it('is undefined while the roles or the organizations load', () => {
    expect(renderWith(usePermissionsResolver)).toBeUndefined()
    roles.value = undefined
    expect(renderWith(usePermissionsResolver, REACHABLE)).toBeUndefined()
  })

  it('resolves the rights held in an organization or above it', () => {
    const permissionsOf = renderWith(usePermissionsResolver, REACHABLE)
    expect(permissionsOf?.('nord')).toEqual(new Set(['role:read']))
    expect(permissionsOf?.('root')).toEqual(new Set())
  })
})

describe('usePermissionsIn', () => {
  it('reads no rights while loading', () => {
    expect(renderWith(() => usePermissionsIn('amt'))).toEqual(new Set())
  })
})

describe('usePermissionsLoadError', () => {
  it('reports a failed request for the reachable organizations', async () => {
    vi.spyOn(userApi, 'getMe').mockResolvedValue({ id: 'u1' } as never)
    vi.spyOn(userApi, 'listMyOrganizations').mockRejectedValue(new Error('500'))
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    )
    const { result } = renderHook(usePermissionsLoadError, { wrapper })
    expect(result.current).toBeNull()
    await waitFor(() => expect(result.current?.message).toBe('500'))
  })
})
