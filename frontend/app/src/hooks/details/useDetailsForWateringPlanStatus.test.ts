import { describe, it, expect } from 'vitest'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import { getWateringPlanStatusTransitionOptions } from './useDetailsForWateringPlanStatus'

const valuesFor = (status: WateringPlanStatus) =>
  getWateringPlanStatusTransitionOptions(status).map((option) => option.value)

describe('getWateringPlanStatusTransitionOptions', () => {
  it('offers start and cancel from a planned plan', () => {
    expect(valuesFor(WateringPlanStatus.Planned)).toEqual([
      WateringPlanStatus.Planned,
      WateringPlanStatus.Active,
      WateringPlanStatus.Canceled,
    ])
  })

  it('offers revert, finish, fail and cancel from an active plan', () => {
    expect(valuesFor(WateringPlanStatus.Active)).toEqual([
      WateringPlanStatus.Active,
      WateringPlanStatus.Planned,
      WateringPlanStatus.Finished,
      WateringPlanStatus.NotCompleted,
      WateringPlanStatus.Canceled,
    ])
  })

  it.each([
    WateringPlanStatus.Finished,
    WateringPlanStatus.Canceled,
    WateringPlanStatus.NotCompleted,
    WateringPlanStatus.Unknown,
  ])('offers no transition out of the terminal status %s', (status) => {
    expect(valuesFor(status)).toEqual([])
  })

  it('never offers "unknown" as a target', () => {
    for (const status of Object.values(WateringPlanStatus)) {
      expect(valuesFor(status)).not.toContain(WateringPlanStatus.Unknown)
    }
  })

  it('carries the label and colour from the status catalogue', () => {
    const [current, next] = getWateringPlanStatusTransitionOptions(WateringPlanStatus.Planned)
    expect(current.label).toBe('Geplant')
    expect(next.label).toBe('Aktiv')
  })
})
