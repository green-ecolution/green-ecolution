import { queryOptions, keepPreviousData } from '@tanstack/react-query'
import {
  AppInfoResponse,
  clusterApi,
  ClusterMarkerListResponse,
  DataStatisticsResponse,
  EvaluationResponse,
  infoApi,
  ListClustersRequest,
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
  regionApi,
  SensorDataResponse,
  SensorModelResponse,
  SensorResponse,
  sensorApi,
  ServerInfoResponse,
  ServicesInfoResponse,
  TreeClusterResponse,
  TreeMarkerListResponse,
  TreeResponse,
  treeApi,
  userApi,
  vehicleApi,
  VehicleResponse,
  WateringPlanResponse,
  WateringStatus,
  wateringPlanApi,
  evaluationApi,
} from './backendApi'

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

/**
 * Check if a string is a valid UUID. Backend identifiers are UUID v7;
 * the regex accepts any UUID version since the frontend never inspects bits.
 */
const isValidUuid = (id: string | undefined): boolean => typeof id === 'string' && UUID_RE.test(id)

export const treeClusterQuery = (params?: ListClustersRequest) =>
  queryOptions<ListResponseTreeClusterInListResponse>({
    queryKey: ['treeclusters', params?.page, params?.perPage].filter(
      (e) => e != undefined || e != null,
    ),
    queryFn: () => clusterApi.listClusters(params),
  })

export const treeClusterIdQuery = (id: string) =>
  queryOptions<TreeClusterResponse>({
    queryKey: ['treecluster', id],
    queryFn: () => clusterApi.getCluster({ clusterId: id }),
    enabled: isValidUuid(id),
  })

export const sensorQuery = (params?: ListSensorsRequest) =>
  queryOptions<ListResponseSensorResponse>({
    queryKey: ['sensors', params?.page ?? '1'],
    queryFn: () => sensorApi.listSensors(params),
  })

export const sensorDataQuery = (id: string) =>
  queryOptions<SensorDataResponse[]>({
    queryKey: ['sensor data', id],
    queryFn: () =>
      sensorApi.listSensorData({
        sensorId: id,
      }),
  })

export const sensorIdQuery = (id: string) =>
  queryOptions<SensorResponse>({
    queryKey: ['sensor', id],
    queryFn: () =>
      sensorApi.getSensor({
        sensorId: id,
      }),
  })

export const sensorModelIdQuery = (id: string) =>
  queryOptions<SensorModelResponse>({
    queryKey: ['sensor-model', id],
    queryFn: () => sensorApi.getSensorModel({ id }),
    enabled: isValidUuid(id),
  })

export const treeQuery = (params?: ListTreesRequest) =>
  queryOptions<ListResponseTreeResponse>({
    queryKey: ['trees', params?.page, params?.perPage, ...(params?.q ? [params.q] : [])].filter(
      (e) => e !== undefined && e !== null,
    ),
    queryFn: () => treeApi.listTrees(params),
  })

export const treeIdQuery = (id: string) =>
  queryOptions<TreeResponse>({
    queryKey: ['tree', id],
    queryFn: () => treeApi.getTree({ treeId: id }),
    enabled: isValidUuid(id),
  })

// TODO: The Rust backend changed this endpoint to /trees/{tree_id}/sensors/{sensor_id}
// which requires both IDs. The Go backend had /tree/sensor/{sensor_id}.
// This query needs a dedicated "get tree by sensor ID" endpoint in the Rust backend.
// export const treeSensorIdQuery = (id: string) =>
//   queryOptions<TreeResponse>({
//     queryKey: ['tree-sensor', id],
//     queryFn: () => treeApi.getTreeSensor({ treeId: ???, sensorId: id }),
//   })

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

export const vehicleQuery = (params?: ListVehiclesRequest) => {
  return queryOptions<ListResponseVehicleResponse>({
    queryKey: ['vehicle', params?.page].filter((e) => e != undefined || e != null),
    queryFn: () => vehicleApi.listVehicles(params),
  })
}

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
