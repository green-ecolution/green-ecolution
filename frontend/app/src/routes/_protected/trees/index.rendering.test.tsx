import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { NO_PERMISSIONS } from '@/lib/auth/permissions'

const { searchMock, resetFilters } = vi.hoisted(() => {
  const searchMock: { current: Record<string, unknown> } = { current: {} }
  return { searchMock, resetFilters: vi.fn() }
})

// `Trees` reads its search via `Route.useSearch()`, not `useTreeListSearch()`, so
// the route object itself has to hand back a controllable value. Everything else
// on the fake route (validateSearch, loaderDeps, loader) is irrelevant here: the
// component under test is rendered directly, never through a real router match.
vi.mock('@tanstack/react-router', async () => {
  const actual =
    await vi.importActual<typeof import('@tanstack/react-router')>('@tanstack/react-router')
  return {
    ...actual,
    createFileRoute: () => (options: Record<string, unknown>) => ({
      options,
      useSearch: () => searchMock.current,
    }),
  }
})

vi.mock('@/components/tree/list/useTreeListSearch', async () => {
  const actual = await vi.importActual<typeof import('@/components/tree/list/useTreeListSearch')>(
    '@/components/tree/list/useTreeListSearch',
  )
  return {
    ...actual,
    useTreeListSearch: () => ({
      search: searchMock.current,
      setQuery: vi.fn(),
      setSort: vi.fn(),
      setFilter: vi.fn(),
      setHasSensor: vi.fn(),
      setHasCluster: vi.fn(),
      setClusterAssignment: vi.fn(),
      resetFilters,
    }),
  }
})

vi.mock('@/api/queries', async () => {
  const actual = await vi.importActual<typeof import('@/api/queries')>('@/api/queries')
  return {
    ...actual,
    clusterQueries: {
      ...actual.clusterQueries,
      list: () => ({
        queryKey: ['cluster-rendering-test'],
        queryFn: () =>
          Promise.resolve({
            data: [],
            pagination: { currentPage: 1, perPage: 100, totalPages: 1, totalRecords: 0 },
          }),
      }),
    },
    treeQueries: {
      ...actual.treeQueries,
      plantingYears: () => ({
        queryKey: ['planting-years-rendering-test'],
        queryFn: () => Promise.resolve([]),
      }),
      list: () => ({
        queryKey: ['tree-list-rendering-test'],
        queryFn: () =>
          Promise.resolve({
            data: [],
            pagination: { currentPage: 1, perPage: 25, totalPages: 1, totalRecords: 0 },
          }),
      }),
    },
  }
})

// Sidesteps AuthSessionProvider entirely; the create button this hides is not
// under test here, only the two empty states below it.
vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => NO_PERMISSIONS,
}))

const { Route } = await import('./index')
const Trees = Route.options.component as React.ComponentType

const renderTrees = () => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={queryClient}>
      <Trees />
    </QueryClientProvider>,
  )
}

describe('/trees route empty states', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    searchMock.current = {}
  })

  it('shows the plain empty message and no reset action when nothing is filtered', async () => {
    renderTrees()

    expect(await screen.findByText('Es ist noch kein Baum erfasst.')).toBeInTheDocument()
    expect(screen.queryByText('Kein Baum passt zu diesen Filtern.')).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Filter zurücksetzen' })).not.toBeInTheDocument()
  })

  it('shows the filtered empty message with a reset action when a filter narrows to zero results', async () => {
    searchMock.current = { hasCluster: true }
    renderTrees()

    expect(await screen.findByText('Kein Baum passt zu diesen Filtern.')).toBeInTheDocument()
    expect(screen.queryByText('Es ist noch kein Baum erfasst.')).not.toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Filter zurücksetzen' })).toBeInTheDocument()
  })

  it('wires the filtered empty state reset action to resetFilters', async () => {
    searchMock.current = { hasCluster: true }
    renderTrees()

    fireEvent.click(await screen.findByRole('button', { name: 'Filter zurücksetzen' }))

    expect(resetFilters).toHaveBeenCalledOnce()
  })
})
