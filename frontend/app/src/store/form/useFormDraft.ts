import useStore from '@/store/store'
import { FormDraftKey, FormDraftState, FormType, MutationType } from './formDraftSlice'

export function useFormDraft<T>(formType: FormType, mutationType: MutationType) {
  const key: FormDraftKey = `${formType}-${mutationType}`

  const draft = useStore((state) => state.formDrafts[key]) as FormDraftState<T> | undefined
  const setFormDraft = useStore((state) => state.setFormDraft)
  const updateFormDraft = useStore((state) => state.updateFormDraft)
  const markFormDraftChanged = useStore((state) => state.markFormDraftChanged)
  const clearFormDraft = useStore((state) => state.clearFormDraft)

  const setData = (data: T) => setFormDraft(key, data)
  const updateData = (updater: (prev: T | null) => T) => updateFormDraft(key, updater)
  const markChanged = () => markFormDraftChanged(key)
  const clear = () => clearFormDraft(key)

  return {
    data: (draft?.data ?? null) as T | null,
    hasChanges: draft?.hasChanges ?? false,
    setData,
    updateData,
    markChanged,
    clear,
  }
}

export const useTreeDraft = <T>(mutationType: MutationType) => useFormDraft<T>('tree', mutationType)

export const useClusterDraft = <T>(mutationType: MutationType) =>
  useFormDraft<T>('cluster', mutationType)

export const useWateringPlanDraft = <T>(mutationType: MutationType) =>
  useFormDraft<T>('wateringplan', mutationType)
