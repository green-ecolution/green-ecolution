import { useCallback } from 'react'
import { getRouteApi } from '@tanstack/react-router'
import type { ListSensorsSortEnum } from '@green-ecolution/backend-client'
import type { SortDirection } from '@/components/general/list/ListSortMenu'

// getRouteApi instead of importing Route: the route module imports this hook,
// so a direct import would close a cycle.
const routeApi = getRouteApi('/_protected/sensors/')

type SensorSearch = ReturnType<typeof routeApi.useSearch>

export const useSensorListSearch = () => {
  const search = routeApi.useSearch()
  const navigate = routeApi.useNavigate()

  // Every setter resets `page`: a filter change can shrink the result below
  // the current page, which would otherwise render an empty list.
  const patch = useCallback(
    (next: Partial<SensorSearch>, replace = false) => {
      navigate({
        search: (prev) => ({ ...prev, ...next, page: 1 }),
        replace,
      }).catch((error) => console.error('Navigation failed:', error))
    },
    [navigate],
  )

  const setQuery = useCallback(
    (q: string) => patch({ q: q.length > 0 ? q : undefined }, true),
    [patch],
  )

  const setSort = useCallback(
    (field: ListSensorsSortEnum, direction: SortDirection) =>
      patch({ sort: field, order: direction }),
    [patch],
  )

  const setFilter = useCallback(
    (key: 'statuses' | 'modelIds' | 'dataHealth', values: string[]) =>
      patch({ [key]: values.length > 0 ? values : undefined }),
    [patch],
  )

  const setHasTree = useCallback((value: boolean | undefined) => patch({ hasTree: value }), [patch])

  const resetFilters = useCallback(
    () =>
      patch({
        q: undefined,
        statuses: undefined,
        modelIds: undefined,
        dataHealth: undefined,
        hasTree: undefined,
      }),
    [patch],
  )

  return { search, setQuery, setSort, setFilter, setHasTree, resetFilters }
}
