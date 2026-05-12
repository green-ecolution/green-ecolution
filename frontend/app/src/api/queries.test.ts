import { describe, it, expect, vi, beforeEach } from 'vitest'
import {
  treeQuery,
  treeIdQuery,
  vehicleQuery,
  vehicleIdQuery,
  treeClusterQuery,
  treeClusterIdQuery,
  wateringPlanQuery,
  wateringPlanIdQuery,
} from './queries'
import type { Tree, Vehicle, TreeCluster, WateringPlan } from '@/api/backendApi'
import type {
  ListResponseTreeResponse,
  ListResponseVehicleResponse,
  ListResponseTreeClusterInListResponse,
  ListResponseWateringPlanInListResponse,
} from '@green-ecolution/backend-client'

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
  wateringPlanApi: {
    listWateringPlans: vi.fn(),
    getWateringPlan: vi.fn(),
  },
}))

import { treeApi, vehicleApi, clusterApi, wateringPlanApi } from './backendApi'

describe('Query Functions', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Tree Queries', () => {
    describe('treeQuery', () => {
      it('returns correct query options for fetching all trees', () => {
        const options = treeQuery()

        expect(options.queryKey).toContain('trees')
        expect(options.queryFn).toBeDefined()
      })

      it('includes pagination params in query key', () => {
        const options = treeQuery({ page: 2 })

        expect(options.queryKey).toContain('trees')
        expect(options.queryKey).toContain(2)
      })

      it('calls treeApi.listTrees when queryFn is executed', async () => {
        const mockResponse = { data: [] } as unknown as ListResponseTreeResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.listTrees).mockResolvedValueOnce(mockResponse)

        const options = treeQuery()
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
        const options = treeQuery(params)
        await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.listTrees).toHaveBeenCalledWith(params)
      })
    })

    describe('treeIdQuery', () => {
      it('returns correct query options for fetching single tree', () => {
        const options = treeIdQuery('123')

        expect(options.queryKey).toEqual(['tree', '123'])
        expect(options.queryFn).toBeDefined()
      })

      it('calls treeApi.getTree with correct id', async () => {
        const mockTree = { id: 123, latitude: 54.0, longitude: 9.0 } as Tree
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.getTree).mockResolvedValueOnce(mockTree)

        const options = treeIdQuery('123')
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.getTree).toHaveBeenCalledWith({ treeId: 123 })
        expect(result).toEqual(mockTree)
      })

      it('converts string id to number', async () => {
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.getTree).mockResolvedValueOnce({} as Tree)

        const options = treeIdQuery('456')
        await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.getTree).toHaveBeenCalledWith({ treeId: 456 })
      })
    })
  })

  describe('Vehicle Queries', () => {
    describe('vehicleQuery', () => {
      it('returns correct query options for fetching all vehicles', () => {
        const options = vehicleQuery()

        expect(options.queryKey).toContain('vehicle')
        expect(options.queryFn).toBeDefined()
      })

      it('calls vehicleApi.listVehicles when queryFn is executed', async () => {
        const mockResponse = { data: [] } as unknown as ListResponseVehicleResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(vehicleApi.listVehicles).mockResolvedValueOnce(mockResponse)

        const options = vehicleQuery()
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
        const options = vehicleQuery(params)
        await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(vehicleApi.listVehicles).toHaveBeenCalledWith(params)
      })
    })

    describe('vehicleIdQuery', () => {
      it('returns correct query options for fetching single vehicle', () => {
        const options = vehicleIdQuery('42')

        expect(options.queryKey).toEqual(['vehicle', '42'])
        expect(options.queryFn).toBeDefined()
      })

      it('calls vehicleApi.getVehicle with correct id', async () => {
        const mockVehicle = { id: 42, numberPlate: 'HH-AB-1234' } as Vehicle
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(vehicleApi.getVehicle).mockResolvedValueOnce(mockVehicle)

        const options = vehicleIdQuery('42')
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(vehicleApi.getVehicle).toHaveBeenCalledWith({ vehicleId: 42 })
        expect(result).toEqual(mockVehicle)
      })
    })
  })

  describe('TreeCluster Queries', () => {
    describe('treeClusterQuery', () => {
      it('returns correct query options for fetching all clusters', () => {
        const options = treeClusterQuery()

        expect(options.queryKey).toContain('treeclusters')
        expect(options.queryFn).toBeDefined()
      })

      it('calls clusterApi.listClusters when queryFn is executed', async () => {
        const mockResponse = { data: [] } as unknown as ListResponseTreeClusterInListResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(clusterApi.listClusters).mockResolvedValueOnce(mockResponse)

        const options = treeClusterQuery()
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(clusterApi.listClusters).toHaveBeenCalledWith(undefined)
        expect(result).toEqual(mockResponse)
      })
    })

    describe('treeClusterIdQuery', () => {
      it('returns correct query options for fetching single cluster', () => {
        const options = treeClusterIdQuery('99')

        expect(options.queryKey).toEqual(['treecluster', '99'])
        expect(options.queryFn).toBeDefined()
      })

      it('calls clusterApi.getCluster with correct id', async () => {
        const mockCluster = { id: 99, name: 'Test Cluster' } as TreeCluster
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(clusterApi.getCluster).mockResolvedValueOnce(mockCluster)

        const options = treeClusterIdQuery('99')
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(clusterApi.getCluster).toHaveBeenCalledWith({ clusterId: 99 })
        expect(result).toEqual(mockCluster)
      })
    })
  })

  describe('WateringPlan Queries', () => {
    describe('wateringPlanQuery', () => {
      it('returns correct query options for fetching all watering plans', () => {
        const options = wateringPlanQuery()

        expect(options.queryKey).toContain('watering-plans')
        expect(options.queryFn).toBeDefined()
      })

      it('includes default page in query key', () => {
        const options = wateringPlanQuery()

        expect(options.queryKey).toContain('1')
      })

      it('includes custom page in query key', () => {
        const options = wateringPlanQuery({ page: 3 })

        expect(options.queryKey).toContain(3)
      })

      it('calls wateringPlanApi.listWateringPlans when queryFn is executed', async () => {
        const mockResponse = { data: [] } as unknown as ListResponseWateringPlanInListResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(wateringPlanApi.listWateringPlans).mockResolvedValueOnce(mockResponse)

        const options = wateringPlanQuery()
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(wateringPlanApi.listWateringPlans).toHaveBeenCalledWith(undefined)
        expect(result).toEqual(mockResponse)
      })
    })

    describe('wateringPlanIdQuery', () => {
      it('returns correct query options for fetching single watering plan', () => {
        const options = wateringPlanIdQuery('55')

        expect(options.queryKey).toEqual(['watering-plan', '55'])
        expect(options.queryFn).toBeDefined()
      })

      it('calls wateringPlanApi.getWateringPlan with correct id', async () => {
        const mockPlan = { id: 55, date: '2025-01-15' } as WateringPlan
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(wateringPlanApi.getWateringPlan).mockResolvedValueOnce(mockPlan)

        const options = wateringPlanIdQuery('55')
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(wateringPlanApi.getWateringPlan).toHaveBeenCalledWith({ wateringPlanId: 55 })
        expect(result).toEqual(mockPlan)
      })
    })
  })
})
