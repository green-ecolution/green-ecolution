import { describe, expect, it } from 'vitest'
import {
  INITIAL_WIZARD_STATE,
  wizardReducer,
  type WizardState,
} from './state'

const fix = {
  latitude: 54.79,
  longitude: 9.43,
  accuracy: 8,
  altitude: null,
  altitudeAccuracy: null,
  heading: null,
  speed: null,
  timestamp: 1_710_000_000_000,
}

describe('wizardReducer', () => {
  it('starts in step 1 with empty fields', () => {
    expect(INITIAL_WIZARD_STATE.step).toBe(1)
    expect(INITIAL_WIZARD_STATE.sensorId).toBeNull()
    expect(INITIAL_WIZARD_STATE.frozenFix).toBeNull()
    expect(INITIAL_WIZARD_STATE.selectedTreeId).toBeNull()
    expect(INITIAL_WIZARD_STATE.submission).toBe('idle')
  })

  it('qrScanned stores sensor id and clears downstream fields', () => {
    const seeded: WizardState = {
      ...INITIAL_WIZARD_STATE,
      step: 3,
      sensorId: 'OLD',
      frozenFix: fix,
      selectedTreeId: 'tree-a',
      selectedTreeNumber: '42',
      selectedTreeSpecies: 'Quercus',
    }
    const next = wizardReducer(seeded, { type: 'qrScanned', sensorId: 'NEW' })
    expect(next.sensorId).toBe('NEW')
    expect(next.frozenFix).toBeNull()
    expect(next.selectedTreeId).toBeNull()
    expect(next.selectedTreeNumber).toBeNull()
    expect(next.selectedTreeSpecies).toBeNull()
  })

  it('scanCleared nullifies sensorId but keeps the rest of the state', () => {
    const seeded: WizardState = {
      ...INITIAL_WIZARD_STATE,
      sensorId: 'EUI-1',
      frozenFix: fix,
      selectedTreeId: 'tree-a',
      selectedTreeNumber: '1',
      selectedTreeSpecies: 'Acer',
    }
    const next = wizardReducer(seeded, { type: 'scanCleared' })
    expect(next.sensorId).toBeNull()
    expect(next.frozenFix).toEqual(fix)
    expect(next.selectedTreeId).toBe('tree-a')
  })

  it('gpsFrozen invalidates the selected tree (different position)', () => {
    const seeded: WizardState = {
      ...INITIAL_WIZARD_STATE,
      sensorId: 'S',
      frozenFix: fix,
      selectedTreeId: 'tree-a',
      selectedTreeNumber: '1',
      selectedTreeSpecies: 'Acer',
    }
    const next = wizardReducer(seeded, {
      type: 'gpsFrozen',
      fix: { ...fix, latitude: 54.80 },
    })
    expect(next.frozenFix?.latitude).toBe(54.80)
    expect(next.selectedTreeId).toBeNull()
    expect(next.selectedTreeNumber).toBeNull()
  })

  it('treeSelected stores id + display fields', () => {
    const next = wizardReducer(INITIAL_WIZARD_STATE, {
      type: 'treeSelected',
      treeId: 'tree-1',
      number: '0815',
      species: 'Tilia cordata',
    })
    expect(next.selectedTreeId).toBe('tree-1')
    expect(next.selectedTreeNumber).toBe('0815')
    expect(next.selectedTreeSpecies).toBe('Tilia cordata')
  })

  it('goToStep allows jumping back to any step', () => {
    const seeded: WizardState = { ...INITIAL_WIZARD_STATE, step: 4 }
    expect(wizardReducer(seeded, { type: 'goToStep', step: 2 }).step).toBe(2)
    expect(wizardReducer(seeded, { type: 'goToStep', step: 1 }).step).toBe(1)
  })

  it('goToStep is a no-op when jumping forward past the current step', () => {
    const seeded: WizardState = { ...INITIAL_WIZARD_STATE, step: 2 }
    expect(wizardReducer(seeded, { type: 'goToStep', step: 4 }).step).toBe(2)
  })

  it('goToStep allows moving to the immediate next step', () => {
    const seeded: WizardState = {
      ...INITIAL_WIZARD_STATE,
      step: 2,
      sensorId: 'S',
      frozenFix: fix,
    }
    expect(wizardReducer(seeded, { type: 'goToStep', step: 3 }).step).toBe(3)
  })

  it('submissionSuccess marks state as success and clears error', () => {
    const seeded: WizardState = {
      ...INITIAL_WIZARD_STATE,
      step: 4,
      submission: 'pending',
      errorMessage: 'whoops',
    }
    const next = wizardReducer(seeded, { type: 'submissionSuccess' })
    expect(next.submission).toBe('success')
    expect(next.errorMessage).toBeNull()
  })

  it('submissionError keeps step + data, sets error message', () => {
    const seeded: WizardState = {
      ...INITIAL_WIZARD_STATE,
      step: 4,
      sensorId: 'S',
      submission: 'pending',
    }
    const next = wizardReducer(seeded, {
      type: 'submissionError',
      message: 'boom',
    })
    expect(next.step).toBe(4)
    expect(next.sensorId).toBe('S')
    expect(next.submission).toBe('error')
    expect(next.errorMessage).toBe('boom')
  })

  it('resetForNextSensor returns to initial state', () => {
    const seeded: WizardState = {
      step: 4,
      sensorId: 'S',
      frozenFix: fix,
      selectedTreeId: 'T',
      selectedTreeNumber: '1',
      selectedTreeSpecies: 'X',
      submission: 'success',
      errorMessage: null,
    }
    expect(wizardReducer(seeded, { type: 'resetForNextSensor' })).toEqual(
      INITIAL_WIZARD_STATE,
    )
  })
})
