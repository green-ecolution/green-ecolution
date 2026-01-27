import useStore from '@/store/store'
import {
  ClientTokenFromJSON,
  Configuration,
  ConfigurationParameters,
  EvaluationApi,
  FetchAPI,
  HTTPHeaders,
  InfoApi,
  PluginApi,
  RegionApi,
  SensorApi,
  TreeApi,
  TreeClusterApi,
  TreeSensorApi,
  UserApi,
  VehicleApi,
  WateringPlanApi,
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

  const res = await fetch(`${basePath}/v1/user/token/refresh`, {
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

  const data = ClientTokenFromJSON(await res.json())
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
    if (!token) {
      return ''
    }
    return `Bearer ${token}`
  },
}

const config = new Configuration(configParams)

export const treeApi = new TreeApi(config)
export const treeSensorApi = new TreeSensorApi(config)
export const clusterApi = new TreeClusterApi(config)
export const infoApi = new InfoApi(config)
export const evaluationApi = new EvaluationApi(config)
export const userApi = new UserApi(config)
export const regionApi = new RegionApi(config)
export const sensorApi = new SensorApi(config)
export const vehicleApi = new VehicleApi(config)
export const pluginApi = new PluginApi(config)
export const wateringPlanApi = new WateringPlanApi(config)

export * from '@green-ecolution/backend-client'
