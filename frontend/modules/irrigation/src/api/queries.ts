import { queryOptions, keepPreviousData } from '@tanstack/react-query'
import {
  ClusterBoundaryListResponse,
  ClusterMarkerListResponse,
  ClusterStatisticsResponse,
  EvaluationResponse,
  ListClustersRequest,
  ListResponseTreeClusterInListResponse,
  ListResponseTreeResponse,
  ListTreeMarkersRequest,
  ListTreesRequest,
  NearestTreeListResponse,
  SoilMoistureSeriesResponse,
  TreeClusterInListResponse,
  TreeClusterResponse,
  TreeMarkerListResponse,
  TreeResponse,
  WateringStatus,
} from '@green-ecolution/backend-client'
import { sensorApi } from '@/api/backendApi'
import { isValidUuid, type BoundingBox } from '@/api/queries'
import { clusterApi, evaluationApi, treeApi } from './client'
import { loadStopClusters } from './plans'

const formatBBox = (b: BoundingBox): string =>
  `${b.swLat.toFixed(5)},${b.swLng.toFixed(5)},${b.neLat.toFixed(5)},${b.neLng.toFixed(5)}`

export { irrigationRoots } from './roots'

export interface TreeMarkersFilters {
  hasCluster?: boolean
  plantingYears?: number[]
  wateringStatuses?: WateringStatus[]
  organizationId?: string
}

export const clusterQueries = {
  list: (params?: ListClustersRequest) =>
    queryOptions<ListResponseTreeClusterInListResponse>({
      queryKey: ['treeclusters', 'list', params ?? {}],
      queryFn: () => clusterApi.listClusters(params),
    }),

  statistics: () =>
    queryOptions<ClusterStatisticsResponse>({
      queryKey: ['clusters', 'statistics'],
      queryFn: () => clusterApi.getClusterStatistics(),
      staleTime: 60_000,
    }),

  detail: (id: string) =>
    queryOptions<TreeClusterResponse>({
      queryKey: ['treecluster', id],
      queryFn: () => clusterApi.getCluster({ clusterId: id }),
      enabled: isValidUuid(id),
    }),

  markers: () =>
    queryOptions<ClusterMarkerListResponse>({
      queryKey: ['clusters', 'markers'],
      queryFn: () => clusterApi.listClusterMarkers(),
      staleTime: 5 * 60_000,
    }),

  boundaries: () =>
    queryOptions<ClusterBoundaryListResponse>({
      queryKey: ['clusters', 'boundaries'],
      queryFn: () => clusterApi.listClusterBoundaries(),
      staleTime: 5 * 60_000,
    }),

  soilMoisture: (id: string, params: { from?: Date; bucket: 'hour' | 'day' }) =>
    queryOptions<SoilMoistureSeriesResponse>({
      queryKey: [
        'treecluster',
        id,
        'soil-moisture',
        params.bucket,
        params.from?.toISOString() ?? 'default',
      ],
      queryFn: () =>
        clusterApi.getClusterSoilMoisture({
          clusterId: id,
          from: params.from,
          bucket: params.bucket,
        }),
      enabled: isValidUuid(id),
    }),

  /**
   * The clusters behind a plan's stops, in stop order, without those the reader
   * cannot see. Carries the stops it answers for, so a kept previous answer
   * still tells which of them stayed unresolved.
   */
  stops: (stops: string[]) =>
    queryOptions<{ stops: string[]; clusters: TreeClusterInListResponse[] }>({
      queryKey: ['treeclusters', 'stops', stops],
      queryFn: async () => ({ stops, clusters: await loadStopClusters(stops) }),
    }),

  suggested: () =>
    queryOptions<ListResponseTreeClusterInListResponse>({
      queryKey: ['treeclusters', 'list', { wateringStatus: [WateringStatus.Bad] }],
      queryFn: () =>
        clusterApi.listClusters({ wateringStatus: [WateringStatus.Bad], page: 1, perPage: 50 }),
    }),
}

/** Feeds the sensor list's cluster filter, see `sensorTargets.ts`. */
export const fetchSensorIdsOfClusters = async (clusterIds: string[]): Promise<string[]> => {
  const perCluster = await Promise.all(
    clusterIds.map((clusterId) => clusterApi.listClusterSensorIds({ clusterId })),
  )
  return [...new Set(perCluster.flat())]
}

export const treeQueries = {
  list: (params?: ListTreesRequest) =>
    queryOptions<ListResponseTreeResponse>({
      queryKey: ['trees', 'list', params ?? {}],
      queryFn: () => treeApi.listTrees(params),
    }),

  detail: (id: string) =>
    queryOptions<TreeResponse>({
      queryKey: ['tree', id],
      queryFn: () => treeApi.getTree({ treeId: id }),
      enabled: isValidUuid(id),
    }),

  markers: (params: { bbox: BoundingBox } & TreeMarkersFilters) =>
    queryOptions<TreeMarkerListResponse>({
      queryKey: [
        'trees',
        'markers',
        formatBBox(params.bbox),
        {
          hasCluster: params.hasCluster,
          plantingYears: params.plantingYears,
          wateringStatuses: params.wateringStatuses,
          organizationId: params.organizationId,
        },
      ],
      queryFn: () =>
        treeApi.listTreeMarkers({
          bbox: formatBBox(params.bbox),
          hasCluster: params.hasCluster,
          plantingYear: params.plantingYears,
          wateringStatus: params.wateringStatuses,
          organizationId: params.organizationId,
        } satisfies ListTreeMarkersRequest),
      placeholderData: keepPreviousData,
      staleTime: 30_000,
    }),

  plantingYears: () =>
    queryOptions<number[]>({
      queryKey: ['planting-years'],
      queryFn: () => treeApi.listPlantingYears(),
    }),

  nearest: (params: { lat: number; lng: number; limit?: number }) =>
    queryOptions<NearestTreeListResponse>({
      queryKey: ['trees', 'nearest', params.lat, params.lng, params.limit],
      queryFn: () =>
        treeApi.getNearestTrees({ lat: params.lat, lng: params.lng, limit: params.limit }),
    }),
}

/** A module route, read against the soil of the tree's cluster; keyed below the sensor so its invalidation reaches it. */
export const sensorSoilMoistureQuery = (
  id: string,
  params: { from?: Date; bucket: 'hour' | 'day' },
) =>
  queryOptions<SoilMoistureSeriesResponse>({
    queryKey: [
      'sensor',
      id,
      'soil-moisture',
      params.bucket,
      params.from?.toISOString() ?? 'default',
    ],
    queryFn: () =>
      sensorApi.getSensorSoilMoisture({
        sensorId: id,
        from: params.from,
        bucket: params.bucket,
      }),
  })

export const evaluationQuery = () =>
  queryOptions<EvaluationResponse>({
    queryKey: ['evaluation'],
    queryFn: () => evaluationApi.getEvaluation(),
  })
