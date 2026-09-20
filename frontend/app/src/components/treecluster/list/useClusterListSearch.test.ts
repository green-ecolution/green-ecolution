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

  it('drops an empty filter instead of writing an empty array to the url', () => {
    const { result } = renderHook(() => useClusterListSearch())

    result.current.setFilter('regions', [])

    expect(appliedSearch().regions).toBeUndefined()
  })
})
