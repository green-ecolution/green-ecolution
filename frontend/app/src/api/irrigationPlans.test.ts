import { describe, expect, it } from 'vitest'
import type {
  PlanResponse,
  RouteResponse,
  TreeClusterInListResponse,
} from '@green-ecolution/backend-client'
import {
  toPlanCreateRequest,
  toPlanUpdateRequest,
  toWateringPlan,
  toWateringRoute,
} from './irrigationPlans'

const cluster = (id: string): TreeClusterInListResponse =>
  ({ id, name: `Cluster ${id}` }) as TreeClusterInListResponse

const plan = (overrides: Partial<PlanResponse> = {}): PlanResponse =>
  ({
    id: 'p1',
    module: 'irrigation',
    stops: ['c1', 'c2'],
    outcomes: [
      { stop: 'c1', completed: true, amount: 120 },
      { stop: 'c2', completed: true, amount: 80 },
    ],
    transporterCapacityUsed: false,
    totalDemand: 200,
    ...overrides,
  }) as PlanResponse

describe('irrigation plan adapter', () => {
  it('resolves stops to clusters in stop order and skips deleted ones', () => {
    const clusters = new Map([
      ['c2', cluster('c2')],
      ['c1', cluster('c1')],
    ])
    const result = toWateringPlan(plan({ stops: ['c1', 'gone', 'c2'] }), clusters)
    expect(result.treeclusters.map((c) => c.id)).toEqual(['c1', 'c2'])
  })

  it('maps outcomes to the evaluation the dashboard shows', () => {
    const result = toWateringPlan(plan(), new Map())
    expect(result.evaluation).toEqual([
      { treeClusterId: 'c1', wateringPlanId: 'p1', consumedWater: 120 },
      { treeClusterId: 'c2', wateringPlanId: 'p1', consumedWater: 80 },
    ])
  })

  it('leaves unfinished stops out of the evaluation', () => {
    const result = toWateringPlan(
      plan({
        outcomes: [
          { stop: 'c1', completed: false, amount: null },
          { stop: 'c2', completed: false, amount: null },
        ],
      }),
      new Map(),
    )
    expect(result.evaluation).toEqual([])
  })

  it('keeps the water vocabulary on the plan', () => {
    const result = toWateringPlan(plan(), new Map())
    expect(result.transporterTankUsed).toBe(false)
    expect(result.totalWaterRequired).toBe(200)
  })

  it('creates an irrigation plan from the form', () => {
    const request = toPlanCreateRequest({
      treeClusterIds: ['c1'],
      transporterTankUsed: true,
    } as Parameters<typeof toPlanCreateRequest>[0])
    expect(request.module).toBe('irrigation')
    expect(request.stops).toEqual(['c1'])
    expect(request.transporterCapacityUsed).toBe(true)
  })

  it('turns an evaluation into completed outcomes', () => {
    const request = toPlanUpdateRequest({
      treeClusterIds: ['c1'],
      transporterTankUsed: true,
      evaluation: [{ treeClusterId: 'c1', wateringPlanId: 'p1', consumedWater: 50 }],
    } as Parameters<typeof toPlanUpdateRequest>[0])
    expect(request.outcomes).toEqual([{ stop: 'c1', completed: true, amount: 50 }])
  })

  it('maps station visits to refill points', () => {
    const route = toWateringRoute({
      stationVisits: [{ name: 'Hof', lat: 54.8, lon: 9.4 }],
      totalDemand: 300,
      refillCount: 1,
    } as RouteResponse)
    expect(route.refillPoints).toEqual([{ name: 'Hof', lat: 54.8, lon: 9.4 }])
    expect(route.totalWaterRequired).toBe(300)
  })
})
