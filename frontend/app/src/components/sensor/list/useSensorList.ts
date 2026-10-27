import { keepPreviousData, useQueries, useQuery } from '@tanstack/react-query'
import type { ListResponseSensorResponse } from '@green-ecolution/backend-client'
import { sensorQueries } from '@/api/queries'
import { listFilterValue, sensorListFiltersOf } from '@/lib/targets'
import { useActiveModules } from '@/modules/useActiveModules'
import { sensorListParams, type SensorListSearch } from './sensorListView'

type SensorListResult = Pick<ListResponseSensorResponse, 'data'> &
  Partial<Pick<ListResponseSensorResponse, 'pagination'>>

/** Sensors every set filter allows; undefined while one is still resolving. */
const intersect = (idSets: (string[] | undefined)[]): string[] | undefined => {
  if (idSets.some((ids) => ids === undefined)) return undefined
  const [first, ...rest] = idSets as string[][]
  return first.filter((id) => rest.every((ids) => ids.includes(id)))
}

export const useSensorList = (search: SensorListSearch) => {
  const { manifests } = useActiveModules()
  const setFilters = sensorListFiltersOf(manifests)
    .map(({ filter }) => ({ filter, value: listFilterValue(search, filter.param) }))
    .filter(({ value }) => value.length > 0)
  const filtersByTarget = setFilters.length > 0

  const idQueries = useQueries({
    queries: setFilters.map(({ filter, value }) => filter.sensorIds(value)),
  })
  const filteredSensorIds = filtersByTarget
    ? intersect(idQueries.map((query) => query.data))
    : undefined
  const noSensorMatches = filtersByTarget && filteredSensorIds?.length === 0

  const listQuery = useQuery({
    ...sensorQueries.list(sensorListParams(search, filteredSensorIds)),
    enabled: !filtersByTarget || (filteredSensorIds?.length ?? 0) > 0,
    placeholderData: keepPreviousData,
  })

  const result: SensorListResult | undefined = noSensorMatches ? { data: [] } : listQuery.data

  return {
    result,
    isPlaceholderData: listQuery.isPlaceholderData,
    error: idQueries.find((query) => query.error)?.error ?? listQuery.error,
  }
}
