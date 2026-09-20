import { useCallback, useMemo } from 'react'
import { getRouteApi } from '@tanstack/react-router'
import { ListVehiclesArchiveEnum, type ListVehiclesSortEnum } from '@green-ecolution/backend-client'
import type { SortDirection } from '@/components/general/list/ListSortMenu'

// getRouteApi instead of importing Route: the route module imports this hook,
// so a direct import would close a cycle.
const routeApi = getRouteApi('/_protected/vehicles/')

type VehicleSearch = ReturnType<typeof routeApi.useSearch>

export const useVehicleListSearch = () => {
  const rawSearch = routeApi.useSearch()
  const navigate = routeApi.useNavigate()

  // A hand-edited URL can carry the enum's explicit default value; treat it
  // the same as the implicit default so toolbar and filter logic agree.
  const search = useMemo(
    () =>
      rawSearch.archive === ListVehiclesArchiveEnum.Active
        ? { ...rawSearch, archive: undefined }
        : rawSearch,
    [rawSearch],
  )

  // Every setter resets `page`: a filter change can shrink the result below
  // the current page, which would otherwise render an empty list.
  const patch = useCallback(
    (next: Partial<VehicleSearch>, replace = false) => {
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
    (field: ListVehiclesSortEnum, direction: SortDirection) =>
      patch({ sort: field, order: direction }),
    [patch],
  )

  const setFilter = useCallback(
    (key: 'statuses' | 'types' | 'drivingLicenses', values: string[]) =>
      patch({ [key]: values.length > 0 ? values : undefined }),
    [patch],
  )

  const setArchive = useCallback(
    (value: ListVehiclesArchiveEnum | undefined) => patch({ archive: value }),
    [patch],
  )

  const resetFilters = useCallback(
    () =>
      patch({
        q: undefined,
        statuses: undefined,
        types: undefined,
        drivingLicenses: undefined,
        archive: undefined,
      }),
    [patch],
  )

  return { search, setQuery, setSort, setFilter, setArchive, resetFilters }
}
