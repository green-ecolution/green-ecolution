import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { WateringStatus } from '@green-ecolution/backend-client'

const { searchMock, setQuery, setSort, setFilter, resetFilters } = vi.hoisted(() => {
  const searchMock: { current: Record<string, unknown> } = { current: {} }
  return {
    searchMock,
    setQuery: vi.fn(),
    setSort: vi.fn(),
    setFilter: vi.fn(),
    resetFilters: vi.fn(),
  }
})

vi.mock('./useClusterListSearch', async () => {
  const actual =
    await vi.importActual<typeof import('./useClusterListSearch')>('./useClusterListSearch')
  return {
    ...actual,
    useClusterListSearch: () => ({
      search: searchMock.current,
      setQuery,
      setSort,
      setFilter,
      resetFilters,
    }),
  }
})

vi.mock('@/api/queries', async () => {
  const actual = await vi.importActual<typeof import('@/api/queries')>('@/api/queries')
  return {
    ...actual,
    regionsQuery: () => ({
      queryKey: ['regions-toolbar-test'],
      queryFn: () =>
        Promise.resolve({
          data: [{ id: 'region-1', name: 'Nordstadt' }],
        }),
    }),
  }
})

const { default: ClusterListToolbar } = await import('./ClusterListToolbar')

const renderToolbar = () => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={queryClient}>
      <ClusterListToolbar filteredRecords={0} />
    </QueryClientProvider>,
  )
}

describe('ClusterListToolbar', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    searchMock.current = {}
  })

  it('picking a status option calls setFilter with the selected statuses', () => {
    renderToolbar()

    fireEvent.click(screen.getByRole('button', { name: /^Status/ }))
    fireEvent.click(screen.getByRole('option', { name: 'In Ordnung' }))

    expect(setFilter).toHaveBeenCalledExactlyOnceWith('wateringStatuses', [WateringStatus.Good])
  })

  it('removing an active status chip calls setFilter without that status', () => {
    searchMock.current = { wateringStatuses: [WateringStatus.Good] }
    renderToolbar()

    fireEvent.click(screen.getByRole('button', { name: 'Filter In Ordnung entfernen' }))

    expect(setFilter).toHaveBeenCalledExactlyOnceWith('wateringStatuses', [])
  })
})
