import { keepPreviousData, useQuery } from '@tanstack/react-query'
import type { ListResponseSensorResponse } from '@green-ecolution/backend-client'
import { sensorQueries } from '@/api/queries'
import { sensorListParams, type SensorListSearch } from './sensorListView'

type SensorListResult = Pick<ListResponseSensorResponse, 'data'> &
  Partial<Pick<ListResponseSensorResponse, 'pagination'>>

export const useSensorList = (search: SensorListSearch) => {
  const clusterIds = search.clusterIds ?? []
  const filtersByCluster = clusterIds.length > 0

  const idsQuery = useQuery({
    ...sensorQueries.clusterSensorIds(clusterIds),
    enabled: filtersByCluster,
  })
  const clusterSensorIds = idsQuery.data
  const clustersHaveNoSensors = filtersByCluster && clusterSensorIds?.length === 0

  const listQuery = useQuery({
    ...sensorQueries.list(sensorListParams(search, clusterSensorIds)),
    enabled: !filtersByCluster || (clusterSensorIds?.length ?? 0) > 0,
    placeholderData: keepPreviousData,
  })

  const result: SensorListResult | undefined = clustersHaveNoSensors ? { data: [] } : listQuery.data

  return {
    result,
    isPlaceholderData: listQuery.isPlaceholderData,
    error: idsQuery.error ?? listQuery.error,
  }
}
