import type { ComponentProps } from 'react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { VehicleStatus, ListVehiclesArchiveEnum } from '@green-ecolution/backend-client'

const { searchMock, setQuery, setSort, setFilter, setArchive, resetFilters } = vi.hoisted(() => {
  const searchMock: { current: Record<string, unknown> } = { current: {} }
  return {
    searchMock,
    setQuery: vi.fn(),
    setSort: vi.fn(),
    setFilter: vi.fn(),
    setArchive: vi.fn(),
    resetFilters: vi.fn(),
  }
})

vi.mock('./useVehicleListSearch', async () => {
  const actual =
    await vi.importActual<typeof import('./useVehicleListSearch')>('./useVehicleListSearch')
  return {
    ...actual,
    useVehicleListSearch: () => ({
      search: searchMock.current,
      setQuery,
      setSort,
      setFilter,
      setArchive,
      resetFilters,
    }),
  }
})

vi.mock('@/modules/capacities', async (importOriginal) => {
  const { manifestOf } = await import('@/test/moduleFixtures')
  return {
    ...(await importOriginal<typeof import('@/modules/capacities')>()),
    useCapacityKinds: () => [
      ...(manifestOf('irrigation').capacities ?? []),
      ...(manifestOf('waste').capacities ?? []),
    ],
  }
})

const { default: VehicleListToolbar } = await import('./VehicleListToolbar')

const renderToolbar = (props: Partial<ComponentProps<typeof VehicleListToolbar>> = {}) =>
  render(<VehicleListToolbar filteredRecords={0} {...props} />)

describe('VehicleListToolbar', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    searchMock.current = {}
  })

  it('offers one sort entry per offered capacity kind', () => {
    searchMock.current = { sort: 'capacity', capacityKind: 'cargo_volume' }
    renderToolbar()

    fireEvent.pointerDown(screen.getByRole('button', { name: /Laderaum/ }), {
      button: 0,
      ctrlKey: false,
    })
    fireEvent.click(screen.getByRole('menuitemradio', { name: 'Wassertank' }))

    expect(setSort).toHaveBeenCalledExactlyOnceWith('capacity:water', 'asc')
  })

  it('picking a status option calls setFilter with the selected statuses', () => {
    renderToolbar()

    fireEvent.click(screen.getByRole('button', { name: /^Status/ }))
    fireEvent.click(screen.getByRole('option', { name: 'Verfügbar' }))

    expect(setFilter).toHaveBeenCalledExactlyOnceWith('statuses', [VehicleStatus.Available])
  })

  it('picking a second archive option replaces the first instead of adding to it', () => {
    searchMock.current = { archive: ListVehiclesArchiveEnum.Include }
    renderToolbar()

    fireEvent.click(screen.getByRole('button', { name: /^Archiv/ }))
    fireEvent.click(screen.getByRole('option', { name: 'Nur archivierte' }))

    expect(setArchive).toHaveBeenCalledExactlyOnceWith(ListVehiclesArchiveEnum.Only)
  })

  it('shows the "X von Y" count when the numbers differ, even with no chips set', () => {
    renderToolbar({ filteredRecords: 10, totalRecords: 12 })

    expect(screen.getByText('10 von 12 Fahrzeugen')).toBeInTheDocument()
  })

  it('shows the plain count when nothing is filtered and the numbers agree', () => {
    renderToolbar({ filteredRecords: 12, totalRecords: 12 })

    expect(screen.getByText('12 Fahrzeuge')).toBeInTheDocument()
  })
})
