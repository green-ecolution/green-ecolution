import { useCallback, useMemo } from 'react'
import useStore from '@/store/store'
import { useOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'
import {
  FormDraftKey,
  FormDraftState,
  FormType,
  MutationType,
  WATERING_PLAN_FORM,
} from './formDraftSlice'

export function useFormDraft<T>(formType: FormType, mutationType: MutationType) {
  const key: FormDraftKey = `${formType}-${mutationType}`

  const scope = useOrganizationScope()
  const stored = useStore((state) => state.formDrafts[key]) as FormDraftState<T> | undefined
  const draft = stored?.scope === scope ? stored : undefined
  const setFormDraft = useStore((state) => state.setFormDraft)
  const updateFormDraft = useStore((state) => state.updateFormDraft)
  const markFormDraftChanged = useStore((state) => state.markFormDraftChanged)
  const clearFormDraft = useStore((state) => state.clearFormDraft)

  const setData = useCallback((data: T) => setFormDraft(key, data), [key, setFormDraft])
  const updateData = useCallback(
    (updater: (prev: T | null) => T) => updateFormDraft(key, updater),
    [key, updateFormDraft],
  )
  const markChanged = useCallback(() => markFormDraftChanged(key), [key, markFormDraftChanged])
  const clear = useCallback(() => clearFormDraft(key), [key, clearFormDraft])

  return useMemo(
    () => ({
      data: draft?.data ?? null,
      hasChanges: draft?.hasChanges ?? false,
      setData,
      updateData,
      markChanged,
      clear,
    }),
    [draft?.data, draft?.hasChanges, setData, updateData, markChanged, clear],
  )
}

export const useWateringPlanDraft = <T>(mutationType: MutationType) =>
  useFormDraft<T>(WATERING_PLAN_FORM, mutationType)
