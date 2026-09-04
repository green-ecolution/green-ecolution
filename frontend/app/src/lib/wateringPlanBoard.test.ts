import { describe, expect, it, beforeAll } from 'vitest'
import type { TFunction } from 'i18next'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import { getI18n } from '@/lib/i18n'
import { columnForStatus, dropActionFor, dropHintFor } from './wateringPlanBoard'

// vitest setup already calls createI18n(); fix the language so assertions
// against the German catalog text stay stable regardless of test order.
let t: TFunction<'wateringPlan'>
beforeAll(() => {
  t = getI18n().getFixedT('de', 'wateringPlan')
})

describe('columnForStatus', () => {
  it('maps planned and active to their columns', () => {
    expect(columnForStatus(WateringPlanStatus.Planned)).toBe('planned')
    expect(columnForStatus(WateringPlanStatus.Active)).toBe('active')
  })

  it('maps all terminal statuses to done', () => {
    expect(columnForStatus(WateringPlanStatus.Finished)).toBe('done')
    expect(columnForStatus(WateringPlanStatus.Canceled)).toBe('done')
    expect(columnForStatus(WateringPlanStatus.NotCompleted)).toBe('done')
  })

  it('maps every status to a column', () => {
    for (const status of Object.values(WateringPlanStatus)) {
      expect(columnForStatus(status)).not.toBeUndefined()
    }
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
    expect(dropHintFor('start', t)).toBe('Einsatzplan starten')
    expect(dropHintFor('cancel', t)).toBe('Einsatzplan abbrechen')
    expect(dropHintFor('complete', t)).toBe('Einsatzplan abschließen')
  })
})
