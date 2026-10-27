import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { SensorStatus } from '@green-ecolution/backend-client'

const { searchMock, setQuery, setSort, setFilter, setAttached, resetFilters, activeManifests } =
  vi.hoisted(() => {
    const searchMock: { current: Record<string, unknown> } = { current: {} }
    return {
      searchMock,
      setQuery: vi.fn(),
      setSort: vi.fn(),
      setFilter: vi.fn(),
      setAttached: vi.fn(),
      resetFilters: vi.fn(),
      activeManifests: { current: [] as import('@/modules/types').ModuleManifest[] },
    }
  })

vi.mock('@/modules/useActiveModules', () => ({
  useActiveModules: () => ({ manifests: activeManifests.current, isActive: () => true }),
}))

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
      setAttached,
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
const { probeTarget, probeTargetManifest } = await import('@/test/targetManifest')

const probeFilter = {
  param: 'probeIds',
  Control: ({ value, onChange }: { value: string[]; onChange: (value: string[]) => void }) => (
    <button type="button" onClick={() => onChange([...value, 'p-1'])}>
      probe-filter
    </button>
  ),
  Chips: ({ value, onChange }: { value: string[]; onChange: (value: string[]) => void }) =>
    value.map((id) => (
      <button type="button" key={id} onClick={() => onChange([])}>
        {`chip-${id}`}
      </button>
    )),
  sensorIds: (value: string[]) => ({
    queryKey: ['probe-sensor-ids', value],
    queryFn: () => Promise.resolve(['s-1']),
  }),
}

const withProbeFilter = () => {
  activeManifests.current = [
    probeTargetManifest('probe', [probeTarget('probe', 'probe', { listFilter: probeFilter })]),
  ]
}

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
    activeManifests.current = []
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

  it('picking "Verknüpft" calls setAttached with true', () => {
    renderToolbar()

    fireEvent.click(screen.getByRole('button', { name: /^Verknüpfung/ }))
    fireEvent.click(screen.getByRole('option', { name: 'Verknüpft' }))

    expect(setAttached).toHaveBeenCalledExactlyOnceWith(true)
  })

  it('picking the already-selected link option again clears it', () => {
    searchMock.current = { attached: false }
    renderToolbar()

    fireEvent.click(screen.getByRole('button', { name: /^Verknüpfung/ }))
    fireEvent.click(screen.getByRole('option', { name: 'Nicht verknüpft' }))

    expect(setAttached).toHaveBeenCalledExactlyOnceWith(undefined)
  })

  it('offers no target filter without an active module that brings one', () => {
    renderToolbar()

    expect(screen.queryByRole('button', { name: 'probe-filter' })).not.toBeInTheDocument()
  })

  it('renders the target filter of an active module and writes its search param', async () => {
    withProbeFilter()
    renderToolbar()

    fireEvent.click(await screen.findByRole('button', { name: 'probe-filter' }))

    expect(setFilter).toHaveBeenCalledExactlyOnceWith('probeIds', ['p-1'])
  })

  it('shows the chips of an active target filter and offers the reset', async () => {
    withProbeFilter()
    searchMock.current = { probeIds: ['p-1'] }
    renderToolbar()

    fireEvent.click(await screen.findByRole('button', { name: 'chip-p-1' }))

    expect(setFilter).toHaveBeenCalledExactlyOnceWith('probeIds', [])
    expect(screen.getByRole('button', { name: /Filter zurücksetzen/ })).toBeInTheDocument()
  })

  it('offers no prepared status in the activated view', () => {
    renderToolbar()

    fireEvent.click(screen.getByRole('button', { name: /^Status/ }))

    expect(screen.getByRole('option', { name: 'Online' })).toBeInTheDocument()
    expect(screen.queryByRole('option', { name: 'Vorbereitet' })).not.toBeInTheDocument()
  })

  it('shows only the filters a prepared sensor can match in the prepared view', () => {
    withProbeFilter()
    searchMock.current = { view: 'prepared' }
    renderToolbar()

    expect(screen.getByRole('button', { name: /^Modell/ })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /^Status/ })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /^Datenqualität/ })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /^Verknüpfung/ })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'probe-filter' })).not.toBeInTheDocument()
  })
})
