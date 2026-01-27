import { queryOptions } from '@tanstack/react-query'
import {
  AppInfo,
  clusterApi,
  infoApi,
  regionApi,
  sensorApi,
  SensorList,
  Tree,
  treeApi,
  TreeCluster,
  TreeClusterList,
  TreeList,
  vehicleApi,
  VehicleList,
  Vehicle,
  treeSensorApi,
  Sensor,
  WateringPlanList,
  wateringPlanApi,
  WateringPlan,
  UserList,
  userApi,
  GeoJson,
  SensorDataList,
  Evaluation,
  evaluationApi,
  GetAllTreeClustersRequest,
  GetAllSensorsRequest,
  GetAllTreesRequest,
  GetAllVehiclesRequest,
  GetAllWateringPlansRequest,
  GetAllUsersRequest,
} from './backendApi'

/**
 * Check if a string ID is a valid positive integer.
 */
const isValidNumericId = (id: string | undefined): boolean => {
  if (id === undefined || id === null || id === '') return false
  const num = Number(id)
  return !isNaN(num) && Number.isInteger(num) && num > 0
}

/**
 * Parse a string ID to a positive integer.
 * Throws an error for invalid IDs.
 */
const parseNumericId = (id: string): number => {
  const num = Number(id)
  if (isNaN(num) || !Number.isInteger(num) || num <= 0) {
    throw new Error(`Invalid ID: ${id}`)
  }
  return num
}

export const treeClusterQuery = (params?: GetAllTreeClustersRequest) =>
  queryOptions<TreeClusterList>({
    queryKey: ['treeclusters', params?.page, params?.regions, params?.wateringStatuses].filter(
      (e) => e != undefined || e != null,
    ),
    queryFn: () => clusterApi.getAllTreeClusters(params),
  })

export const treeClusterIdQuery = (id: string) =>
  queryOptions<TreeCluster>({
    queryKey: ['treecluster', id],
    queryFn: () => clusterApi.getTreeClusterById({ clusterId: parseNumericId(id) }),
    enabled: isValidNumericId(id),
  })

export const sensorQuery = (params?: GetAllSensorsRequest) =>
  queryOptions<SensorList>({
    queryKey: ['sensors', params?.page ?? '1'],
    queryFn: () => sensorApi.getAllSensors(params),
  })

export const sensorDataQuery = (id: string) =>
  queryOptions<SensorDataList>({
    queryKey: ['sensor data', id],
    queryFn: () =>
      sensorApi.getAllSensorDataById({
        sensorId: id,
      }),
  })

export const sensorIdQuery = (id: string) =>
  queryOptions<Sensor>({
    queryKey: ['sensor', id],
    queryFn: () =>
      sensorApi.getSensorById({
        sensorId: id,
      }),
  })

export const treeQuery = (params?: GetAllTreesRequest) =>
  queryOptions<TreeList>({
    queryKey: ['trees', params?.page, params?.wateringStatuses, params?.plantingYears].filter(
      (e) => e != undefined || e != null,
    ),
    queryFn: () => treeApi.getAllTrees(params),
  })

export const treeIdQuery = (id: string) =>
  queryOptions<Tree>({
    queryKey: ['tree', id],
    queryFn: () => treeApi.getTrees({ treeId: parseNumericId(id) }),
    enabled: isValidNumericId(id),
  })

export const treeSensorIdQuery = (id: string) =>
  queryOptions<Tree>({
    queryKey: ['tree-sensor', id],
    queryFn: () =>
      treeSensorApi.getTreeBySensorId({
        sensorId: id,
      }),
  })

export const regionsQuery = () =>
  queryOptions({
    queryKey: ['regions'],
    queryFn: () => regionApi.v1RegionGet(),
  })

export const infoQuery = () =>
  queryOptions<AppInfo>({
    queryKey: ['info'],
    queryFn: () => infoApi.getAppInfo(),
  })

export const evaluationQuery = () =>
  queryOptions<Evaluation>({
    queryKey: ['evaluation'],
    queryFn: () => evaluationApi.getEvaluation(),
  })

export const vehicleQuery = (params?: GetAllVehiclesRequest) => {
  return queryOptions<VehicleList>({
    queryKey: ['vehicle', params?.type, params?.page].filter((e) => e != undefined || e != null),
    queryFn: () => vehicleApi.getAllVehicles(params),
  })
}

export const vehicleIdQuery = (id: string) =>
  queryOptions<Vehicle>({
    queryKey: ['vehicle', id],
    queryFn: () => vehicleApi.getVehicleById({ id: parseNumericId(id) }),
    enabled: isValidNumericId(id),
  })

export const wateringPlanQuery = (params?: GetAllWateringPlansRequest) =>
  queryOptions<WateringPlanList>({
    queryKey: ['watering-plans', params?.page ?? '1'],
    queryFn: () => wateringPlanApi.getAllWateringPlans(params),
  })

export const wateringPlanIdQuery = (id: string) =>
  queryOptions<WateringPlan>({
    queryKey: ['watering-plan', id],
    queryFn: () => wateringPlanApi.getWateringPlanById({ id: parseNumericId(id) }),
    enabled: isValidNumericId(id),
  })

export const userQuery = (params?: GetAllUsersRequest) => {
  return queryOptions<UserList>({
    queryKey: ['users', params],
    queryFn: () => userApi.getAllUsers(params),
  })
}

export const userRoleQuery = (role: string) =>
  queryOptions<UserList>({
    queryKey: ['user', role],
    queryFn: () =>
      userApi.getUsersByRole({
        role: role,
      }),
  })

export const routePreviewQuery = (
  transporterId: number,
  clusterIds: number[],
  trailerId?: number,
) =>
  queryOptions<GeoJson>({
    queryKey: ['route', 'preview', `transporter:${transporterId}`, ...clusterIds],
    queryFn: () =>
      wateringPlanApi.v1WateringPlanRoutePreviewPost({
        body: {
          transporterId: Number(transporterId),
          trailerId: Number(trailerId),
          clusterIds,
        },
      }),
  })
