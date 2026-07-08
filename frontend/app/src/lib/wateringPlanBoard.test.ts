import { describe, expect, it } from 'vitest'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import { columnForStatus, dropActionFor, dropHintFor } from './wateringPlanBoard'

describe('columnForStatus', () => {
  it('maps planned and active to their columns', () => {
    expect(columnForStatus(WateringPlanStatus.Planned)).toBe('planned')
    expect(columnForStatus(WateringPlanStatus.Active)).toBe('active')
  })

  it('maps all terminal statuses to done', () => {
    expect(columnForStatus(WateringPlanStatus.Finished)).toBe('done')
    expect(columnForStatus(WateringPlanStatus.Canceled)).toBe('done')
    expect(columnForStatus(WateringPlanStatus.NotCompeted)).toBe('done')
  })

  it('maps unknown to null', () => {
    expect(columnForStatus(WateringPlanStatus.Unknown)).toBeNull()
  })
})

describe('dropActionFor', () => {
  it('allows the state machine transitions', () => {
    expect(dropActionFor('planned', 'active')).toBe('start')
    expect(dropActionFor('planned', 'done')).toBe('cancel')
    expect(dropActionFor('active', 'done')).toBe('complete')
  })

  it('rejects everything else', () => {
    expect(dropActionFor('active', 'planned')).toBeNull()
    expect(dropActionFor('done', 'active')).toBeNull()
    expect(dropActionFor('done', 'planned')).toBeNull()
    expect(dropActionFor('planned', 'planned')).toBeNull()
  })
})

describe('dropHintFor', () => {
  it('labels actions with their consequence', () => {
    expect(dropHintFor('start')).toBe('Einsatz starten')
    expect(dropHintFor('cancel')).toBe('Einsatz abbrechen')
    expect(dropHintFor('complete')).toBe('Einsatz abschließen')
  })
})
