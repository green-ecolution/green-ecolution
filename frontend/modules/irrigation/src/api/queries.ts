import { queryOptions, keepPreviousData, type QueryKey } from '@tanstack/react-query'
import {
  ClusterBoundaryListResponse,
  ClusterMarkerListResponse,
  ClusterStatisticsResponse,
  EvaluationResponse,
  IrrigationSettingsResponse,
  ListClustersRequest,
  ListResponseTreeClusterInListResponse,
  ListResponseTreeResponse,
  ListTreeMarkersRequest,
  ListTreesRequest,
  NearestTreeListResponse,
  SoilMoistureSeriesResponse,
  TreeClusterResponse,
  TreeMarkerListResponse,
  TreeResponse,
  WateringStatus,
} from '@green-ecolution/backend-client'
import { isValidUuid, sensorQueries, type BoundingBox } from '@/api/queries'
import { clusterApi, evaluationApi, irrigationApi, treeApi } from './client'

const formatBBox = (b: BoundingBox): string =>
  `${b.swLat.toFixed(5)},${b.swLng.toFixed(5)},${b.neLat.toFixed(5)},${b.neLng.toFixed(5)}`

/**
 * Every key prefix the irrigation aggregates own. The roots grew inconsistent
 * ('treecluster' vs 'treeclusters' vs 'clusters'), so invalidate through this
 * table instead of guessing a prefix.
 */
export const irrigationRoots = {
  cluster: [['treecluster'], ['treeclusters'], ['clusters']],
  tree: [['tree'], ['trees'], ['planting-years']],
  evaluation: [['evaluation']],
} as const satisfies Record<string, readonly QueryKey[]>

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

  suggested: () =>
    queryOptions<ListResponseTreeClusterInListResponse>({
      queryKey: ['treeclusters', 'list', { wateringStatus: [WateringStatus.Bad] }],
      queryFn: () =>
        clusterApi.listClusters({ wateringStatus: [WateringStatus.Bad], page: 1, perPage: 50 }),
    }),

  /** Sensors of the given clusters; lives under the sensor key so sensor invalidation reaches it. */
  sensorIds: (clusterIds: string[]) => {
    const sortedIds = clusterIds.slice().sort()
    return queryOptions<string[]>({
      queryKey: [...sensorQueries.key, 'by-cluster', sortedIds],
      queryFn: async () => {
        const perCluster = await Promise.all(
          sortedIds.map((clusterId) => clusterApi.listClusterSensorIds({ clusterId })),
        )
        return [...new Set(perCluster.flat())]
      },
    })
  },
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

export const evaluationQuery = () =>
  queryOptions<EvaluationResponse>({
    queryKey: ['evaluation'],
    queryFn: () => evaluationApi.getEvaluation(),
  })

// Below the core settings key, so invalidating those reaches this one too.
export const irrigationSettingsQueries = {
  byOrganization: (orgId: string) =>
    queryOptions<IrrigationSettingsResponse>({
      queryKey: ['organizations', orgId, 'settings', 'irrigation'],
      queryFn: () => irrigationApi.getIrrigationSettings({ orgId }),
    }),
}
