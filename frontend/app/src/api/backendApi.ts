import useStore from '@/store/store'
import {
  ClientTokenResponseFromJSON,
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
import { redirect } from '@tanstack/react-router'

export const basePath = import.meta.env.VITE_BACKEND_BASEURL ?? '/api-local'

const headers: HTTPHeaders = {
  'Content-Type': 'application/json',
  Accept: 'application/json',
}

let refreshPromise: Promise<void> | null = null
let lastRefreshTime = 0
const MIN_REFRESH_INTERVAL = 5000

async function performTokenRefresh(): Promise<void> {
  const refreshToken =
    useStore.getState().token?.refreshToken ?? localStorage.getItem('refreshToken')
  if (!refreshToken) {
    useStore.getState().clearAuth()
    throw redirect({
      to: '/login',
      search: { redirect: window.location.pathname + window.location.search },
    })
  }

  const res = await fetch(`${basePath}/v1/users/token/refresh`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ refresh_token: refreshToken }),
  })

  if (res.status !== 200) {
    useStore.getState().clearAuth()
    throw redirect({
      to: '/login',
      search: { redirect: window.location.pathname + window.location.search },
    })
  }

  const data = ClientTokenResponseFromJSON(await res.json())
  useStore.getState().setToken(data)
  useStore.getState().setUserFromJwt(data.accessToken)
  lastRefreshTime = Date.now()
}

async function ensureFreshToken(): Promise<void> {
  const store = useStore.getState()
  if (!store.isAuthenticated) return

  if (store.token && !store.isTokenExpiringSoon()) return
  if (Date.now() - lastRefreshTime < MIN_REFRESH_INTERVAL && store.token) return

  if (refreshPromise) {
    await refreshPromise
    return
  }

  refreshPromise = performTokenRefresh().finally(() => {
    refreshPromise = null
  })
  await refreshPromise
}

const backendFetch: FetchAPI = async (...args) => {
  const [resource, config] = args

  await ensureFreshToken()

  const currentToken = useStore.getState().token?.accessToken
  const updatedConfig = currentToken
    ? { ...config, headers: { ...config?.headers, Authorization: `Bearer ${currentToken}` } }
    : config

  const response = await fetch(resource, updatedConfig)

  const resourceUrl =
    typeof resource === 'string' ? resource : resource instanceof URL ? resource.href : ''
  if (response.status === 401 && !resourceUrl.includes('/token/refresh')) {
    if (!refreshPromise) {
      useStore.getState().clearAuth()
      throw redirect({
        to: '/login',
        search: { redirect: window.location.pathname + window.location.search },
      })
    }
  }

  return response
}

const configParams: ConfigurationParameters = {
  basePath,
  headers,
  fetchApi: backendFetch,
  accessToken() {
    const token = useStore.getState().token?.accessToken
    return token ? `Bearer ${token}` : ''
  },
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
