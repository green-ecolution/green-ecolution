import { lazy } from 'react'
import { defineSettingsSection } from '@/modules/settings'
import {
  useIrrigationSettingsDraft,
  type IrrigationSettingsDraft,
} from './useIrrigationSettingsDraft'

export const irrigationSettingsSection = defineSettingsSection<IrrigationSettingsDraft>({
  useDraft: useIrrigationSettingsDraft,
  Section: lazy(() => import('./IrrigationSettingsSection')),
})
