import { describe, expect, it, beforeAll } from 'vitest'
import type { TFunction } from 'i18next'
import { PlanStatus } from '@green-ecolution/backend-client'
import { getI18n } from '@/lib/i18n'
import { columnForStatus, dropActionFor, dropHintFor } from './planBoard'

// vitest setup already calls createI18n(); fix the language so assertions
// against the German catalog text stay stable regardless of test order.
let t: TFunction<'wateringPlan'>
beforeAll(() => {
  t = getI18n().getFixedT('de', 'wateringPlan')
})

describe('columnForStatus', () => {
  it('maps planned and active to their columns', () => {
    expect(columnForStatus(PlanStatus.Planned)).toBe('planned')
    expect(columnForStatus(PlanStatus.Active)).toBe('active')
  })

  it('maps all terminal statuses to done', () => {
    expect(columnForStatus(PlanStatus.Finished)).toBe('done')
    expect(columnForStatus(PlanStatus.Canceled)).toBe('done')
    expect(columnForStatus(PlanStatus.NotCompleted)).toBe('done')
  })

  it('maps every status to a column', () => {
    for (const status of Object.values(PlanStatus)) {
      expect(columnForStatus(status)).not.toBeUndefined()
    }
  })
})

describe('dropActionFor', () => {
  it('allows the state machine transitions', () => {
    expect(dropActionFor('planned', 'active')).toBe('start')
    expect(dropActionFor('planned', 'done')).toBe('cancel')
    expect(dropActionFor('active', 'done')).toBe('complete')
    expect(dropActionFor('active', 'planned')).toBe('revertStart')
  })

  it('rejects everything else', () => {
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
    expect(dropHintFor('revertStart', t)).toBe('Start zurücknehmen')
  })
})
