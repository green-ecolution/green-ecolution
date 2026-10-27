import type { ModuleSettingsDraft, ModuleSettingsSection } from './types'

export const defineSettingsSection = <D extends ModuleSettingsDraft>(
  section: ModuleSettingsSection<D>,
): ModuleSettingsSection =>
  // The page hands a section only the draft its own `useDraft` returned.
  section as unknown as ModuleSettingsSection
