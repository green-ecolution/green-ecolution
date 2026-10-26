import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ReactNode } from 'react'
import { renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ModuleListResponse } from '@green-ecolution/backend-client'

const isAuthenticated = vi.fn(() => true)
const queryFn = vi.fn<() => Promise<ModuleListResponse>>()

vi.mock('@/lib/auth/authSessionContext', () => ({
  useAuthSession: () => ({ isAuthenticated: isAuthenticated() }),
}))

vi.mock('@/api/queries', () => ({
  modulesQuery: () => ({ queryKey: ['modules'], queryFn: () => queryFn() }),
}))

vi.mock('./index', () => ({
  handbookIndex: {
    chapters: {
      intro: { slug: 'intro' },
      trees: { slug: 'trees', module: 'irrigation' },
    },
  },
}))

const { useChapterVisibility } = await import('./useChapterVisibility')

const wrapper = ({ children }: { children: ReactNode }) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } })
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
}

const response = (active: string[]): ModuleListResponse => ({
  modules: ['irrigation'],
  organizations: [{ organizationId: 'org', active }],
})

describe('useChapterVisibility', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    isAuthenticated.mockReturnValue(true)
  })

  it('shows every chapter while the modules request is pending', () => {
    queryFn.mockReturnValue(new Promise<ModuleListResponse>(() => undefined))

    const { result } = renderHook(() => useChapterVisibility(), { wrapper })

    expect(result.current('trees')).toBe(true)
  })

  it('hides a chapter of an inactive module once the modules are known', async () => {
    queryFn.mockResolvedValue(response([]))

    const { result } = renderHook(() => useChapterVisibility(), { wrapper })

    await waitFor(() => expect(result.current('trees')).toBe(false))
    expect(result.current('intro')).toBe(true)
  })

  it('keeps the chapter of an active module', async () => {
    queryFn.mockResolvedValue(response(['irrigation']))

    const { result } = renderHook(() => useChapterVisibility(), { wrapper })

    await waitFor(() => expect(queryFn).toHaveBeenCalled())
    expect(result.current('trees')).toBe(true)
  })

  it('shows every chapter when the modules request fails', async () => {
    queryFn.mockRejectedValue(new Error('boom'))

    const { result } = renderHook(() => useChapterVisibility(), { wrapper })

    await waitFor(() => expect(queryFn).toHaveBeenCalled())
    expect(result.current('trees')).toBe(true)
  })

  it('shows every chapter to a logged-out visitor without asking for modules', () => {
    isAuthenticated.mockReturnValue(false)

    const { result } = renderHook(() => useChapterVisibility(), { wrapper })

    expect(result.current('trees')).toBe(true)
    expect(queryFn).not.toHaveBeenCalled()
  })
})
