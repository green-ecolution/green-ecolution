import { queryOptions, infiniteQueryOptions, type QueryKey } from '@tanstack/react-query'
import {
  AppInfoResponse,
  commentApi,
  DataStatisticsResponse,
  infoApi,
  ListResponseSensorDataResponse,
  ListResponseSensorResponse,
  ListResponseUserResponse,
  ListResponseVehicleResponse,
  ListSensorsRequest,
  ListUsersRequest,
  ListVehiclesRequest,
  ListResponsePlanInListResponse,
  MapInfoResponse,
  ReachableOrganizationResponse,
  moduleApi,
  ModuleListResponse,
  organizationApi,
  OrganizationDetailResponse,
  OrganizationModulesResponse,
  OrganizationResponse,
  OrganizationResourceCountsResponse,
  OrganizationSettingsResponse,
  pluginApi,
  PluginResponse,
  PluginNavEntryResponse,
  PluginViewResponse,
  regionApi,
  ResponseError,
  RoleResponse,
  roleApi,
  planApi,
  planPreviewApi,
  PlanResponse,
  PlanStatus,
  routingApi,
  SensorDataQualityResponse,
  SensorModelResponse,
  SensorResponse,
  sensorApi,
  ServerInfoResponse,
  ServicesInfoResponse,
  settingsApi,
  StartPointResponse,
  userApi,
  UserResponse,
  vehicleApi,
  VehicleResponse,
  RouteResponse,
} from './backendApi'
import type { TargetRefDto } from '@green-ecolution/backend-client'
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

/** Partial key matching every sensor list page; use for broad invalidation. */
const SENSORS_KEY = ['sensors'] as const

/**
 * Every key prefix an aggregate owns, grouped per aggregate. The keys below
 * grew inconsistent roots ('treecluster' vs 'treeclusters' vs 'clusters'), so
 * invalidating one prefix silently misses the rest — go through this table
 * instead of guessing. See lib/queryInvalidation.ts.
 */
export const queryRoots = {
  vehicle: [['vehicle'], ['vehicles']],
  sensor: [['sensor'], ['sensors'], ['sensor data'], ['sensor-model']],
  wateringPlan: [['watering-plan'], ['watering-plans'], ['watering-plan-route'], ['route-preview']],
  region: [['regions']],
  statistics: [['info']],
  user: [['users']],
  role: [['roles']],
  organization: [['organizations']],
  comment: [['comments']],
  plugin: [['plugins']],
  startPoint: [['routing-start-points']],
} as const satisfies Record<string, readonly QueryKey[]>

export type Aggregate = keyof typeof queryRoots

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
      queryKey: [...SENSORS_KEY, params ?? {}],
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

  models: () =>
    queryOptions<SensorModelResponse[]>({
      queryKey: ['sensor-model', 'list'],
      queryFn: () => sensorApi.listSensorModels(),
    }),

  dataQuality: (id: string) =>
    queryOptions<SensorDataQualityResponse>({
      queryKey: ['sensor', id, 'data-quality'],
      queryFn: () => sensorApi.getSensorDataQuality({ sensorId: id }),
      enabled: id !== '',
    }),
}

export const wateringPlanQueries = {
  detail: (id: string) =>
    queryOptions<PlanResponse>({
      queryKey: ['watering-plan', id],
      queryFn: () => planApi.getPlan({ planId: id }),
      enabled: isValidUuid(id),
    }),

  route: (id: string) =>
    queryOptions<RouteResponse | null>({
      queryKey: ['watering-plan-route', id],
      queryFn: async () => {
        try {
          return await planApi.getPlanRoute({ planId: id })
        } catch (error) {
          // 404: plan has no computed route; 503: routing feature disabled.
          if (error instanceof ResponseError && [404, 503].includes(error.response.status))
            return null
          throw error
        }
      },
      enabled: isValidUuid(id),
    }),

  boardColumn: (module: string, statuses: PlanStatus[]) =>
    queryOptions<ListResponsePlanInListResponse>({
      queryKey: ['watering-plans', 'board', module, statuses],
      queryFn: () => planApi.listPlans({ module, status: statuses, page: 1, perPage: 100 }),
    }),

  boardDone: (module: string) =>
    infiniteQueryOptions({
      queryKey: ['watering-plans', 'board', module, 'done'],
      queryFn: ({ pageParam }) =>
        planApi.listPlans({ module, status: DONE_STATUSES, page: pageParam, perPage: 20 }),
      initialPageParam: 1,
      getNextPageParam: (lastPage) =>
        lastPage.pagination?.nextPage ? lastPage.pagination.currentPage + 1 : undefined,
    }),

  routePreview: ({
    module,
    stops,
    transporterId,
    trailerId,
    transporterCapacityUsed = true,
    startPointName,
    organizationId,
  }: {
    module: string
    stops: string[]
    transporterId: string
    trailerId?: string
    transporterCapacityUsed?: boolean
    startPointName?: string | null
    organizationId?: string | null
  }) =>
    queryOptions<RouteResponse | null>({
      queryKey: [
        'route-preview',
        module,
        stops.slice().sort(),
        transporterId,
        trailerId ?? null,
        transporterCapacityUsed,
        startPointName ?? null,
        organizationId ?? null,
      ],
      queryFn: async () => {
        try {
          return await planPreviewApi.previewPlanRoute({
            planRouteRequest: {
              module,
              stops,
              transporterId,
              trailerId,
              transporterCapacityUsed,
              startPointName,
              organizationId,
            },
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

  // The viewport the caller's organization opens at. The instance-wide
  // `/info/map` stays unauthenticated and deliberately answers only the
  // instance default, so it is no use here.
  mapView: () =>
    queryOptions<MapInfoResponse>({
      queryKey: ['users', 'me', 'map-view'],
      queryFn: () => userApi.getMyMapView(),
      staleTime: Infinity,
    }),

  myOrganizations: () =>
    queryOptions<ReachableOrganizationResponse[]>({
      queryKey: ['users', 'me', 'organizations'],
      queryFn: () => userApi.listMyOrganizations(),
      staleTime: 5 * 60 * 1000,
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

  // Lives under 'organizations' so the invalidation on that prefix
  // (queryRoots.organization) refreshes it after an organization changes.
  resourceCounts: () =>
    queryOptions<OrganizationResourceCountsResponse[]>({
      queryKey: ['organizations', 'resource-counts'],
      queryFn: () => organizationApi.listOrganizationResourceCounts(),
    }),

  byId: (orgId: string) =>
    queryOptions<OrganizationDetailResponse>({
      queryKey: ['organizations', orgId],
      queryFn: () => organizationApi.getOrganization({ orgId }),
    }),
}

// Key lives under 'organizations' so the existing invalidation on that
// prefix (queryRoots.organization) reaches it too.
export const settingsQueries = {
  byOrganization: (orgId: string) =>
    queryOptions<OrganizationSettingsResponse>({
      queryKey: ['organizations', orgId, 'settings'],
      queryFn: () => settingsApi.getOrganizationSettings({ orgId }),
    }),
}

export type CommentSubject = { kind: 'plan'; id: string } | { kind: 'target'; target: TargetRefDto }

export const commentSubjectId = (subject: CommentSubject): string =>
  subject.kind === 'plan' ? subject.id : subject.target.id

export const commentQueryKey = (subject: CommentSubject): QueryKey =>
  subject.kind === 'plan'
    ? ['comments', 'plan', subject.id]
    : ['comments', 'target', subject.target.module, subject.target.kind, subject.target.id]

export const commentQueries = {
  list: (subject: CommentSubject) =>
    infiniteQueryOptions({
      queryKey: commentQueryKey(subject),
      queryFn: ({ pageParam }) =>
        subject.kind === 'plan'
          ? commentApi.listPlanComments({ planId: subject.id, page: pageParam, perPage: 20 })
          : commentApi.listTargetComments({
              module: subject.target.module,
              kind: subject.target.kind,
              targetId: subject.target.id,
              page: pageParam,
              perPage: 20,
            }),
      initialPageParam: 1,
      getNextPageParam: (lastPage) =>
        lastPage.pagination?.nextPage ? lastPage.pagination.currentPage + 1 : undefined,
      enabled: isValidUuid(commentSubjectId(subject)),
    }),
}

export const regionsQuery = () =>
  queryOptions({
    queryKey: ['regions'],
    queryFn: () => regionApi.listRegions(),
  })

export const pluginsQuery = () =>
  queryOptions({
    queryKey: ['plugins'],
    queryFn: () => pluginApi.listPlugins(),
  })

export const pluginQuery = (slug: string) =>
  queryOptions<PluginResponse>({
    queryKey: ['plugins', slug],
    queryFn: () => pluginApi.getPlugin({ pluginSlug: slug }),
  })

/** Navigation entries: the plugin views the signed-in user may open. */
export const pluginViewsQuery = () =>
  queryOptions<PluginNavEntryResponse[]>({
    queryKey: ['plugins', 'views'],
    queryFn: () => pluginApi.listPluginViews(),
    // A disabled plugin feature answers 503; the navigation just stays without plugins.
    retry: false,
  })

/**
 * The view surface: readable with the plugin's own required permissions,
 * unlike `pluginQuery`, which needs `plugin:read`.
 */
export const pluginViewQuery = (slug: string) =>
  queryOptions<PluginViewResponse>({
    queryKey: ['plugins', slug, 'view'],
    queryFn: () => pluginApi.getPluginView({ pluginSlug: slug }),
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

export const modulesQuery = () =>
  queryOptions<ModuleListResponse>({
    queryKey: ['modules'],
    queryFn: () => moduleApi.listModules(),
  })

export const organizationModulesQuery = (orgId: string) =>
  queryOptions<OrganizationModulesResponse>({
    queryKey: ['organizations', orgId, 'modules'],
    queryFn: () => moduleApi.getOrganizationModules({ orgId }),
  })
