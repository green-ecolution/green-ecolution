import type { GeolocationFix } from '@/hooks/useGeolocation'

export const normalizeSensorId = (raw: string): string => {
  const trimmed = raw.trim().toLowerCase()
  return trimmed.startsWith('eui-') ? trimmed : `eui-${trimmed}`
}

export type WizardStep = 1 | 2 | 3 | 4

export type SubmissionState = 'idle' | 'pending' | 'success' | 'error'

export interface WizardState {
  step: WizardStep
  sensorId: string | null
  frozenFix: GeolocationFix | null
  selectedTreeId: string | null
  selectedTreeNumber: string | null
  selectedTreeSpecies: string | null
  submission: SubmissionState
  errorMessage: string | null
}

export type WizardAction =
  | { type: 'qrScanned'; sensorId: string }
  | { type: 'scanCleared' }
  | { type: 'gpsFrozen'; fix: GeolocationFix }
  | { type: 'gpsCleared' }
  | { type: 'treeSelected'; treeId: string; number: string; species: string }
  | { type: 'goToStep'; step: WizardStep }
  | { type: 'submissionStart' }
  | { type: 'submissionSuccess' }
  | { type: 'submissionError'; message: string }
  | { type: 'resetForNextSensor' }

export const INITIAL_WIZARD_STATE: WizardState = {
  step: 1,
  sensorId: null,
  frozenFix: null,
  selectedTreeId: null,
  selectedTreeNumber: null,
  selectedTreeSpecies: null,
  submission: 'idle',
  errorMessage: null,
}

const clearTree = {
  selectedTreeId: null,
  selectedTreeNumber: null,
  selectedTreeSpecies: null,
} as const

export const wizardReducer = (state: WizardState, action: WizardAction): WizardState => {
  switch (action.type) {
    case 'qrScanned':
      return {
        ...state,
        sensorId: action.sensorId,
        frozenFix: null,
        ...clearTree,
        submission: 'idle',
        errorMessage: null,
      }
    case 'scanCleared':
      return { ...state, sensorId: null }
    case 'gpsFrozen':
      return { ...state, frozenFix: action.fix, ...clearTree }
    case 'gpsCleared':
      return { ...state, frozenFix: null, ...clearTree }
    case 'treeSelected':
      return {
        ...state,
        selectedTreeId: action.treeId,
        selectedTreeNumber: action.number,
        selectedTreeSpecies: action.species,
      }
    case 'goToStep': {
      const target = action.step
      if (target <= state.step) return { ...state, step: target }
      if (target === state.step + 1) return { ...state, step: target }
      return state
    }
    case 'submissionStart':
      return { ...state, submission: 'pending', errorMessage: null }
    case 'submissionSuccess':
      return { ...state, submission: 'success', errorMessage: null }
    case 'submissionError':
      return { ...state, submission: 'error', errorMessage: action.message }
    case 'resetForNextSensor':
      return INITIAL_WIZARD_STATE
  }
}
