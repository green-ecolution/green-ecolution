import { lazy } from 'react'
import { defineSettingsSection } from '@/modules/settings'
import { useWasteSettingsDraft, type WasteSettingsDraft } from './useWasteSettingsDraft'

export const wasteSettingsSection = defineSettingsSection<WasteSettingsDraft>({
  useDraft: useWasteSettingsDraft,
  Section: lazy(() => import('./WasteSettingsSection')),
})
