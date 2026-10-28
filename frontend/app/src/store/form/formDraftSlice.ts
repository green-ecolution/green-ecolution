export const WATERING_PLAN_FORM = 'wateringplan'
export const VEHICLE_FORM = 'vehicle'

// Core uses the constants above; modules bring their own keys.
export type FormType = string
export type MutationType = 'create' | 'update'

export type FormDraftKey = `${FormType}-${MutationType}`

export interface FormDraftState<T = unknown> {
  data: T | null
  hasChanges: boolean
  /** Organization scope the draft was made under; a draft is void under another one. */
  scope: string | null
}

export interface FormDraftSlice {
  formDrafts: Partial<Record<FormDraftKey, FormDraftState>>
  setFormDraft: <T>(key: FormDraftKey, data: T) => void
  updateFormDraft: <T>(key: FormDraftKey, updater: (prev: T | null) => T) => void
  markFormDraftChanged: (key: FormDraftKey) => void
  clearFormDraft: (key: FormDraftKey) => void
  clearAllFormDrafts: () => void
}
