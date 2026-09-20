import { renderHook } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

const navigate = vi.fn((_options: unknown) => Promise.resolve())
const search = {
  page: 4,
  q: 'allee',
  wateringStatuses: undefined,
  regions: undefined,
  soil: undefined,
}

vi.mock('@tanstack/react-router', () => ({
  getRouteApi: () => ({
    useSearch: () => search,
    useNavigate: () => navigate,
  }),
}))

import { useClusterListSearch } from './useClusterListSearch'

const appliedSearch = () => {
  const calls = navigate.mock.calls
  const call = calls[calls.length - 1]?.[0] as {
    search: (prev: Record<string, unknown>) => Record<string, unknown>
  }
  return call.search({ ...search })
}

describe('useClusterListSearch', () => {
  it('resets the page on every setter', () => {
    const { result } = renderHook(() => useClusterListSearch())

    result.current.setQuery('bahnhof')
    expect(appliedSearch().page).toBe(1)

    result.current.setFilter('regions', ['a'])
    expect(appliedSearch().page).toBe(1)

    result.current.setSort('name', 'desc')
    expect(appliedSearch().page).toBe(1)

    result.current.resetFilters()
    expect(appliedSearch().page).toBe(1)
  })

  // Added beyond the template: the page-reset test above would still pass if a
  // setter wrote to the wrong field (e.g. `region` instead of `regions`), so
  // each setter is checked here for the field it actually claims to write.
  it('writes the field each setter names, not just the page reset', () => {
    const { result } = renderHook(() => useClusterListSearch())

    result.current.setQuery('bahnhof')
    expect(appliedSearch().q).toBe('bahnhof')

    result.current.setFilter('wateringStatuses', ['good'])
    expect(appliedSearch().wateringStatuses).toEqual(['good'])

    result.current.setFilter('regions', ['a'])
    expect(appliedSearch().regions).toEqual(['a'])

    result.current.setFilter('soil', ['sand'])
    expect(appliedSearch().soil).toEqual(['sand'])

    result.current.setSort('name', 'desc')
    expect(appliedSearch().sort).toBe('name')
    expect(appliedSearch().order).toBe('desc')

    result.current.resetFilters()
    const reset = appliedSearch()
    expect(reset.q).toBeUndefined()
    expect(reset.wateringStatuses).toBeUndefined()
    expect(reset.regions).toBeUndefined()
    expect(reset.soil).toBeUndefined()
  })

  it('drops an empty filter instead of writing an empty array to the url', () => {
    const { result } = renderHook(() => useClusterListSearch())

    result.current.setFilter('regions', [])

    expect(appliedSearch().regions).toBeUndefined()
  })
})
