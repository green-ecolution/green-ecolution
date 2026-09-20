import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { SensorStatus } from '@green-ecolution/backend-client'

const { searchMock, setQuery, setSort, setFilter, setHasTree, resetFilters } = vi.hoisted(() => {
  const searchMock: { current: Record<string, unknown> } = { current: {} }
  return {
    searchMock,
    setQuery: vi.fn(),
    setSort: vi.fn(),
    setFilter: vi.fn(),
    setHasTree: vi.fn(),
    resetFilters: vi.fn(),
  }
})

vi.mock('./useSensorListSearch', async () => {
  const actual =
    await vi.importActual<typeof import('./useSensorListSearch')>('./useSensorListSearch')
  return {
    ...actual,
    useSensorListSearch: () => ({
      search: searchMock.current,
      setQuery,
      setSort,
      setFilter,
      setHasTree,
      resetFilters,
    }),
  }
})

vi.mock('@/api/queries', async () => {
  const actual = await vi.importActual<typeof import('@/api/queries')>('@/api/queries')
  return {
    ...actual,
    sensorQueries: {
      ...actual.sensorQueries,
      models: () => ({
        queryKey: ['sensor-models-toolbar-test'],
        queryFn: () =>
          Promise.resolve([
            { id: 'model-1', name: 'TEROS 12', abilities: [] },
            { id: 'model-2', name: 'FDR 12', abilities: [] },
          ]),
      }),
    },
  }
})

const { default: SensorListToolbar } = await import('./SensorListToolbar')

const renderToolbar = () => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={queryClient}>
      <SensorListToolbar filteredRecords={0} />
    </QueryClientProvider>,
  )
}

describe('SensorListToolbar', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    searchMock.current = {}
  })

  it('picking a status option calls setFilter with the selected statuses', () => {
    renderToolbar()

    fireEvent.click(screen.getByRole('button', { name: /^Status/ }))
    fireEvent.click(screen.getByRole('option', { name: 'Online' }))

    expect(setFilter).toHaveBeenCalledExactlyOnceWith('statuses', [SensorStatus.Online])
  })

  it('removing an active status chip calls setFilter without that status', () => {
    searchMock.current = { statuses: [SensorStatus.Online] }
    renderToolbar()

    fireEvent.click(screen.getByRole('button', { name: 'Filter Online entfernen' }))

    expect(setFilter).toHaveBeenCalledExactlyOnceWith('statuses', [])
  })

  it('picking "Mit Baum" calls setHasTree with true', () => {
    renderToolbar()

    fireEvent.click(screen.getByRole('button', { name: /^Baum/ }))
    fireEvent.click(screen.getByRole('option', { name: 'Mit Baum' }))

    expect(setHasTree).toHaveBeenCalledExactlyOnceWith(true)
  })

  it('picking the already-selected tree option again clears it', () => {
    searchMock.current = { hasTree: true }
    renderToolbar()

    fireEvent.click(screen.getByRole('button', { name: /^Baum/ }))
    fireEvent.click(screen.getByRole('option', { name: 'Mit Baum' }))

    expect(setHasTree).toHaveBeenCalledExactlyOnceWith(undefined)
  })
})
