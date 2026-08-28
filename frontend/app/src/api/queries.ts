import {
  queryOptions,
  infiniteQueryOptions,
  keepPreviousData,
  type QueryKey,
} from '@tanstack/react-query'
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
  organizationApi,
  OrganizationDetailResponse,
  OrganizationResponse,
  pluginApi,
  regionApi,
  ResponseError,
  RoleResponse,
  roleApi,
  RouteResponse,
  routingApi,
  SensorDataQualityResponse,
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
  organizationId?: string
}

/** Partial key matching every sensor list page; use for broad invalidation. */
const SENSORS_KEY = ['sensors'] as const

/**
 * Every key prefix an aggregate owns, grouped per aggregate. The keys below
 * grew inconsistent roots ('treecluster' vs 'treeclusters' vs 'clusters'), so
 * invalidating one prefix silently misses the rest — go through this table
 * instead of guessing. See lib/queryInvalidation.ts.
 */
export const queryRoots = {
  cluster: [['treecluster'], ['treeclusters'], ['clusters']],
  tree: [['tree'], ['trees'], ['planting-years']],
  vehicle: [['vehicle'], ['vehicles']],
  sensor: [['sensor'], ['sensors'], ['sensor data'], ['sensor-model']],
  wateringPlan: [['watering-plan'], ['watering-plans'], ['watering-plan-route'], ['route-preview']],
  evaluation: [['evaluation']],
  region: [['regions']],
  statistics: [['info']],
  user: [['users']],
  role: [['roles']],
  organization: [['organizations']],
} as const satisfies Record<string, readonly QueryKey[]>

export type Aggregate = keyof typeof queryRoots

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

export const vehicleQueries = {
  list: (params?: ListVehiclesRequest) =>
    queryOptions<ListResponseVehicleResponse>({
      queryKey: ['vehicles', 'list', params ?? {}],
      queryFn: () => vehicleApi.listVehicles(params),
    }),

  detail: (id: string) =>
    queryOptions<VehicleResponse>({
      queryKey: ['vehicle', id],
      queryFn: () => vehicleApi.getVehicle({ vehicleId: id }),
      enabled: isValidUuid(id),
    }),
}

export const sensorQueries = {
  /** Partial key matching every sensor list page; use for broad invalidation. */
  key: SENSORS_KEY,

  list: (params?: ListSensorsRequest) =>
    queryOptions<ListResponseSensorResponse>({
      queryKey: [...SENSORS_KEY, params?.page ?? '1'],
      queryFn: () => sensorApi.listSensors(params),
    }),

  // Sensor ids are LoRaWAN EUIs (e.g. "eui-a81758fffe0c3b52"), not UUIDs,
  // so these queries only guard against empty ids.
  data: (id: string, params?: { from?: Date; perPage?: number }) =>
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
    }),

  detail: (id: string) =>
    queryOptions<SensorResponse>({
      queryKey: ['sensor', id],
      queryFn: () =>
        sensorApi.getSensor({
          sensorId: id,
        }),
      enabled: id !== '',
    }),

  model: (id: string) =>
    queryOptions<SensorModelResponse>({
      queryKey: ['sensor-model', id],
      queryFn: () => sensorApi.getSensorModel({ id }),
      enabled: isValidUuid(id),
    }),

  dataQuality: (id: string) =>
    queryOptions<SensorDataQualityResponse>({
      queryKey: ['sensor', id, 'data-quality'],
      queryFn: () => sensorApi.getSensorDataQuality({ sensorId: id }),
      enabled: id !== '',
    }),

  soilMoisture: (id: string, params: { from?: Date; bucket: 'hour' | 'day' }) =>
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
    }),
}

export const wateringPlanQueries = {
  list: (params?: ListWateringPlansRequest) =>
    queryOptions<ListResponseWateringPlanInListResponse>({
      queryKey: ['watering-plans', params?.page ?? '1'],
      queryFn: () => wateringPlanApi.listWateringPlans(params),
    }),

  detail: (id: string) =>
    queryOptions<WateringPlanResponse>({
      queryKey: ['watering-plan', id],
      queryFn: () => wateringPlanApi.getWateringPlan({ wateringPlanId: id }),
      enabled: isValidUuid(id),
    }),

  route: (id: string) =>
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
    }),

  boardColumn: (statuses: WateringPlanStatus[]) =>
    queryOptions<ListResponseWateringPlanInListResponse>({
      queryKey: ['watering-plans', 'board', statuses],
      queryFn: () => wateringPlanApi.listWateringPlans({ status: statuses, page: 1, perPage: 100 }),
    }),

  boardDone: () =>
    infiniteQueryOptions({
      queryKey: ['watering-plans', 'board', 'done'],
      queryFn: ({ pageParam }) =>
        wateringPlanApi.listWateringPlans({ status: DONE_STATUSES, page: pageParam, perPage: 20 }),
      initialPageParam: 1,
      getNextPageParam: (lastPage) =>
        lastPage.pagination?.nextPage ? lastPage.pagination.currentPage + 1 : undefined,
    }),

  routePreview: (clusterIds: string[], transporterId: string, startPointName?: string | null) =>
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
    }),
}

export const infoQueries = {
  app: () =>
    queryOptions<AppInfoResponse>({
      queryKey: ['info'],
      queryFn: () => infoApi.getInfo(),
    }),

  map: () =>
    queryOptions<MapInfoResponse>({
      queryKey: ['info', 'map'],
      queryFn: () => infoApi.getMapInfo(),
    }),

  server: () =>
    queryOptions<ServerInfoResponse>({
      queryKey: ['info', 'server'],
      queryFn: () => infoApi.getServerInfo(),
    }),

  services: () =>
    queryOptions<ServicesInfoResponse>({
      queryKey: ['info', 'services'],
      queryFn: () => infoApi.getServicesInfo(),
    }),

  statistics: () =>
    queryOptions<DataStatisticsResponse>({
      queryKey: ['info', 'statistics'],
      queryFn: () => infoApi.getStatistics(),
    }),
}

export const userQueries = {
  me: () =>
    queryOptions<UserResponse>({
      queryKey: ['users', 'me'],
      queryFn: () => userApi.getMe(),
    }),

  list: (params?: ListUsersRequest) =>
    queryOptions<ListResponseUserResponse>({
      queryKey: ['users', params],
      queryFn: () => userApi.listUsers(params),
    }),
}

export const roleQueries = {
  templates: () =>
    queryOptions<RoleResponse[]>({
      queryKey: ['roles', 'templates'],
      queryFn: () => roleApi.listRoleTemplates(),
    }),

  org: (orgId: string) =>
    queryOptions<RoleResponse[]>({
      queryKey: ['roles', 'org', orgId],
      queryFn: () => roleApi.listOrgRoles({ orgId }),
    }),

  // Every role across the caller's visible organization subtree. Only for the
  // members-list role filter; the assignment picker must stay on `org` (see
  // its call site in MembersPage).
  visible: () =>
    queryOptions<RoleResponse[]>({
      queryKey: ['roles', 'visible'],
      queryFn: () => roleApi.listRoles(),
    }),
}

export const organizationQueries = {
  list: () =>
    queryOptions<OrganizationResponse[]>({
      queryKey: ['organizations'],
      queryFn: () => organizationApi.listOrganizations(),
    }),

  byId: (orgId: string) =>
    queryOptions<OrganizationDetailResponse>({
      queryKey: ['organizations', orgId],
      queryFn: () => organizationApi.getOrganization({ orgId }),
    }),
}

export const regionsQuery = () =>
  queryOptions({
    queryKey: ['regions'],
    queryFn: () => regionApi.listRegions(),
  })

export const evaluationQuery = () =>
  queryOptions<EvaluationResponse>({
    queryKey: ['evaluation'],
    queryFn: () => evaluationApi.getEvaluation(),
  })

export const pluginsQuery = () =>
  queryOptions({
    queryKey: ['plugins'],
    queryFn: () => pluginApi.listPlugins(),
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
