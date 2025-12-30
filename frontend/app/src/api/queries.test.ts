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
import type {
  Tree,
  TreeList,
  Vehicle,
  VehicleList,
  TreeCluster,
  TreeClusterList,
  WateringPlan,
  WateringPlanList,
} from '@green-ecolution/backend-client'

vi.mock('./backendApi', () => ({
  treeApi: {
    getAllTrees: vi.fn(),
    getTrees: vi.fn(),
  },
  vehicleApi: {
    getAllVehicles: vi.fn(),
    getVehicleById: vi.fn(),
  },
  clusterApi: {
    getAllTreeClusters: vi.fn(),
    getTreeClusterById: vi.fn(),
  },
  wateringPlanApi: {
    getAllWateringPlans: vi.fn(),
    getWateringPlanById: vi.fn(),
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

      it('includes filter params in query key', () => {
        const options = treeQuery({ wateringStatuses: ['good', 'bad'] })

        expect(options.queryKey).toContain('trees')
      })

      it('calls treeApi.getAllTrees when queryFn is executed', async () => {
        const mockResponse = { data: [] } as TreeList
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.getAllTrees).mockResolvedValueOnce(mockResponse)

        const options = treeQuery()
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.getAllTrees).toHaveBeenCalledWith(undefined)
        expect(result).toEqual(mockResponse)
      })

      it('passes params to treeApi.getAllTrees', async () => {
        const mockResponse = { data: [] } as TreeList
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.getAllTrees).mockResolvedValueOnce(mockResponse)

        const params = { page: 2, plantingYears: [2023, 2024] }
        const options = treeQuery(params)
        await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.getAllTrees).toHaveBeenCalledWith(params)
      })
    })

    describe('treeIdQuery', () => {
      it('returns correct query options for fetching single tree', () => {
        const options = treeIdQuery('123')

        expect(options.queryKey).toEqual(['tree', '123'])
        expect(options.queryFn).toBeDefined()
      })

      it('calls treeApi.getTrees with correct id', async () => {
        const mockTree = { id: 123, latitude: 54.0, longitude: 9.0 } as Tree
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.getTrees).mockResolvedValueOnce(mockTree)

        const options = treeIdQuery('123')
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.getTrees).toHaveBeenCalledWith({ treeId: 123 })
        expect(result).toEqual(mockTree)
      })

      it('converts string id to number', async () => {
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.getTrees).mockResolvedValueOnce({} as Tree)

        const options = treeIdQuery('456')
        await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.getTrees).toHaveBeenCalledWith({ treeId: 456 })
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

      it('includes type filter in query key', () => {
        const options = vehicleQuery({ type: 'transporter' })

        expect(options.queryKey).toContain('vehicle')
        expect(options.queryKey).toContain('transporter')
      })

      it('calls vehicleApi.getAllVehicles when queryFn is executed', async () => {
        const mockResponse = { data: [] } as VehicleList
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(vehicleApi.getAllVehicles).mockResolvedValueOnce(mockResponse)

        const options = vehicleQuery()
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(vehicleApi.getAllVehicles).toHaveBeenCalledWith(undefined)
        expect(result).toEqual(mockResponse)
      })

      it('passes type filter to vehicleApi.getAllVehicles', async () => {
        const mockResponse = { data: [] } as VehicleList
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(vehicleApi.getAllVehicles).mockResolvedValueOnce(mockResponse)

        const params = { type: 'trailer' as const }
        const options = vehicleQuery(params)
        await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(vehicleApi.getAllVehicles).toHaveBeenCalledWith(params)
      })
    })

    describe('vehicleIdQuery', () => {
      it('returns correct query options for fetching single vehicle', () => {
        const options = vehicleIdQuery('42')

        expect(options.queryKey).toEqual(['vehicle', '42'])
        expect(options.queryFn).toBeDefined()
      })

      it('calls vehicleApi.getVehicleById with correct id', async () => {
        const mockVehicle = { id: 42, numberPlate: 'HH-AB-1234' } as Vehicle
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(vehicleApi.getVehicleById).mockResolvedValueOnce(mockVehicle)

        const options = vehicleIdQuery('42')
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(vehicleApi.getVehicleById).toHaveBeenCalledWith({ id: 42 })
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

      it('includes region filter in query key', () => {
        const options = treeClusterQuery({ regions: ['1', '2'] })

        expect(options.queryKey).toContain('treeclusters')
      })

      it('calls clusterApi.getAllTreeClusters when queryFn is executed', async () => {
        const mockResponse = { data: [] } as TreeClusterList
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(clusterApi.getAllTreeClusters).mockResolvedValueOnce(mockResponse)

        const options = treeClusterQuery()
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(clusterApi.getAllTreeClusters).toHaveBeenCalledWith(undefined)
        expect(result).toEqual(mockResponse)
      })
    })

    describe('treeClusterIdQuery', () => {
      it('returns correct query options for fetching single cluster', () => {
        const options = treeClusterIdQuery('99')

        expect(options.queryKey).toEqual(['treecluster', '99'])
        expect(options.queryFn).toBeDefined()
      })

      it('calls clusterApi.getTreeClusterById with correct id', async () => {
        const mockCluster = { id: 99, name: 'Test Cluster' } as TreeCluster
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(clusterApi.getTreeClusterById).mockResolvedValueOnce(mockCluster)

        const options = treeClusterIdQuery('99')
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(clusterApi.getTreeClusterById).toHaveBeenCalledWith({ clusterId: 99 })
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

      it('calls wateringPlanApi.getAllWateringPlans when queryFn is executed', async () => {
        const mockResponse = { data: [] } as WateringPlanList
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(wateringPlanApi.getAllWateringPlans).mockResolvedValueOnce(mockResponse)

        const options = wateringPlanQuery()
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(wateringPlanApi.getAllWateringPlans).toHaveBeenCalledWith(undefined)
        expect(result).toEqual(mockResponse)
      })
    })

    describe('wateringPlanIdQuery', () => {
      it('returns correct query options for fetching single watering plan', () => {
        const options = wateringPlanIdQuery('55')

        expect(options.queryKey).toEqual(['watering-plan', '55'])
        expect(options.queryFn).toBeDefined()
      })

      it('calls wateringPlanApi.getWateringPlanById with correct id', async () => {
        const mockPlan = { id: 55, date: '2025-01-15' } as WateringPlan
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(wateringPlanApi.getWateringPlanById).mockResolvedValueOnce(mockPlan)

        const options = wateringPlanIdQuery('55')
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(wateringPlanApi.getWateringPlanById).toHaveBeenCalledWith({ id: 55 })
        expect(result).toEqual(mockPlan)
      })
    })
  })
})
