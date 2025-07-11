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

const backendFetch: FetchAPI = async (...args) => {
  const [resource, config] = args
  let response = await fetch(resource, config)
  if (response.status === 401) {
    const params: RequestInit = {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        refresh_token: useStore.getState().auth.token?.refreshToken ?? '',
      }),
    }
    const res = await fetch(`${basePath}/v1/user/token/refresh`, params)
    if (res.status !== 200) {
      useStore.getState().auth.clear()
      throw redirect({
        to: '/login',
        search: { redirect: window.location.pathname + window.location.search },
      })
    }
    const data = ClientTokenFromJSON(await res.json())
    useStore.getState().auth.setToken(data)
    useStore.getState().user.setFromJwt(data.accessToken)

    response = await fetch(resource, {
      ...config,
      headers: {
        ...config?.headers,
        Authorization: `Bearer ${data.accessToken}`,
      },
    })
  }
  return response
}

const configParams: ConfigurationParameters = {
  basePath,
  headers,
  fetchApi: backendFetch,
  accessToken() {
    const token = useStore.getState().auth.token?.accessToken
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
