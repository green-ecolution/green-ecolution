import { describe, it, expect, vi, beforeEach } from 'vitest'
import type { ReactNode } from 'react'
import { renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { UserResponse } from '@green-ecolution/backend-client'
import { UNRESTRICTED } from './permissions'

const readAuthBypass = vi.fn(() => false)
const isAuthenticated = vi.fn(() => true)
const queryFn = vi.fn<() => Promise<UserResponse>>()

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
  },
}))

const { usePermissions } = await import('./usePermissions')

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
