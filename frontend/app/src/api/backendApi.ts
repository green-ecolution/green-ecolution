import {
  Configuration,
  ConfigurationParameters,
  EvaluationApi,
  FetchAPI,
  HTTPHeaders,
  InfoApi,
  PluginsApi,
  RegionsApi,
  SensorsApi,
  TreesApi,
  TreeClustersApi,
  UsersApi,
  VehiclesApi,
  WateringPlansApi,
} from '@green-ecolution/backend-client'

export const basePath = import.meta.env.VITE_BACKEND_BASEURL ?? '/api-local'

const headers: HTTPHeaders = {
  'Content-Type': 'application/json',
  Accept: 'application/json',
}

// TODO: Re-enable token refresh + auth guard once Rust backend implements authentication
const backendFetch: FetchAPI = async (...args) => {
  const [resource, config] = args
  return fetch(resource, config)
}

const configParams: ConfigurationParameters = {
  basePath,
  headers,
  fetchApi: backendFetch,
  // TODO: Re-enable once auth is implemented in Rust backend
  // accessToken() {
  //   const token = useStore.getState().token?.accessToken
  //   return token ? `Bearer ${token}` : ''
  // },
}

const config = new Configuration(configParams)

export const treeApi = new TreesApi(config)
export const clusterApi = new TreeClustersApi(config)
export const infoApi = new InfoApi(config)
export const evaluationApi = new EvaluationApi(config)
export const userApi = new UsersApi(config)
export const regionApi = new RegionsApi(config)
export const sensorApi = new SensorsApi(config)
export const vehicleApi = new VehiclesApi(config)
export const pluginApi = new PluginsApi(config)
export const wateringPlanApi = new WateringPlansApi(config)

export * from '@green-ecolution/backend-client'

// Domain type aliases — clean names re-exported from the generated client
export type { TreeResponse as Tree } from '@green-ecolution/backend-client'
export type { TreeCreateRequest as TreeCreate } from '@green-ecolution/backend-client'
export type { TreeUpdateRequest as TreeUpdate } from '@green-ecolution/backend-client'
export type { TreeClusterResponse as TreeCluster } from '@green-ecolution/backend-client'
export type { TreeClusterInListResponse as TreeClusterInList } from '@green-ecolution/backend-client'
export type { TreeClusterCreateRequest as TreeClusterCreate } from '@green-ecolution/backend-client'
export type { TreeClusterUpdateRequest as TreeClusterUpdate } from '@green-ecolution/backend-client'
export type { SensorResponse as Sensor } from '@green-ecolution/backend-client'
export type { SensorDataResponse as SensorData } from '@green-ecolution/backend-client'
export type { VehicleResponse as Vehicle } from '@green-ecolution/backend-client'
export type { VehicleCreateRequest as VehicleCreate } from '@green-ecolution/backend-client'
export type { VehicleUpdateRequest as VehicleUpdate } from '@green-ecolution/backend-client'
export type { WateringPlanResponse as WateringPlan } from '@green-ecolution/backend-client'
export type { WateringPlanInListResponse as WateringPlanInList } from '@green-ecolution/backend-client'
export type { WateringPlanCreateRequest as WateringPlanCreate } from '@green-ecolution/backend-client'
export type { WateringPlanUpdateRequest as WateringPlanUpdate } from '@green-ecolution/backend-client'
export type { UserResponse as User } from '@green-ecolution/backend-client'
export type { RegionResponse as Region } from '@green-ecolution/backend-client'
export type { EvaluationResponse as Evaluation } from '@green-ecolution/backend-client'
export type { AppInfoResponse as AppInfo } from '@green-ecolution/backend-client'
export type { MapInfoResponse as MapInfo } from '@green-ecolution/backend-client'
export type { ServerInfoResponse as ServerInfo } from '@green-ecolution/backend-client'
export type { ServicesInfoResponse as ServicesInfo } from '@green-ecolution/backend-client'
export type { DataStatisticsResponse as DataStatistics } from '@green-ecolution/backend-client'
export type { ClientTokenResponse as ClientToken } from '@green-ecolution/backend-client'
export type { NearestTreeListResponse as NearestTreeList } from '@green-ecolution/backend-client'
export type { TreeWithDistanceResponse as TreeWithDistance } from '@green-ecolution/backend-client'
export type { EvaluationValueResponse as EvaluationValue } from '@green-ecolution/backend-client'
export type { PaginationResponse as Pagination } from '@green-ecolution/backend-client'
