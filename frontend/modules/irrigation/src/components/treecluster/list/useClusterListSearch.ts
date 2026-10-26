import { useCallback } from 'react'
import { getRouteApi } from '@tanstack/react-router'
import type { ListClustersSortEnum } from '@green-ecolution/backend-client'
import type { SortDirection } from '@/components/general/list/ListSortMenu'

// getRouteApi instead of importing Route: the route module imports this hook,
// so a direct import would close a cycle.
const routeApi = getRouteApi('/_protected/treecluster/')

type ClusterSearch = ReturnType<typeof routeApi.useSearch>

export const useClusterListSearch = () => {
  const search = routeApi.useSearch()
  const navigate = routeApi.useNavigate()

  // Every setter resets `page`: a filter change can shrink the result below
  // the current page, which would otherwise render an empty list.
  const patch = useCallback(
    (next: Partial<ClusterSearch>, replace = false) => {
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
    (field: ListClustersSortEnum, direction: SortDirection) =>
      patch({ sort: field, order: direction }),
    [patch],
  )

  const setFilter = useCallback(
    (key: 'wateringStatuses' | 'regions' | 'soil', values: string[]) =>
      patch({ [key]: values.length > 0 ? values : undefined }),
    [patch],
  )

  const resetFilters = useCallback(
    () =>
      patch({
        q: undefined,
        wateringStatuses: undefined,
        regions: undefined,
        soil: undefined,
      }),
    [patch],
  )

  return { search, setQuery, setSort, setFilter, resetFilters }
}
