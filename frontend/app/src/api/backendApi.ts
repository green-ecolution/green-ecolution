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

// Module-level refresh promise to prevent race conditions
let refreshPromise: Promise<void> | null = null

async function performTokenRefresh(): Promise<void> {
  const refreshToken = useStore.getState().token?.refreshToken
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
}

async function ensureFreshToken(): Promise<void> {
  const store = useStore.getState()
  if (!store.isAuthenticated || !store.token) return
  if (!store.isTokenExpiringSoon()) return

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

  // Proactive refresh before request
  await ensureFreshToken()

  let response = await fetch(resource, config)

  // Reactive refresh as fallback (token revoked server-side)
  if (response.status === 401) {
    if (refreshPromise) {
      await refreshPromise
    } else {
      refreshPromise = performTokenRefresh().finally(() => {
        refreshPromise = null
      })
      await refreshPromise
    }

    const newToken = useStore.getState().token?.accessToken
    response = await fetch(resource, {
      ...config,
      headers: { ...config?.headers, Authorization: `Bearer ${newToken}` },
    })
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
