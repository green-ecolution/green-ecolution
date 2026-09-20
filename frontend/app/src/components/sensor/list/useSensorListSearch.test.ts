import { renderHook } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

const navigate = vi.fn((_options: unknown) => Promise.resolve())
const search = {
  page: 3,
  q: 'eui',
  statuses: undefined,
  modelIds: undefined,
  dataHealth: undefined,
  hasTree: true,
}

vi.mock('@tanstack/react-router', () => ({
  getRouteApi: () => ({
    useSearch: () => search,
    useNavigate: () => navigate,
  }),
}))

import { useSensorListSearch } from './useSensorListSearch'

const appliedSearch = () => {
  const calls = navigate.mock.calls
  const call = calls[calls.length - 1]?.[0] as {
    search: (prev: Record<string, unknown>) => Record<string, unknown>
  }
  return call.search({ ...search })
}

describe('useSensorListSearch', () => {
  it('resets the page on every setter', () => {
    const { result } = renderHook(() => useSensorListSearch())

    result.current.setQuery('a81758')
    expect(appliedSearch().page).toBe(1)

    result.current.setFilter('statuses', ['offline'])
    expect(appliedSearch().page).toBe(1)

    result.current.setSort('last_reading', 'desc')
    expect(appliedSearch().page).toBe(1)

    result.current.setHasTree(false)
    expect(appliedSearch().page).toBe(1)

    result.current.resetFilters()
    expect(appliedSearch().page).toBe(1)
  })

  // Added beyond the template: the page-reset test above would still pass if a
  // setter wrote to the wrong field (e.g. `modelId` instead of `modelIds`), so
  // each setter is checked here for the field it actually claims to write.
  it('writes the field each setter names, not just the page reset', () => {
    const { result } = renderHook(() => useSensorListSearch())

    result.current.setQuery('a81758')
    expect(appliedSearch().q).toBe('a81758')

    result.current.setFilter('statuses', ['offline'])
    expect(appliedSearch().statuses).toEqual(['offline'])

    result.current.setFilter('modelIds', ['model-1'])
    expect(appliedSearch().modelIds).toEqual(['model-1'])

    result.current.setFilter('dataHealth', ['suspect'])
    expect(appliedSearch().dataHealth).toEqual(['suspect'])

    result.current.setFilter('clusterIds', ['cluster-1'])
    expect(appliedSearch().clusterIds).toEqual(['cluster-1'])

    result.current.setSort('last_reading', 'desc')
    expect(appliedSearch().sort).toBe('last_reading')
    expect(appliedSearch().order).toBe('desc')

    result.current.setHasTree(false)
    expect(appliedSearch().hasTree).toBe(false)

    result.current.resetFilters()
    const reset = appliedSearch()
    expect(reset.q).toBeUndefined()
    expect(reset.statuses).toBeUndefined()
    expect(reset.modelIds).toBeUndefined()
    expect(reset.dataHealth).toBeUndefined()
    expect(reset.hasTree).toBeUndefined()
    expect(reset.clusterIds).toBeUndefined()
  })

  it('drops an empty filter instead of writing an empty array to the url', () => {
    const { result } = renderHook(() => useSensorListSearch())

    result.current.setFilter('statuses', [])

    expect(appliedSearch().statuses).toBeUndefined()
  })

  it('removes the tree filter from the url when cleared', () => {
    const { result } = renderHook(() => useSensorListSearch())

    result.current.setHasTree(undefined)

    expect(appliedSearch().hasTree).toBeUndefined()
  })
})
