import { useCallback } from 'react'
import { getRouteApi } from '@tanstack/react-router'
import type { SortDirection } from '@/components/general/list/ListSortMenu'

export type TreeSortField =
  | 'number'
  | 'species'
  | 'status'
  | 'planting_year'
  | 'last_watered'
  | 'cluster'

// getRouteApi instead of importing Route: the route module imports this hook,
// so a direct import would close a cycle.
const routeApi = getRouteApi('/_protected/trees/')

type TreeSearch = ReturnType<typeof routeApi.useSearch>

export const useTreeListSearch = () => {
  const search = routeApi.useSearch()
  const navigate = routeApi.useNavigate()

  // Every setter resets `page` to 1: a filter change can shrink the result
  // below the current page, which would otherwise render an empty list.
  const patch = useCallback(
    (next: Partial<TreeSearch>, replace = false) => {
      void navigate({
        search: (prev) => ({ ...prev, ...next, page: 1 }),
        replace,
      })
    },
    [navigate],
  )

  const setQuery = useCallback(
    (q: string) => patch({ q: q.length > 0 ? q : undefined }, true),
    [patch],
  )

  const setSort = useCallback(
    (field: TreeSortField, direction: SortDirection) => patch({ sort: field, order: direction }),
    [patch],
  )

  const setFilter = useCallback(
    (key: 'wateringStatuses' | 'clusterIds' | 'plantingYears', values: unknown[]) =>
      patch({ [key]: values.length > 0 ? values : undefined }),
    [patch],
  )

  const setHasSensor = useCallback(
    (value: boolean | undefined) => patch({ hasSensor: value }),
    [patch],
  )

  const setHasCluster = useCallback(
    (value: boolean | undefined) => patch({ hasCluster: value }),
    [patch],
  )

  // clusterIds and hasCluster are mutually exclusive; patched together so a
  // switch between them is one navigation, not two racing ones.
  const setClusterAssignment = useCallback(
    (next: { clusterIds?: string[]; hasCluster?: boolean }) =>
      patch({
        clusterIds: next.clusterIds && next.clusterIds.length > 0 ? next.clusterIds : undefined,
        hasCluster: next.hasCluster,
      }),
    [patch],
  )

  const resetFilters = useCallback(
    () =>
      patch({
        q: undefined,
        wateringStatuses: undefined,
        clusterIds: undefined,
        plantingYears: undefined,
        hasCluster: undefined,
        hasSensor: undefined,
      }),
    [patch],
  )

  return {
    search,
    setQuery,
    setSort,
    setFilter,
    setHasSensor,
    setHasCluster,
    setClusterAssignment,
    resetFilters,
  }
}
