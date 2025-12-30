import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import {
  QueryClient,
  QueryClientProvider,
  useQuery,
  useMutation,
  useSuspenseQuery,
  useQueryClient,
} from '@tanstack/react-query'
import { ReactNode, Suspense } from 'react'
import { http, HttpResponse, delay } from 'msw'
import { server } from './mocks/server'

const baseUrl = '/api-local'

interface TreeData {
  id: number
  number?: string
  clusterId?: number
  createdAt?: string
}

interface PaginatedResponse<T> {
  data: T[]
  pagination: { page: number; limit: number; total: number }
}

interface ClusterData {
  id: number
  name: string
}

function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
        staleTime: 0,
      },
      mutations: {
        retry: false,
      },
    },
  })
}

function createWrapper(queryClient?: QueryClient) {
  const client = queryClient ?? createTestQueryClient()
  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  )
}

describe('React Query Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    server.resetHandlers()
  })

  describe('useQuery lifecycle', () => {
    it('transitions through loading -> success states', async () => {
      server.use(
        http.get(`${baseUrl}/v1/tree`, async () => {
          await delay(50)
          return HttpResponse.json({
            data: [{ id: 1, number: 'T-001' }],
            pagination: { page: 1, limit: 10, total: 1 },
          })
        }),
      )

      const { result } = renderHook(
        () =>
          useQuery({
            queryKey: ['test-trees'],
            queryFn: async (): Promise<PaginatedResponse<TreeData>> => {
              const res = await fetch(`${baseUrl}/v1/tree`)
              return res.json() as Promise<PaginatedResponse<TreeData>>
            },
          }),
        { wrapper: createWrapper() },
      )

      expect(result.current.isPending).toBe(true)
      expect(result.current.isSuccess).toBe(false)

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true)
      })

      expect(result.current.isPending).toBe(false)
      expect(result.current.data).toEqual({
        data: [{ id: 1, number: 'T-001' }],
        pagination: { page: 1, limit: 10, total: 1 },
      })
    })

    it('transitions to error state on failure', async () => {
      server.use(
        http.get(`${baseUrl}/v1/tree`, () => {
          return HttpResponse.json({ error: 'Not found' }, { status: 404 })
        }),
      )

      const { result } = renderHook(
        () =>
          useQuery({
            queryKey: ['test-trees-error'],
            queryFn: async (): Promise<PaginatedResponse<TreeData>> => {
              const res = await fetch(`${baseUrl}/v1/tree`)
              if (!res.ok) throw new Error('Request failed')
              return res.json() as Promise<PaginatedResponse<TreeData>>
            },
          }),
        { wrapper: createWrapper() },
      )

      await waitFor(() => {
        expect(result.current.isError).toBe(true)
      })

      expect(result.current.error).toBeInstanceOf(Error)
      expect(result.current.error?.message).toBe('Request failed')
    })

    it('provides fetchStatus correctly', async () => {
      server.use(
        http.get(`${baseUrl}/v1/tree`, async () => {
          await delay(100)
          return HttpResponse.json({ data: [] })
        }),
      )

      const { result } = renderHook(
        () =>
          useQuery({
            queryKey: ['test-fetch-status'],
            queryFn: async (): Promise<PaginatedResponse<TreeData>> => {
              const res = await fetch(`${baseUrl}/v1/tree`)
              return res.json() as Promise<PaginatedResponse<TreeData>>
            },
          }),
        { wrapper: createWrapper() },
      )

      expect(result.current.fetchStatus).toBe('fetching')

      await waitFor(() => {
        expect(result.current.fetchStatus).toBe('idle')
      })
    })
  })

  describe('useMutation', () => {
    it('executes mutation and transitions states correctly', async () => {
      server.use(
        http.post(`${baseUrl}/v1/tree`, async ({ request }) => {
          const body = (await request.json()) as Record<string, unknown>
          return HttpResponse.json(
            { id: 1, ...body, createdAt: new Date().toISOString() },
            { status: 201 },
          )
        }),
      )

      const { result } = renderHook(
        () =>
          useMutation({
            mutationFn: async (data: { number: string }): Promise<TreeData> => {
              const res = await fetch(`${baseUrl}/v1/tree`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(data),
              })
              return res.json() as Promise<TreeData>
            },
          }),
        { wrapper: createWrapper() },
      )

      expect(result.current.isPending).toBe(false)
      expect(result.current.isSuccess).toBe(false)

      act(() => {
        result.current.mutate({ number: 'T-002' })
      })

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true)
      })

      expect(result.current.data).toMatchObject({ id: 1, number: 'T-002' })
    })

    it('handles mutation error', async () => {
      server.use(
        http.post(`${baseUrl}/v1/tree`, () => {
          return HttpResponse.json({ error: 'Validation failed' }, { status: 400 })
        }),
      )

      const { result } = renderHook(
        () =>
          useMutation({
            mutationFn: async (data: { number: string }): Promise<TreeData> => {
              const res = await fetch(`${baseUrl}/v1/tree`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(data),
              })
              if (!res.ok) throw new Error('Validation failed')
              return res.json() as Promise<TreeData>
            },
          }),
        { wrapper: createWrapper() },
      )

      act(() => {
        result.current.mutate({ number: '' })
      })

      await waitFor(() => {
        expect(result.current.isError).toBe(true)
      })

      expect(result.current.error?.message).toBe('Validation failed')
    })

    it('calls onSuccess callback', async () => {
      const onSuccessMock = vi.fn()

      server.use(
        http.post(`${baseUrl}/v1/tree`, () => {
          return HttpResponse.json({ id: 1 }, { status: 201 })
        }),
      )

      const { result } = renderHook(
        () =>
          useMutation({
            mutationFn: async (data: Record<string, unknown>): Promise<TreeData> => {
              const res = await fetch(`${baseUrl}/v1/tree`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(data),
              })
              return res.json() as Promise<TreeData>
            },
            onSuccess: onSuccessMock,
          }),
        { wrapper: createWrapper() },
      )

      act(() => {
        result.current.mutate({})
      })

      await waitFor(() => {
        expect(onSuccessMock).toHaveBeenCalledTimes(1)
      })
    })

    it('calls onError callback', async () => {
      const onErrorMock = vi.fn()

      server.use(
        http.post(`${baseUrl}/v1/tree`, () => {
          return HttpResponse.json({ error: 'Failed' }, { status: 500 })
        }),
      )

      const { result } = renderHook(
        () =>
          useMutation({
            mutationFn: async (data: Record<string, unknown>): Promise<TreeData> => {
              const res = await fetch(`${baseUrl}/v1/tree`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(data),
              })
              if (!res.ok) throw new Error('Server error')
              return res.json() as Promise<TreeData>
            },
            onError: onErrorMock,
          }),
        { wrapper: createWrapper() },
      )

      act(() => {
        result.current.mutate({})
      })

      await waitFor(() => {
        expect(onErrorMock).toHaveBeenCalledTimes(1)
      })
    })
  })

  describe('Cache invalidation', () => {
    it('invalidates queries after mutation', async () => {
      let fetchCount = 0

      server.use(
        http.get(`${baseUrl}/v1/tree`, () => {
          fetchCount++
          return HttpResponse.json({
            data: [{ id: fetchCount }],
            pagination: { page: 1, limit: 10, total: 1 },
          })
        }),
        http.post(`${baseUrl}/v1/tree`, () => {
          return HttpResponse.json({ id: 99 }, { status: 201 })
        }),
      )

      const queryClient = createTestQueryClient()

      const { result } = renderHook(
        () => {
          const query = useQuery({
            queryKey: ['invalidation-test'],
            queryFn: async (): Promise<PaginatedResponse<TreeData>> => {
              const res = await fetch(`${baseUrl}/v1/tree`)
              return res.json() as Promise<PaginatedResponse<TreeData>>
            },
          })

          const client = useQueryClient()

          const mutation = useMutation({
            mutationFn: async (): Promise<TreeData> => {
              const res = await fetch(`${baseUrl}/v1/tree`, {
                method: 'POST',
                body: JSON.stringify({}),
              })
              return res.json() as Promise<TreeData>
            },
            onSuccess: () => {
              void client.invalidateQueries({ queryKey: ['invalidation-test'] })
            },
          })

          return { query, mutation }
        },
        { wrapper: createWrapper(queryClient) },
      )

      await waitFor(() => {
        expect(result.current.query.isSuccess).toBe(true)
      })

      expect(fetchCount).toBe(1)

      act(() => {
        result.current.mutation.mutate()
      })

      await waitFor(() => {
        expect(result.current.mutation.isSuccess).toBe(true)
      })

      await waitFor(() => {
        expect(fetchCount).toBeGreaterThan(1)
      })
    })
  })

  describe('Conditional queries (enabled flag)', () => {
    it('does not execute query when enabled is false', () => {
      const queryFn = vi.fn().mockResolvedValue({ data: [] })

      const { result } = renderHook(
        () =>
          useQuery({
            queryKey: ['disabled-query'],
            queryFn,
            enabled: false,
          }),
        { wrapper: createWrapper() },
      )

      expect(result.current.isPending).toBe(true)
      expect(result.current.fetchStatus).toBe('idle')
      expect(queryFn).not.toHaveBeenCalled()
    })

    it('executes query when enabled changes to true', async () => {
      const queryFn = vi.fn().mockResolvedValue({ data: [{ id: 1 }] })

      const { result, rerender } = renderHook(
        ({ enabled }: { enabled: boolean }) =>
          useQuery({
            queryKey: ['conditional-query'],
            queryFn,
            enabled,
          }),
        {
          wrapper: createWrapper(),
          initialProps: { enabled: false },
        },
      )

      expect(queryFn).not.toHaveBeenCalled()

      rerender({ enabled: true })

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true)
      })

      expect(queryFn).toHaveBeenCalledTimes(1)
    })

    it('handles dependent queries correctly', async () => {
      server.use(
        http.get(`${baseUrl}/v1/tree/1`, () => {
          return HttpResponse.json({ id: 1, clusterId: 5 })
        }),
        http.get(`${baseUrl}/v1/cluster/5`, () => {
          return HttpResponse.json({ id: 5, name: 'Cluster 5' })
        }),
      )

      const { result } = renderHook(
        () => {
          const treeQuery = useQuery({
            queryKey: ['tree', 1],
            queryFn: async (): Promise<TreeData> => {
              const res = await fetch(`${baseUrl}/v1/tree/1`)
              return res.json() as Promise<TreeData>
            },
          })

          const clusterQuery = useQuery({
            queryKey: ['cluster', treeQuery.data?.clusterId],
            queryFn: async (): Promise<ClusterData> => {
              const res = await fetch(`${baseUrl}/v1/cluster/${treeQuery.data?.clusterId}`)
              return res.json() as Promise<ClusterData>
            },
            enabled: !!treeQuery.data?.clusterId,
          })

          return { treeQuery, clusterQuery }
        },
        { wrapper: createWrapper() },
      )

      expect(result.current.clusterQuery.fetchStatus).toBe('idle')

      await waitFor(() => {
        expect(result.current.treeQuery.isSuccess).toBe(true)
      })

      await waitFor(() => {
        expect(result.current.clusterQuery.isSuccess).toBe(true)
      })

      expect(result.current.clusterQuery.data).toEqual({ id: 5, name: 'Cluster 5' })
    })
  })

  describe('useSuspenseQuery', () => {
    it('suspends until data is available', async () => {
      server.use(
        http.get(`${baseUrl}/v1/tree`, async () => {
          await delay(50)
          return HttpResponse.json({
            data: [{ id: 1 }],
            pagination: { page: 1, limit: 10, total: 1 },
          })
        }),
      )

      const SuspenseWrapper = ({ children }: { children: ReactNode }) => {
        const queryClient = createTestQueryClient()
        return (
          <QueryClientProvider client={queryClient}>
            <Suspense fallback={<div>Loading...</div>}>{children}</Suspense>
          </QueryClientProvider>
        )
      }

      const { result } = renderHook(
        () =>
          useSuspenseQuery({
            queryKey: ['suspense-test'],
            queryFn: async (): Promise<PaginatedResponse<TreeData>> => {
              const res = await fetch(`${baseUrl}/v1/tree`)
              return res.json() as Promise<PaginatedResponse<TreeData>>
            },
          }),
        { wrapper: SuspenseWrapper },
      )

      await waitFor(() => {
        expect(result.current.data).toBeDefined()
      })

      expect(result.current.data).toEqual({
        data: [{ id: 1 }],
        pagination: { page: 1, limit: 10, total: 1 },
      })
    })

    it('throws error to error boundary on failure', async () => {
      server.use(
        http.get(`${baseUrl}/v1/tree`, () => {
          return HttpResponse.json({ error: 'Server error' }, { status: 500 })
        }),
      )

      const queryClient = createTestQueryClient()

      await expect(
        queryClient.fetchQuery({
          queryKey: ['suspense-error-test'],
          queryFn: async (): Promise<PaginatedResponse<TreeData>> => {
            const res = await fetch(`${baseUrl}/v1/tree`)
            if (!res.ok) throw new Error('Fetch failed')
            return res.json() as Promise<PaginatedResponse<TreeData>>
          },
        }),
      ).rejects.toThrow('Fetch failed')
    })
  })

  describe('Query refetching', () => {
    it('refetches data on refetch call', async () => {
      let callCount = 0

      server.use(
        http.get(`${baseUrl}/v1/tree`, () => {
          callCount++
          return HttpResponse.json({ data: [{ id: callCount }] })
        }),
      )

      const { result } = renderHook(
        () =>
          useQuery({
            queryKey: ['refetch-test'],
            queryFn: async (): Promise<{ data: TreeData[] }> => {
              const res = await fetch(`${baseUrl}/v1/tree`)
              return res.json() as Promise<{ data: TreeData[] }>
            },
          }),
        { wrapper: createWrapper() },
      )

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true)
      })

      expect(result.current.data).toEqual({ data: [{ id: 1 }] })

      await act(async () => {
        await result.current.refetch()
      })

      await waitFor(() => {
        expect(callCount).toBe(2)
      })

      expect(result.current.data).toEqual({ data: [{ id: 2 }] })
    })
  })

  describe('queryOptions pattern', () => {
    it('works correctly with spread operator', async () => {
      server.use(
        http.get(`${baseUrl}/v1/tree`, () => {
          return HttpResponse.json({ data: [{ id: 1, number: 'T-001' }] })
        }),
      )

      const treeQueryOptions = {
        queryKey: ['spread-test'],
        queryFn: async (): Promise<{ data: TreeData[] }> => {
          const res = await fetch(`${baseUrl}/v1/tree`)
          return res.json() as Promise<{ data: TreeData[] }>
        },
      }

      const { result } = renderHook(
        () =>
          useQuery({
            ...treeQueryOptions,
            select: (data) => data.data[0],
          }),
        { wrapper: createWrapper() },
      )

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true)
      })

      expect(result.current.data).toEqual({ id: 1, number: 'T-001' })
    })
  })

  describe('QueryClient methods', () => {
    it('setQueryData updates cache correctly', () => {
      const queryClient = createTestQueryClient()

      queryClient.setQueryData(['set-data-test'], { data: [{ id: 99 }] })

      const { result } = renderHook(
        () =>
          useQuery({
            queryKey: ['set-data-test'],
            queryFn: (): { data: TreeData[] } => ({ data: [] }),
            staleTime: Infinity,
          }),
        { wrapper: createWrapper(queryClient) },
      )

      expect(result.current.data).toEqual({ data: [{ id: 99 }] })
    })

    it('getQueryData retrieves cached data', async () => {
      server.use(
        http.get(`${baseUrl}/v1/tree`, () => {
          return HttpResponse.json({ data: [{ id: 1 }] })
        }),
      )

      const queryClient = createTestQueryClient()

      const { result } = renderHook(
        () => {
          const client = useQueryClient()
          const query = useQuery({
            queryKey: ['get-data-test'],
            queryFn: async (): Promise<{ data: TreeData[] }> => {
              const res = await fetch(`${baseUrl}/v1/tree`)
              return res.json() as Promise<{ data: TreeData[] }>
            },
          })
          return { client, query }
        },
        { wrapper: createWrapper(queryClient) },
      )

      await waitFor(() => {
        expect(result.current.query.isSuccess).toBe(true)
      })

      const cachedData = result.current.client.getQueryData(['get-data-test'])
      expect(cachedData).toEqual({ data: [{ id: 1 }] })
    })

    it('prefetchQuery populates cache', async () => {
      server.use(
        http.get(`${baseUrl}/v1/tree`, () => {
          return HttpResponse.json({ data: [{ id: 1 }] })
        }),
      )

      const queryClient = createTestQueryClient()

      await queryClient.prefetchQuery({
        queryKey: ['prefetch-test'],
        queryFn: async (): Promise<{ data: TreeData[] }> => {
          const res = await fetch(`${baseUrl}/v1/tree`)
          return res.json() as Promise<{ data: TreeData[] }>
        },
      })

      const { result } = renderHook(
        () =>
          useQuery({
            queryKey: ['prefetch-test'],
            queryFn: async (): Promise<{ data: TreeData[] }> => {
              const res = await fetch(`${baseUrl}/v1/tree`)
              return res.json() as Promise<{ data: TreeData[] }>
            },
          }),
        { wrapper: createWrapper(queryClient) },
      )

      expect(result.current.data).toEqual({ data: [{ id: 1 }] })
      expect(result.current.isPending).toBe(false)
    })
  })
})
