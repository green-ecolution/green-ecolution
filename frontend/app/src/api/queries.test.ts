import { describe, it, expect, vi, beforeEach } from 'vitest'
import {
  treeQueries,
  vehicleQueries,
  clusterQueries,
  wateringPlanQueries,
  sensorQueries,
} from './queries'
import type { Tree, Vehicle, TreeCluster } from '@/api/backendApi'
import type {
  ListResponsePlanInListResponse,
  ListResponseTreeResponse,
  ListResponseVehicleResponse,
  ListResponseTreeClusterInListResponse,
  PlanResponse,
  RouteResponse,
  TreeClusterInListResponse,
} from '@green-ecolution/backend-client'
import { SensorStatus } from '@green-ecolution/backend-client'

vi.mock('./backendApi', () => ({
  treeApi: {
    listTrees: vi.fn(),
    getTree: vi.fn(),
  },
  vehicleApi: {
    listVehicles: vi.fn(),
    getVehicle: vi.fn(),
  },
  clusterApi: {
    listClusters: vi.fn(),
    getCluster: vi.fn(),
  },
  planApi: {
    listPlans: vi.fn(),
    getPlan: vi.fn(),
  },
  planPreviewApi: {
    previewPlanRoute: vi.fn(),
  },
}))

import { treeApi, vehicleApi, clusterApi, planApi, planPreviewApi } from './backendApi'

describe('Query Functions', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Tree Queries', () => {
    describe('treeQueries.list', () => {
      it('returns correct query options for fetching all trees', () => {
        const options = treeQueries.list()

        expect(options.queryKey).toContain('trees')
        expect(options.queryFn).toBeDefined()
      })

      it('includes pagination params in query key', () => {
        const options = treeQueries.list({ page: 2 })

        expect(options.queryKey).toEqual(['trees', 'list', { page: 2 }])
      })

      it('calls treeApi.listTrees when queryFn is executed', async () => {
        const mockResponse = { data: [] } as unknown as ListResponseTreeResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.listTrees).mockResolvedValueOnce(mockResponse)

        const options = treeQueries.list()
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.listTrees).toHaveBeenCalledWith(undefined)
        expect(result).toEqual(mockResponse)
      })

      it('passes params to treeApi.listTrees', async () => {
        const mockResponse = { data: [] } as unknown as ListResponseTreeResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.listTrees).mockResolvedValueOnce(mockResponse)

        const params = { page: 2 }
        const options = treeQueries.list(params)
        await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.listTrees).toHaveBeenCalledWith(params)
      })

      it('includes q in the query key when provided', () => {
        const options = treeQueries.list({ page: 1, perPage: 10, q: 'Eiche' })
        expect(options.queryKey).toEqual(['trees', 'list', { page: 1, perPage: 10, q: 'Eiche' }])
      })

      it('includes filter params in the query key', () => {
        const options = treeQueries.list({ page: 1, perPage: 10, wateringStatus: ['good'] })
        expect(options.queryKey).toEqual([
          'trees',
          'list',
          { page: 1, perPage: 10, wateringStatus: ['good'] },
        ])
      })

      it('passes q to treeApi.listTrees', async () => {
        const mockResponse = { data: [] } as unknown as ListResponseTreeResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.listTrees).mockResolvedValueOnce(mockResponse)

        const options = treeQueries.list({ page: 1, perPage: 10, q: 'Eiche' })
        await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.listTrees).toHaveBeenCalledWith({ page: 1, perPage: 10, q: 'Eiche' })
      })
    })

    describe('treeQueries.detail', () => {
      it('returns correct query options for fetching single tree', () => {
        const options = treeQueries.detail('123')

        expect(options.queryKey).toEqual(['tree', '123'])
        expect(options.queryFn).toBeDefined()
      })

      it('calls treeApi.getTree with correct id', async () => {
        const mockTree = { id: 'tree-uuid-1', latitude: 54.0, longitude: 9.0 } as unknown as Tree
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.getTree).mockResolvedValueOnce(mockTree)

        const options = treeQueries.detail('tree-uuid-1')
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.getTree).toHaveBeenCalledWith({ treeId: 'tree-uuid-1' })
        expect(result).toEqual(mockTree)
      })

      it('passes string id directly to API', async () => {
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.getTree).mockResolvedValueOnce({} as Tree)

        const options = treeQueries.detail('tree-uuid-2')
        await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.getTree).toHaveBeenCalledWith({ treeId: 'tree-uuid-2' })
      })
    })
  })

  describe('Vehicle Queries', () => {
    describe('vehicleQueries.list', () => {
      it('returns correct query options for fetching all vehicles', () => {
        const options = vehicleQueries.list()

        expect(options.queryKey).toEqual(['vehicles', 'list', {}])
        expect(options.queryFn).toBeDefined()
      })

      it('does not collide with vehicleQueries.detail keys', () => {
        const listOptions = vehicleQueries.list({ page: 1 })
        const detailOptions = vehicleQueries.detail('1')

        expect(listOptions.queryKey[0]).not.toBe(detailOptions.queryKey[0])
      })

      it('includes pagination params in query key', () => {
        const options = vehicleQueries.list({ page: 2 })

        expect(options.queryKey).toEqual(['vehicles', 'list', { page: 2 }])
      })

      it('calls vehicleApi.listVehicles when queryFn is executed', async () => {
        const mockResponse = { data: [] } as unknown as ListResponseVehicleResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(vehicleApi.listVehicles).mockResolvedValueOnce(mockResponse)

        const options = vehicleQueries.list()
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(vehicleApi.listVehicles).toHaveBeenCalledWith(undefined)
        expect(result).toEqual(mockResponse)
      })

      it('passes params to vehicleApi.listVehicles', async () => {
        const mockResponse = { data: [] } as unknown as ListResponseVehicleResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(vehicleApi.listVehicles).mockResolvedValueOnce(mockResponse)

        const params = { page: 1 }
        const options = vehicleQueries.list(params)
        await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(vehicleApi.listVehicles).toHaveBeenCalledWith(params)
      })
    })

    describe('vehicleQueries.detail', () => {
      it('returns correct query options for fetching single vehicle', () => {
        const options = vehicleQueries.detail('42')

        expect(options.queryKey).toEqual(['vehicle', '42'])
        expect(options.queryFn).toBeDefined()
      })

      it('calls vehicleApi.getVehicle with correct id', async () => {
        const mockVehicle = {
          id: 'vehicle-uuid-1',
          numberPlate: 'HH-AB-1234',
        } as unknown as Vehicle
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(vehicleApi.getVehicle).mockResolvedValueOnce(mockVehicle)

        const options = vehicleQueries.detail('vehicle-uuid-1')
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(vehicleApi.getVehicle).toHaveBeenCalledWith({ vehicleId: 'vehicle-uuid-1' })
        expect(result).toEqual(mockVehicle)
      })
    })
  })

  describe('Sensor Queries', () => {
    describe('sensorQueries.list', () => {
      it('returns correct query options for fetching all sensors', () => {
        const options = sensorQueries.list()

        expect(options.queryKey).toEqual(['sensors', {}])
        expect(options.queryFn).toBeDefined()
      })

      it('includes the full params object in the query key, not just the page', () => {
        const options = sensorQueries.list({ page: 2 })

        expect(options.queryKey).toEqual(['sensors', { page: 2 }])
      })

      // Guards the cache-key bug directly: a key that dropped everything but
      // the page would pass the assertion above yet still fail this one,
      // since both calls share `page: 1`.
      it('produces different keys for different filter combinations on the same page', () => {
        const online = sensorQueries.list({ page: 1, status: [SensorStatus.Online] })
        const offline = sensorQueries.list({ page: 1, status: [SensorStatus.Offline] })

        expect(online.queryKey).not.toEqual(offline.queryKey)
      })
    })
  })

  describe('TreeCluster Queries', () => {
    describe('clusterQueries.list', () => {
      it('returns correct query options for fetching all clusters', () => {
        const options = clusterQueries.list()

        expect(options.queryKey).toContain('treeclusters')
        expect(options.queryFn).toBeDefined()
      })

      it('calls clusterApi.listClusters when queryFn is executed', async () => {
        const mockResponse = { data: [] } as unknown as ListResponseTreeClusterInListResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(clusterApi.listClusters).mockResolvedValueOnce(mockResponse)

        const options = clusterQueries.list()
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(clusterApi.listClusters).toHaveBeenCalledWith(undefined)
        expect(result).toEqual(mockResponse)
      })
    })

    describe('clusterQueries.detail', () => {
      it('returns correct query options for fetching single cluster', () => {
        const options = clusterQueries.detail('99')

        expect(options.queryKey).toEqual(['treecluster', '99'])
        expect(options.queryFn).toBeDefined()
      })

      it('calls clusterApi.getCluster with correct id', async () => {
        const mockCluster = {
          id: 'cluster-uuid-1',
          name: 'Test Cluster',
        } as unknown as TreeCluster
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(clusterApi.getCluster).mockResolvedValueOnce(mockCluster)

        const options = clusterQueries.detail('cluster-uuid-1')
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(clusterApi.getCluster).toHaveBeenCalledWith({ clusterId: 'cluster-uuid-1' })
        expect(result).toEqual(mockCluster)
      })
    })
  })

  describe('WateringPlan Queries', () => {
    const clusterPage = (...ids: string[]) =>
      ({
        data: ids.map((id) => ({ id, name: `Cluster ${id}` })),
      }) as unknown as ListResponseTreeClusterInListResponse

    describe('wateringPlanQueries.list', () => {
      it('returns correct query options for fetching all watering plans', () => {
        const options = wateringPlanQueries.list()

        expect(options.queryKey).toContain('watering-plans')
        expect(options.queryFn).toBeDefined()
      })

      it('includes default page in query key', () => {
        const options = wateringPlanQueries.list()

        expect(options.queryKey).toContain('1')
      })

      it('includes custom page in query key', () => {
        const options = wateringPlanQueries.list({ page: 3 })

        expect(options.queryKey).toContain(3)
      })

      it('lists irrigation plans and resolves their clusters in one call', async () => {
        const mockResponse = {
          data: [
            { id: 'p1', module: 'irrigation', stops: ['c1', 'c2'], totalDemand: 10 },
            { id: 'p2', module: 'irrigation', stops: ['c2'], totalDemand: 5 },
          ],
          pagination: { currentPage: 1 },
        } as unknown as ListResponsePlanInListResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(planApi.listPlans).mockResolvedValueOnce(mockResponse)
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(clusterApi.listClusters).mockResolvedValueOnce(clusterPage('c1', 'c2'))

        const options = wateringPlanQueries.list()
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(planApi.listPlans).toHaveBeenCalledWith({ module: 'irrigation' })
        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(clusterApi.listClusters).toHaveBeenCalledTimes(1)
        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(clusterApi.listClusters).toHaveBeenCalledWith({ id: ['c1', 'c2'], perPage: 100 })
        expect(result.data.map((plan) => plan.treeclusters.map((c) => c.id))).toEqual([
          ['c1', 'c2'],
          ['c2'],
        ])
        expect(result.data[0].totalWaterRequired).toBe(10)
      })
    })

    describe('wateringPlanQueries.detail', () => {
      it('returns correct query options for fetching single watering plan', () => {
        const options = wateringPlanQueries.detail('55')

        expect(options.queryKey).toEqual(['watering-plan', '55'])
        expect(options.queryFn).toBeDefined()
      })

      it('loads the clusters of the stops and keeps the stop order', async () => {
        const mockPlan = {
          id: 'plan-uuid-1',
          module: 'irrigation',
          date: '2025-01-15',
          stops: ['c2', 'c1'],
          outcomes: [],
          transporterCapacityUsed: true,
          totalDemand: 0,
        } as unknown as PlanResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(planApi.getPlan).mockResolvedValueOnce(mockPlan)
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(clusterApi.listClusters).mockResolvedValueOnce(clusterPage('c1', 'c2'))

        const options = wateringPlanQueries.detail('plan-uuid-1')
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(planApi.getPlan).toHaveBeenCalledWith({ planId: 'plan-uuid-1' })
        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(clusterApi.listClusters).toHaveBeenCalledWith({ id: ['c2', 'c1'], perPage: 100 })
        expect(result.id).toBe('plan-uuid-1')
        expect(result.treeclusters.map((c: TreeClusterInListResponse) => c.id)).toEqual([
          'c2',
          'c1',
        ])
      })
    })

    describe('wateringPlanQueries.routePreview', () => {
      it('returns the route in irrigation terms on success', async () => {
        const mockRoute = {
          stationVisits: [{ name: 'Hof', lat: 1, lon: 2 }],
          totalDemand: 40,
        } as unknown as RouteResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(planPreviewApi.previewPlanRoute).mockResolvedValueOnce(mockRoute)

        const options = wateringPlanQueries.routePreview({
          clusterIds: ['cluster-1'],
          transporterId: 'vehicle-1',
        })
        const result = await options.queryFn!({} as never)

        expect(result).toEqual({
          refillPoints: [{ name: 'Hof', lat: 1, lon: 2 }],
          totalWaterRequired: 40,
        })
      })

      it('returns null on any error instead of propagating', async () => {
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(planPreviewApi.previewPlanRoute).mockRejectedValueOnce(new Error('422'))

        const options = wateringPlanQueries.routePreview({
          clusterIds: ['cluster-1'],
          transporterId: 'vehicle-1',
        })
        const result = await options.queryFn!({} as never)

        expect(result).toBeNull()
      })

      it('uses planPreviewApi, not planApi', async () => {
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(planPreviewApi.previewPlanRoute).mockResolvedValueOnce({} as RouteResponse)

        const options = wateringPlanQueries.routePreview({
          clusterIds: ['cluster-1'],
          transporterId: 'vehicle-1',
        })
        await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(planPreviewApi.previewPlanRoute).toHaveBeenCalled()
        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(planApi.listPlans).not.toHaveBeenCalled()
      })

      it('routes with the trailer and without an unused transporter tank', async () => {
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(planPreviewApi.previewPlanRoute).mockResolvedValueOnce({} as RouteResponse)

        const options = wateringPlanQueries.routePreview({
          clusterIds: ['cluster-1'],
          transporterId: 'vehicle-1',
          trailerId: 'trailer-1',
          transporterTankUsed: false,
        })
        await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(planPreviewApi.previewPlanRoute).toHaveBeenCalledWith({
          planRouteRequest: expect.objectContaining({
            module: 'irrigation',
            stops: ['cluster-1'],
            trailerId: 'trailer-1',
            transporterCapacityUsed: false,
          }) as unknown,
        })
      })
    })
  })
})
