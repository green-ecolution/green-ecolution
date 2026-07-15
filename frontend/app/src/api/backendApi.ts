import {
  Configuration,
  ConfigurationParameters,
  EvaluationApi,
  FetchAPI,
  HTTPHeaders,
  InfoApi,
  PluginsApi,
  RegionsApi,
  RoutingApi,
  SensorsApi,
  TreesApi,
  TreeClustersApi,
  UsersApi,
  VehiclesApi,
  WateringPlansApi,
} from '@green-ecolution/backend-client'
import { getAuthSession } from '@/lib/auth/session'

export const basePath = import.meta.env.VITE_BACKEND_BASEURL ?? '/api-local'

const headers: HTTPHeaders = {
  'Content-Type': 'application/json',
  Accept: 'application/json',
}

const withAuthHeader = async (
  config: RequestInit | undefined,
): Promise<RequestInit | undefined> => {
  const token = await getAuthSession().getAccessToken()
  return token
    ? { ...config, headers: { ...config?.headers, Authorization: `Bearer ${token}` } }
    : config
}

let signinRedirectInFlight = false

const backendFetch: FetchAPI = async (resource, config) => {
  const response = await fetch(resource, await withAuthHeader(config))

  // axum's Json extractor rejects malformed bodies with 422 before our handlers run,
  // so treat it like 401 and force a fresh sign-in.
  if ((response.status === 401 || response.status === 422) && !signinRedirectInFlight) {
    signinRedirectInFlight = true
    try {
      await getAuthSession().signinRedirect({
        returnTo: window.location.pathname + window.location.search,
      })
    } catch (err) {
      signinRedirectInFlight = false
      throw err
    }
  }
  return response
}

// preview errors must stay silent — a 422 here is a routing problem, not an auth problem.
const silentFetch: FetchAPI = async (resource, config) =>
  fetch(resource, await withAuthHeader(config))

const configParams: ConfigurationParameters = {
  basePath,
  headers,
  fetchApi: backendFetch,
  async accessToken() {
    const token = await getAuthSession().getAccessToken()
    return token ? `Bearer ${token}` : ''
  },
}

const config = new Configuration(configParams)

const silentConfig = new Configuration({ ...configParams, fetchApi: silentFetch })

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
export const wateringPlanPreviewApi = new WateringPlansApi(silentConfig)
export const routingApi = new RoutingApi(config)

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

export interface SensorPayloadWatermark {
  depth: number
  centibar: number
  resistance: number
}

/**
 * Shape of `SensorData.data` — untyped in the OpenAPI spec, so consumers cast to this.
 * All fields may be absent depending on the sensor model (watermark vs. volumetric readings).
 */
export interface SensorPayload {
  humidity?: number
  temperature?: number
  watermarks?: SensorPayloadWatermark[]
}
