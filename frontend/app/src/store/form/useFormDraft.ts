import { useCallback, useMemo } from 'react'
import useStore from '@/store/store'
import { FormDraftKey, FormDraftState, FormType, MutationType } from './formDraftSlice'

export function useFormDraft<T>(formType: FormType, mutationType: MutationType) {
  const key: FormDraftKey = `${formType}-${mutationType}`

  const draft = useStore((state) => state.formDrafts[key]) as FormDraftState<T> | undefined
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
      data: (draft?.data ?? null) as T | null,
      hasChanges: draft?.hasChanges ?? false,
      setData,
      updateData,
      markChanged,
      clear,
    }),
    [draft?.data, draft?.hasChanges, setData, updateData, markChanged, clear],
  )
}

export const useTreeDraft = <T>(mutationType: MutationType) => useFormDraft<T>('tree', mutationType)

export const useClusterDraft = <T>(mutationType: MutationType) =>
  useFormDraft<T>('cluster', mutationType)

export const useWateringPlanDraft = <T>(mutationType: MutationType) =>
  useFormDraft<T>('wateringplan', mutationType)
