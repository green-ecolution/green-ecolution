import { describe, expect, it } from 'vitest'
import { PlanStatus } from '@green-ecolution/backend-client'
import type { WateringPlanForm } from '@/schema/wateringPlanSchema'
import { planInList } from '@/test/plans'
import { toPlanCreateRequest, toPlanFormUpdateRequest, toUpdateRequest } from './plans'

const form: WateringPlanForm = {
  module: 'waste',
  date: new Date('2026-08-01T00:00:00Z'),
  description: 'Altstadt',
  transporterId: 'v-1',
  trailerId: '-1',
  transporterTankUsed: true,
  stopIds: ['s1', 's2'],
  status: PlanStatus.Planned,
  driverIds: ['u1'],
  startPointName: 'Hof',
}

describe('toPlanCreateRequest', () => {
  it('creates the plan in the chosen module with its stops', () => {
    expect(toPlanCreateRequest(form)).toEqual({
      module: 'waste',
      date: '2026-08-01T00:00:00.000Z',
      description: 'Altstadt',
      transporterId: 'v-1',
      trailerId: undefined,
      transporterCapacityUsed: true,
      stops: ['s1', 's2'],
      userIds: ['u1'],
      startPointName: 'Hof',
    })
  })
})

describe('toPlanFormUpdateRequest', () => {
  it('saves the edited stops and leaves the outcomes alone', () => {
    const request = toPlanFormUpdateRequest({ ...form, trailerId: 't-1' })
    expect(request.stops).toEqual(['s1', 's2'])
    expect(request.trailerId).toBe('t-1')
    expect(request.outcomes).toBeUndefined()
  })
})

describe('toUpdateRequest', () => {
  it('keeps a trailer-only plan trailer-only when its status changes', () => {
    const plan = planInList({ transporterCapacityUsed: false, stops: ['s1', 'hidden'] })
    const request = toUpdateRequest(plan, { status: PlanStatus.Active })
    expect(request.transporterCapacityUsed).toBe(false)
    expect(request.stops).toEqual(['s1', 'hidden'])
    expect(request.status).toBe(PlanStatus.Active)
  })
})
