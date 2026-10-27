import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook } from '@testing-library/react'

const { navigateMock, searchMock } = vi.hoisted(() => {
  const searchMock: { current: Record<string, unknown> } = { current: {} }
  return { navigateMock: vi.fn((_options: unknown) => Promise.resolve()), searchMock }
})

// useTreeListSearch resolves its route via getRouteApi (not a direct Route
// import, see the hook's own comment), so the mock has to replace that
// lookup rather than the route module itself.
vi.mock('@tanstack/react-router', async () => {
  const actual =
    await vi.importActual<typeof import('@tanstack/react-router')>('@tanstack/react-router')
  return {
    ...actual,
    getRouteApi: () => ({
      useSearch: () => searchMock.current,
      useNavigate: () => navigateMock,
    }),
  }
})

const { useTreeListSearch } = await import('./useTreeListSearch')

const lastNavigateArg = () => {
  const call = navigateMock.mock.calls[0]
  if (!call) throw new Error('navigate was not called')
  return call[0] as {
    search: (prev: Record<string, unknown>) => Record<string, unknown>
    replace?: boolean
  }
}

describe('useTreeListSearch', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    searchMock.current = { page: 7 }
  })

  it('setQuery resets the page to 1 and replaces the history entry', () => {
    const { result } = renderHook(() => useTreeListSearch())

    result.current.setQuery('oak')

    const { search, replace } = lastNavigateArg()
    expect(search({ page: 7 })).toMatchObject({ page: 1 })
    expect(replace).toBe(true)
  })

  it('setSort resets the page to 1 without replacing the history entry', () => {
    const { result } = renderHook(() => useTreeListSearch())

    result.current.setSort('species', 'desc')

    const { search, replace } = lastNavigateArg()
    expect(search({ page: 7 })).toMatchObject({ page: 1 })
    expect(replace).toBe(false)
  })

  it('setFilter resets the page to 1 without replacing the history entry', () => {
    const { result } = renderHook(() => useTreeListSearch())

    result.current.setFilter('wateringStatuses', ['good'])

    const { search, replace } = lastNavigateArg()
    expect(search({ page: 7 })).toMatchObject({ page: 1 })
    expect(replace).toBe(false)
  })

  it('setHasSensor resets the page to 1 without replacing the history entry', () => {
    const { result } = renderHook(() => useTreeListSearch())

    result.current.setHasSensor(true)

    const { search, replace } = lastNavigateArg()
    expect(search({ page: 7 })).toMatchObject({ page: 1 })
    expect(replace).toBe(false)
  })

  it('setHasCluster resets the page to 1 without replacing the history entry', () => {
    const { result } = renderHook(() => useTreeListSearch())

    result.current.setHasCluster(false)

    const { search, replace } = lastNavigateArg()
    expect(search({ page: 7 })).toMatchObject({ page: 1 })
    expect(replace).toBe(false)
  })

  it('setClusterAssignment resets the page to 1 without replacing the history entry', () => {
    const { result } = renderHook(() => useTreeListSearch())

    result.current.setClusterAssignment({ clusterIds: ['cluster-1'] })

    const { search, replace } = lastNavigateArg()
    expect(search({ page: 7 })).toMatchObject({ page: 1 })
    expect(replace).toBe(false)
  })

  it('resetFilters resets the page to 1 without replacing the history entry', () => {
    const { result } = renderHook(() => useTreeListSearch())

    result.current.resetFilters()

    const { search, replace } = lastNavigateArg()
    expect(search({ page: 7 })).toMatchObject({ page: 1 })
    expect(replace).toBe(false)
  })
})
