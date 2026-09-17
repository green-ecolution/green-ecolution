import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { WateringStatus, SoilCondition } from '@green-ecolution/backend-client'

const {
  searchMock,
  setQuery,
  setSort,
  setFilter,
  setHasSensor,
  setHasCluster,
  setClusterAssignment,
  resetFilters,
} = vi.hoisted(() => {
  const searchMock: { current: Record<string, unknown> } = { current: {} }
  return {
    searchMock,
    setQuery: vi.fn(),
    setSort: vi.fn(),
    setFilter: vi.fn(),
    setHasSensor: vi.fn(),
    setHasCluster: vi.fn(),
    setClusterAssignment: vi.fn(),
    resetFilters: vi.fn(),
  }
})

vi.mock('./useTreeListSearch', async () => {
  const actual = await vi.importActual<typeof import('./useTreeListSearch')>('./useTreeListSearch')
  return {
    ...actual,
    useTreeListSearch: () => ({
      search: searchMock.current,
      setQuery,
      setSort,
      setFilter,
      setHasSensor,
      setHasCluster,
      setClusterAssignment,
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
        queryKey: ['cluster-toolbar-test'],
        queryFn: () =>
          Promise.resolve({
            data: [
              {
                id: 'cluster-1',
                name: 'Nordpark',
                address: 'Am Nordpark 1',
                description: '',
                archived: false,
                createdAt: '2024-01-01T00:00:00Z',
                updatedAt: '2024-01-01T00:00:00Z',
                latitude: 54.78,
                longitude: 9.43,
                moistureLevel: 0,
                sensorCount: 0,
                soilCondition: SoilCondition.Ss,
                treeIds: [],
                wateringStatus: WateringStatus.Good,
              },
            ],
            pagination: { currentPage: 1, perPage: 100, totalPages: 1, totalRecords: 1 },
          }),
      }),
    },
    treeQueries: {
      ...actual.treeQueries,
      plantingYears: () => ({
        queryKey: ['planting-years-toolbar-test'],
        queryFn: () => Promise.resolve([]),
      }),
      list: () => ({
        queryKey: ['tree-list-toolbar-test'],
        queryFn: () =>
          Promise.resolve({
            data: [],
            pagination: { currentPage: 1, perPage: 1, totalPages: 1, totalRecords: 0 },
          }),
      }),
    },
  }
})

const { default: TreeListToolbar } = await import('./TreeListToolbar')

const renderToolbar = () => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={queryClient}>
      <TreeListToolbar filteredRecords={0} />
    </QueryClientProvider>,
  )
}

const openClusterDropdown = () => {
  // Anchored: an active "hasCluster" filter also renders a chip whose remove
  // button's label contains "Gruppe", which an unanchored match would catch too.
  fireEvent.click(screen.getByRole('button', { name: /^Gruppe/ }))
}

describe('TreeListToolbar cluster assignment filter', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    searchMock.current = {}
  })

  it('picking "Ohne Gruppe" sets hasCluster to false and clears clusterIds', () => {
    renderToolbar()

    openClusterDropdown()
    fireEvent.click(screen.getByRole('option', { name: 'Ohne Gruppe' }))

    expect(setClusterAssignment).toHaveBeenCalledExactlyOnceWith({ hasCluster: false })
  })

  it('picking a real cluster afterwards clears hasCluster', async () => {
    searchMock.current = { hasCluster: false }
    renderToolbar()

    openClusterDropdown()
    // The real-cluster option only appears once clusterQueries.list resolves.
    fireEvent.click(await screen.findByRole('option', { name: 'Nordpark' }))

    expect(setClusterAssignment).toHaveBeenCalledExactlyOnceWith({ clusterIds: ['cluster-1'] })
  })

  it('removing the cluster-assignment chip clears hasCluster', () => {
    searchMock.current = { hasCluster: true }
    renderToolbar()

    fireEvent.click(screen.getByRole('button', { name: 'Filter Mit Gruppe entfernen' }))

    expect(setHasCluster).toHaveBeenCalledExactlyOnceWith(undefined)
  })
})
