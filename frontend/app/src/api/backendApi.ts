import {
  CommentsApi,
  Configuration,
  ConfigurationParameters,
  FetchAPI,
  HTTPHeaders,
  InfoApi,
  ModulesApi,
  OrganizationsApi,
  PlansApi,
  PluginsApi,
  RegionsApi,
  RolesApi,
  RoutingApi,
  SensorsApi,
  SettingsApi,
  UsersApi,
  VehiclesApi,
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

  // Only 401 means the session is gone. 422 is a domain rejection the backend
  // sends with a message (invalid input, organization mismatch, …) — sending
  // the user through a fresh sign-in would swallow it.
  if (response.status === 401 && !signinRedirectInFlight) {
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

// Route-preview calls fail routinely while a plan is still being assembled and
// must never bounce the user to the login page.
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

export const configuration = new Configuration(configParams)

const silentConfig = new Configuration({ ...configParams, fetchApi: silentFetch })

export const infoApi = new InfoApi(configuration)
export const userApi = new UsersApi(configuration)
export const regionApi = new RegionsApi(configuration)
export const sensorApi = new SensorsApi(configuration)
export const vehicleApi = new VehiclesApi(configuration)
export const pluginApi = new PluginsApi(configuration)
export const planApi = new PlansApi(configuration)
export const planPreviewApi = new PlansApi(silentConfig)
export const routingApi = new RoutingApi(configuration)
export const roleApi = new RolesApi(configuration)
export const organizationApi = new OrganizationsApi(configuration)
export const commentApi = new CommentsApi(configuration)
export const settingsApi = new SettingsApi(configuration)
export const moduleApi = new ModulesApi(configuration)

export * from '@green-ecolution/backend-client'

// Domain type aliases — clean names re-exported from the generated client
export type { SensorResponse as Sensor } from '@green-ecolution/backend-client'
export type { SensorDataResponse as SensorData } from '@green-ecolution/backend-client'
export type { VehicleResponse as Vehicle } from '@green-ecolution/backend-client'
export type { VehicleCreateRequest as VehicleCreate } from '@green-ecolution/backend-client'
export type { VehicleUpdateRequest as VehicleUpdate } from '@green-ecolution/backend-client'
export type {
  EvaluationValue,
  WateringPlan,
  WateringPlanCreate,
  WateringPlanInList,
  WateringPlanList,
  WateringPlanUpdate,
  WateringRoute,
} from './irrigationPlans'
export { PlanStatus as WateringPlanStatus } from '@green-ecolution/backend-client'
export type { UserResponse as User } from '@green-ecolution/backend-client'
export type { RegionResponse as Region } from '@green-ecolution/backend-client'
export type { AppInfoResponse as AppInfo } from '@green-ecolution/backend-client'
export type { MapInfoResponse as MapInfo } from '@green-ecolution/backend-client'
export type { ServerInfoResponse as ServerInfo } from '@green-ecolution/backend-client'
export type { ServicesInfoResponse as ServicesInfo } from '@green-ecolution/backend-client'
export type { DataStatisticsResponse as DataStatistics } from '@green-ecolution/backend-client'
export type { PaginationResponse as Pagination } from '@green-ecolution/backend-client'
export type { RoleResponse as Role } from '@green-ecolution/backend-client'
export type { RoleCreateRequest as RoleCreate } from '@green-ecolution/backend-client'
export type { RoleUpdateRequest as RoleUpdate } from '@green-ecolution/backend-client'
export type { OrganizationResponse as Organization } from '@green-ecolution/backend-client'

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
