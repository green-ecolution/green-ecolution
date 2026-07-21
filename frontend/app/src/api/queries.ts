import { queryOptions, infiniteQueryOptions, keepPreviousData } from '@tanstack/react-query'
import {
  AppInfoResponse,
  clusterApi,
  ClusterBoundaryListResponse,
  ClusterMarkerListResponse,
  ClusterStatisticsResponse,
  DataStatisticsResponse,
  EvaluationResponse,
  infoApi,
  ListClustersRequest,
  ListResponseSensorDataResponse,
  ListResponseSensorResponse,
  ListResponseTreeClusterInListResponse,
  ListResponseTreeResponse,
  ListResponseUserResponse,
  ListResponseVehicleResponse,
  ListResponseWateringPlanInListResponse,
  ListSensorsRequest,
  ListTreeMarkersRequest,
  ListTreesRequest,
  ListUsersRequest,
  ListVehiclesRequest,
  ListWateringPlansRequest,
  MapInfoResponse,
  NearestTreeListResponse,
  pluginApi,
  regionApi,
  ResponseError,
  RouteResponse,
  routingApi,
  SensorModelResponse,
  SensorResponse,
  sensorApi,
  ServerInfoResponse,
  ServicesInfoResponse,
  SoilMoistureSeriesResponse,
  StartPointResponse,
  TreeClusterResponse,
  TreeMarkerListResponse,
  TreeResponse,
  treeApi,
  userApi,
  UserResponse,
  vehicleApi,
  VehicleResponse,
  WateringPlanResponse,
  WateringPlanStatus,
  WateringStatus,
  wateringPlanApi,
  wateringPlanPreviewApi,
  evaluationApi,
} from './backendApi'
import { DONE_STATUSES } from '@/lib/wateringPlanBoard'

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

/**
 * Check if a string is a valid UUID. Backend identifiers are UUID v7;
 * the regex accepts any UUID version since the frontend never inspects bits.
 */
export const isValidUuid = (id: string | undefined): boolean =>
  typeof id === 'string' && UUID_RE.test(id)

export const treeClusterQuery = (params?: ListClustersRequest) =>
  queryOptions<ListResponseTreeClusterInListResponse>({
    queryKey: ['treeclusters', 'list', params ?? {}],
    queryFn: () => clusterApi.listClusters(params),
  })

export const clusterStatisticsQuery = () =>
  queryOptions<ClusterStatisticsResponse>({
    queryKey: ['clusters', 'statistics'],
    queryFn: () => clusterApi.getClusterStatistics(),
    staleTime: 60_000,
  })

export const treeClusterIdQuery = (id: string) =>
  queryOptions<TreeClusterResponse>({
    queryKey: ['treecluster', id],
    queryFn: () => clusterApi.getCluster({ clusterId: id }),
    enabled: isValidUuid(id),
  })

/** Partial key matching every sensor list page; use for broad invalidation. */
export const sensorsKey = ['sensors'] as const

export const sensorQuery = (params?: ListSensorsRequest) =>
  queryOptions<ListResponseSensorResponse>({
    queryKey: [...sensorsKey, params?.page ?? '1'],
    queryFn: () => sensorApi.listSensors(params),
  })

// Sensor ids are LoRaWAN EUIs (e.g. "eui-a81758fffe0c3b52"), not UUIDs,
// so these queries only guard against empty ids.
export const sensorDataQuery = (id: string, params?: { from?: Date; perPage?: number }) =>
  queryOptions<ListResponseSensorDataResponse>({
    queryKey: [
      'sensor data',
      id,
      params?.perPage ?? 'default',
      params?.from?.toISOString() ?? 'all',
    ],
    queryFn: () =>
      sensorApi.listSensorData({
        sensorId: id,
        from: params?.from,
        perPage: params?.perPage,
      }),
    enabled: id !== '',
  })

export const sensorIdQuery = (id: string) =>
  queryOptions<SensorResponse>({
    queryKey: ['sensor', id],
    queryFn: () =>
      sensorApi.getSensor({
        sensorId: id,
      }),
    enabled: id !== '',
  })

export const sensorModelIdQuery = (id: string) =>
  queryOptions<SensorModelResponse>({
    queryKey: ['sensor-model', id],
    queryFn: () => sensorApi.getSensorModel({ id }),
    enabled: isValidUuid(id),
  })

export const treeQuery = (params?: ListTreesRequest) =>
  queryOptions<ListResponseTreeResponse>({
    queryKey: ['trees', 'list', params ?? {}],
    queryFn: () => treeApi.listTrees(params),
  })

export const treeIdQuery = (id: string) =>
  queryOptions<TreeResponse>({
    queryKey: ['tree', id],
    queryFn: () => treeApi.getTree({ treeId: id }),
    enabled: isValidUuid(id),
  })

export const regionsQuery = () =>
  queryOptions({
    queryKey: ['regions'],
    queryFn: () => regionApi.listRegions(),
  })

export const infoQuery = () =>
  queryOptions<AppInfoResponse>({
    queryKey: ['info'],
    queryFn: () => infoApi.getInfo(),
  })

export const mapInfoQuery = () =>
  queryOptions<MapInfoResponse>({
    queryKey: ['info', 'map'],
    queryFn: () => infoApi.getMapInfo(),
  })

export const serverInfoQuery = () =>
  queryOptions<ServerInfoResponse>({
    queryKey: ['info', 'server'],
    queryFn: () => infoApi.getServerInfo(),
  })

export const servicesInfoQuery = () =>
  queryOptions<ServicesInfoResponse>({
    queryKey: ['info', 'services'],
    queryFn: () => infoApi.getServicesInfo(),
  })

export const statisticsQuery = () =>
  queryOptions<DataStatisticsResponse>({
    queryKey: ['info', 'statistics'],
    queryFn: () => infoApi.getStatistics(),
  })

export const evaluationQuery = () =>
  queryOptions<EvaluationResponse>({
    queryKey: ['evaluation'],
    queryFn: () => evaluationApi.getEvaluation(),
  })

export const vehicleQuery = (params?: ListVehiclesRequest) =>
  queryOptions<ListResponseVehicleResponse>({
    queryKey: ['vehicles', 'list', params ?? {}],
    queryFn: () => vehicleApi.listVehicles(params),
  })

export const vehicleIdQuery = (id: string) =>
  queryOptions<VehicleResponse>({
    queryKey: ['vehicle', id],
    queryFn: () => vehicleApi.getVehicle({ vehicleId: id }),
    enabled: isValidUuid(id),
  })

export const wateringPlanQuery = (params?: ListWateringPlansRequest) =>
  queryOptions<ListResponseWateringPlanInListResponse>({
    queryKey: ['watering-plans', params?.page ?? '1'],
    queryFn: () => wateringPlanApi.listWateringPlans(params),
  })

export const wateringPlanIdQuery = (id: string) =>
  queryOptions<WateringPlanResponse>({
    queryKey: ['watering-plan', id],
    queryFn: () => wateringPlanApi.getWateringPlan({ wateringPlanId: id }),
    enabled: isValidUuid(id),
  })

export const wateringPlanRouteQuery = (id: string) =>
  queryOptions<RouteResponse | null>({
    queryKey: ['watering-plan-route', id],
    queryFn: async () => {
      try {
        return await wateringPlanApi.getWateringPlanRoute({ wateringPlanId: id })
      } catch (error) {
        // 404: plan has no computed route; 503: routing feature disabled.
        if (error instanceof ResponseError && [404, 503].includes(error.response.status))
          return null
        throw error
      }
    },
    enabled: isValidUuid(id),
  })

export const routePreviewQuery = (
  clusterIds: string[],
  transporterId: string,
  startPointName?: string | null,
) =>
  queryOptions<RouteResponse | null>({
    queryKey: ['route-preview', clusterIds.slice().sort(), transporterId, startPointName ?? null],
    queryFn: async () => {
      try {
        return await wateringPlanPreviewApi.previewRoute({
          routeRequest: { clusterIds, transporterId, startPointName },
        })
      } catch {
        return null
      }
    },
    retry: false,
  })

export const currentUserQuery = () =>
  queryOptions<UserResponse>({
    queryKey: ['users', 'me'],
    queryFn: () => userApi.getMe(),
  })

export const userQuery = (params?: ListUsersRequest) => {
  return queryOptions<ListResponseUserResponse>({
    queryKey: ['users', params],
    queryFn: () => userApi.listUsers(params),
  })
}

export const userRoleQuery = (role: string) =>
  queryOptions<ListResponseUserResponse>({
    queryKey: ['user', role],
    queryFn: () =>
      userApi.listUsersByRole({
        roleId: role,
      }),
  })

export const pluginsQuery = () =>
  queryOptions({
    queryKey: ['plugins'],
    queryFn: () => pluginApi.listPlugins(),
  })

export const plantingYearsQuery = () =>
  queryOptions<number[]>({
    queryKey: ['planting-years'],
    queryFn: () => treeApi.listPlantingYears(),
  })

export const nearestTreeQuery = (params: { lat: number; lng: number; limit?: number }) =>
  queryOptions<NearestTreeListResponse>({
    queryKey: ['trees', 'nearest', params.lat, params.lng, params.limit],
    queryFn: () =>
      treeApi.getNearestTrees({ lat: params.lat, lng: params.lng, limit: params.limit }),
  })

export interface BoundingBox {
  swLat: number
  swLng: number
  neLat: number
  neLng: number
}

const formatBBox = (b: BoundingBox): string =>
  `${b.swLat.toFixed(5)},${b.swLng.toFixed(5)},${b.neLat.toFixed(5)},${b.neLng.toFixed(5)}`

export interface TreeMarkersFilters {
  hasCluster?: boolean
  plantingYears?: number[]
  wateringStatuses?: WateringStatus[]
}

export const treeMarkersQuery = (params: { bbox: BoundingBox } & TreeMarkersFilters) =>
  queryOptions<TreeMarkerListResponse>({
    queryKey: [
      'trees',
      'markers',
      formatBBox(params.bbox),
      {
        hasCluster: params.hasCluster,
        plantingYears: params.plantingYears,
        wateringStatuses: params.wateringStatuses,
      },
    ],
    queryFn: () =>
      treeApi.listTreeMarkers({
        bbox: formatBBox(params.bbox),
        hasCluster: params.hasCluster,
        plantingYear: params.plantingYears,
        wateringStatus: params.wateringStatuses,
      } satisfies ListTreeMarkersRequest),
    placeholderData: keepPreviousData,
    staleTime: 30_000,
  })

export const clusterMarkersQuery = () =>
  queryOptions<ClusterMarkerListResponse>({
    queryKey: ['clusters', 'markers'],
    queryFn: () => clusterApi.listClusterMarkers(),
    staleTime: 5 * 60_000,
  })

export const clusterBoundariesQuery = () =>
  queryOptions<ClusterBoundaryListResponse>({
    queryKey: ['clusters', 'boundaries'],
    queryFn: () => clusterApi.listClusterBoundaries(),
    staleTime: 5 * 60_000,
  })

export const routingStartPointsQuery = () =>
  queryOptions<StartPointResponse[] | null>({
    queryKey: ['routing-start-points'],
    queryFn: async () => {
      try {
        return await routingApi.listRoutingStartPoints()
      } catch (error) {
        if (error instanceof ResponseError && error.response.status === 503) return null
        throw error
      }
    },
    staleTime: Infinity,
  })

export const wateringPlanBoardColumnQuery = (statuses: WateringPlanStatus[]) =>
  queryOptions<ListResponseWateringPlanInListResponse>({
    queryKey: ['watering-plans', 'board', statuses],
    queryFn: () => wateringPlanApi.listWateringPlans({ status: statuses, page: 1, perPage: 100 }),
  })

export const wateringPlanBoardDoneQuery = () =>
  infiniteQueryOptions({
    queryKey: ['watering-plans', 'board', 'done'],
    queryFn: ({ pageParam }) =>
      wateringPlanApi.listWateringPlans({ status: DONE_STATUSES, page: pageParam, perPage: 20 }),
    initialPageParam: 1,
    getNextPageParam: (lastPage) =>
      lastPage.pagination?.nextPage ? lastPage.pagination.currentPage + 1 : undefined,
  })

export const suggestedClustersQuery = () =>
  queryOptions<ListResponseTreeClusterInListResponse>({
    queryKey: ['treeclusters', 'list', { wateringStatus: [WateringStatus.Bad] }],
    queryFn: () =>
      clusterApi.listClusters({ wateringStatus: [WateringStatus.Bad], page: 1, perPage: 50 }),
  })

export const clusterSoilMoistureQuery = (
  id: string,
  params: { from?: Date; bucket: 'hour' | 'day' },
) =>
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
  })

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
