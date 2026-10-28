import { describe, it, expect, vi, beforeEach } from 'vitest'
import type { ReactNode } from 'react'
import { renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { UserResponse } from '@green-ecolution/backend-client'
import { UNRESTRICTED } from './permissions'

const readAuthBypass = vi.fn(() => false)
const isAuthenticated = vi.fn(() => true)
const queryFn = vi.fn<() => Promise<UserResponse>>()
const reachableFn = vi.fn()

vi.mock('./runtimeConfig', () => ({
  readAuthBypass: () => readAuthBypass(),
}))

vi.mock('./authSessionContext', () => ({
  useAuthSession: () => ({ isAuthenticated: isAuthenticated() }),
}))

// Mocking the query keeps react-query real while avoiding network and auth headers.
vi.mock('@/api/queries', () => ({
  userQueries: {
    me: () => ({ queryKey: ['users', 'me'], queryFn: () => queryFn() }),
    myOrganizations: () => ({
      queryKey: ['users', 'me', 'organizations'],
      queryFn: () => reachableFn() as Promise<unknown>,
    }),
  },
}))

const { usePermissions } = await import('./usePermissions')
const { setOrganizationScope } = await import('./scope/organizationScopeStore')

const me = (permissions: string[]): UserResponse =>
  ({
    roles: [
      {
        id: 'role-1',
        name: 'Rolle',
        description: '',
        organizationId: null,
        permissions,
        createdAt: '2026-07-27T00:00:00Z',
      },
    ],
  }) as unknown as UserResponse

const wrapper = ({ children }: { children: ReactNode }) => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  })
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
}

describe('usePermissions', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    readAuthBypass.mockReturnValue(false)
    isAuthenticated.mockReturnValue(true)
    setOrganizationScope('u1', null)
  })

  it('applies the roles of an ancestor organization under a scope and drops unrelated ones', async () => {
    const roleAt = (organizationId: string, permissions: string[]) => ({
      id: organizationId,
      name: 'Rolle',
      description: '',
      organizationId,
      permissions,
      createdAt: '2026-07-27T00:00:00Z',
    })
    queryFn.mockResolvedValue({
      roles: [roleAt('tbz', ['tree:read']), roleAt('firma_b', ['vehicle:read'])],
    } as unknown as UserResponse)
    reachableFn.mockResolvedValue([
      { id: 'tbz', name: 'TBZ', parentId: null },
      { id: 'firma_a', name: 'Firma A', parentId: 'tbz' },
      { id: 'firma_b', name: 'Firma B', parentId: 'tbz' },
    ])
    setOrganizationScope('u1', 'firma_a')

    const { result } = renderHook(() => usePermissions(), { wrapper })

    await waitFor(() => {
      expect([...(result.current as ReadonlySet<string>)]).toEqual(['tree:read'])
    })
  })

  it('returns the union of the roles of the current user', async () => {
    queryFn.mockResolvedValue(me(['tree:read', 'vehicle:read']))

    const { result } = renderHook(() => usePermissions(), { wrapper })

    await waitFor(() => {
      expect(result.current).not.toBe(UNRESTRICTED)
      expect([...(result.current as ReadonlySet<string>)].sort()).toEqual([
        'tree:read',
        'vehicle:read',
      ])
    })
  })

  it('returns an empty set while the user is still loading', () => {
    queryFn.mockReturnValue(new Promise<UserResponse>(() => undefined))

    const { result } = renderHook(() => usePermissions(), { wrapper })

    expect([...(result.current as ReadonlySet<string>)]).toEqual([])
  })

  it('returns unrestricted and skips the request when auth is bypassed', () => {
    readAuthBypass.mockReturnValue(true)

    const { result } = renderHook(() => usePermissions(), { wrapper })

    expect(result.current).toBe(UNRESTRICTED)
    expect(queryFn).not.toHaveBeenCalled()
  })

  it('does not request the user when not authenticated', () => {
    isAuthenticated.mockReturnValue(false)

    const { result } = renderHook(() => usePermissions(), { wrapper })

    expect([...(result.current as ReadonlySet<string>)]).toEqual([])
    expect(queryFn).not.toHaveBeenCalled()
  })
})
