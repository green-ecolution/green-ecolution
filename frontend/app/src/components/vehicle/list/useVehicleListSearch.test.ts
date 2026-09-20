import { renderHook } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { ListVehiclesArchiveEnum } from '@green-ecolution/backend-client'

const navigate = vi.fn((_options: unknown) => Promise.resolve())
const search: {
  page: number
  q?: string
  statuses?: string[]
  types?: string[]
  drivingLicenses?: string[]
  archive?: ListVehiclesArchiveEnum
} = {
  page: 4,
  q: 'man',
  statuses: undefined,
  types: undefined,
  drivingLicenses: undefined,
  archive: undefined,
}

vi.mock('@tanstack/react-router', () => ({
  getRouteApi: () => ({
    useSearch: () => search,
    useNavigate: () => navigate,
  }),
}))

import { useVehicleListSearch } from './useVehicleListSearch'

const appliedSearch = () => {
  const calls = navigate.mock.calls
  const call = calls[calls.length - 1]?.[0] as {
    search: (prev: Record<string, unknown>) => Record<string, unknown>
  }
  return call.search({ ...search })
}

describe('useVehicleListSearch', () => {
  it('resets the page on every setter', () => {
    const { result } = renderHook(() => useVehicleListSearch())

    result.current.setQuery('iveco')
    expect(appliedSearch().page).toBe(1)

    result.current.setFilter('types', ['trailer'])
    expect(appliedSearch().page).toBe(1)

    result.current.setSort('water_capacity', 'desc')
    expect(appliedSearch().page).toBe(1)

    result.current.setArchive('only')
    expect(appliedSearch().page).toBe(1)

    result.current.resetFilters()
    expect(appliedSearch().page).toBe(1)
  })

  // Added beyond the template: the page-reset test above would still pass if a
  // setter wrote to the wrong field (e.g. `type` instead of `types`), so each
  // setter is checked here for the field it actually claims to write.
  it('writes the field each setter names, not just the page reset', () => {
    const { result } = renderHook(() => useVehicleListSearch())

    result.current.setQuery('iveco')
    expect(appliedSearch().q).toBe('iveco')

    result.current.setFilter('statuses', ['active'])
    expect(appliedSearch().statuses).toEqual(['active'])

    result.current.setFilter('types', ['trailer'])
    expect(appliedSearch().types).toEqual(['trailer'])

    result.current.setFilter('drivingLicenses', ['BE'])
    expect(appliedSearch().drivingLicenses).toEqual(['BE'])

    result.current.setSort('water_capacity', 'desc')
    expect(appliedSearch().sort).toBe('water_capacity')
    expect(appliedSearch().order).toBe('desc')

    result.current.setArchive('only')
    expect(appliedSearch().archive).toBe('only')

    result.current.resetFilters()
    const reset = appliedSearch()
    expect(reset.q).toBeUndefined()
    expect(reset.statuses).toBeUndefined()
    expect(reset.types).toBeUndefined()
    expect(reset.drivingLicenses).toBeUndefined()
    expect(reset.archive).toBeUndefined()
  })

  it('drops an empty filter instead of writing an empty array to the url', () => {
    const { result } = renderHook(() => useVehicleListSearch())

    result.current.setFilter('types', [])

    expect(appliedSearch().types).toBeUndefined()
  })

  it('drops the archive filter when set back to the default', () => {
    const { result } = renderHook(() => useVehicleListSearch())

    result.current.setArchive(undefined)

    expect(appliedSearch().archive).toBeUndefined()
    expect(appliedSearch().page).toBe(1)
  })

  it('normalises a hand-edited ?archive=active to the default state', () => {
    search.archive = ListVehiclesArchiveEnum.Active

    try {
      const { result } = renderHook(() => useVehicleListSearch())

      expect(result.current.search.archive).toBeUndefined()
    } finally {
      search.archive = undefined
    }
  })
})
