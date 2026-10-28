import { describe, it, expect, vi, beforeEach } from 'vitest'
import {
  commentQueries,
  commentQueryKey,
  vehicleQueries,
  wateringPlanQueries,
  sensorQueries,
} from './queries'
import type { Vehicle } from '@/api/backendApi'
import type {
  ListResponseVehicleResponse,
  PlanResponse,
  RouteResponse,
} from '@green-ecolution/backend-client'
import { PlanStatus, SensorStatus } from '@green-ecolution/backend-client'

vi.mock('./backendApi', () => ({
  vehicleApi: {
    listVehicles: vi.fn(),
    getVehicle: vi.fn(),
  },
  commentApi: {
    listPlanComments: vi.fn(),
    listTargetComments: vi.fn(),
  },
  planApi: {
    listPlans: vi.fn(),
    getPlan: vi.fn(),
  },
  planPreviewApi: {
    previewPlanRoute: vi.fn(),
  },
}))

import { commentApi, vehicleApi, planApi, planPreviewApi } from './backendApi'

describe('Query Functions', () => {
  beforeEach(() => {
    vi.clearAllMocks()
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

  describe('WateringPlan Queries', () => {
    describe('wateringPlanQueries.boardColumn', () => {
      it('lists the plans of one module in the given statuses', async () => {
        const options = wateringPlanQueries.boardColumn('irrigation', [PlanStatus.Planned])
        await options.queryFn!({} as never)

        expect(options.queryKey).toEqual(['watering-plans', 'board', 'irrigation', ['planned']])
        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(planApi.listPlans).toHaveBeenCalledWith({
          module: 'irrigation',
          status: ['planned'],
          page: 1,
          perPage: 100,
        })
      })
    })

    describe('wateringPlanQueries.detail', () => {
      it('returns correct query options for fetching single watering plan', () => {
        const options = wateringPlanQueries.detail('55')

        expect(options.queryKey).toEqual(['watering-plan', '55'])
        expect(options.queryFn).toBeDefined()
      })

      it('returns the plan as the backend sends it', async () => {
        const mockPlan = { id: 'plan-uuid-1', module: 'irrigation', stops: ['c2'] } as PlanResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(planApi.getPlan).mockResolvedValueOnce(mockPlan)

        const result = await wateringPlanQueries.detail('plan-uuid-1').queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(planApi.getPlan).toHaveBeenCalledWith({ planId: 'plan-uuid-1' })
        expect(result).toBe(mockPlan)
      })
    })

    describe('wateringPlanQueries.routePreview', () => {
      const preview = (overrides = {}) =>
        wateringPlanQueries.routePreview({
          module: 'irrigation',
          stops: ['cluster-1'],
          transporterId: 'vehicle-1',
          ...overrides,
        })

      it('returns the route on success', async () => {
        const mockRoute = { stationVisits: [{ name: 'Hof', lat: 1, lon: 2 }] } as RouteResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(planPreviewApi.previewPlanRoute).mockResolvedValueOnce(mockRoute)

        expect(await preview().queryFn!({} as never)).toBe(mockRoute)
      })

      it('returns null on any error instead of propagating', async () => {
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(planPreviewApi.previewPlanRoute).mockRejectedValueOnce(new Error('422'))

        expect(await preview().queryFn!({} as never)).toBeNull()
      })

      it('sends the organization the plan belongs to and keys the cache by it', async () => {
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(planPreviewApi.previewPlanRoute).mockResolvedValueOnce({} as RouteResponse)

        const query = preview({ organizationId: 'org-1' })
        await query.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(planPreviewApi.previewPlanRoute).toHaveBeenCalledWith({
          planRouteRequest: expect.objectContaining({ organizationId: 'org-1' }) as unknown,
        })
        expect(query.queryKey).toContain('org-1')
        expect(preview().queryKey).not.toEqual(query.queryKey)
      })

      it('uses planPreviewApi, not planApi', async () => {
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(planPreviewApi.previewPlanRoute).mockResolvedValueOnce({} as RouteResponse)

        await preview().queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(planPreviewApi.previewPlanRoute).toHaveBeenCalled()
        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(planApi.listPlans).not.toHaveBeenCalled()
      })

      it('routes the module stops with the trailer and without an unused transporter tank', async () => {
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(planPreviewApi.previewPlanRoute).mockResolvedValueOnce({} as RouteResponse)

        await preview({ trailerId: 'trailer-1', transporterCapacityUsed: false }).queryFn!(
          {} as never,
        )

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

  describe('Comment Queries', () => {
    const CLUSTER = '0190a8e9-7c4f-7000-8000-000000000001'
    const clusterSubject = {
      kind: 'target',
      target: { module: 'irrigation', kind: 'cluster', id: CLUSTER },
    } as const

    it('keys a target subject by module, kind and id below the comments root', () => {
      expect(commentQueryKey(clusterSubject)).toEqual([
        'comments',
        'target',
        'irrigation',
        'cluster',
        CLUSTER,
      ])
      expect(commentQueryKey({ kind: 'plan', id: CLUSTER })).toEqual(['comments', 'plan', CLUSTER])
    })

    it('lists a target subject through the target comment route', async () => {
      await commentQueries.list(clusterSubject).queryFn!({ pageParam: 2 } as never)

      // eslint-disable-next-line @typescript-eslint/unbound-method
      expect(commentApi.listTargetComments).toHaveBeenCalledWith({
        module: 'irrigation',
        kind: 'cluster',
        targetId: CLUSTER,
        page: 2,
        perPage: 20,
      })
    })

    it('lists a plan subject through the plan comment route', async () => {
      await commentQueries.list({ kind: 'plan', id: CLUSTER }).queryFn!({ pageParam: 1 } as never)

      // eslint-disable-next-line @typescript-eslint/unbound-method
      expect(commentApi.listPlanComments).toHaveBeenCalledWith({
        planId: CLUSTER,
        page: 1,
        perPage: 20,
      })
    })
  })
})
